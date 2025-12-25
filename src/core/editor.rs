use crate::core::{buffer::TextBuffer, cursor::Cursor};

pub struct Editor {
  pub buffer: TextBuffer,
  pub cursor: Cursor,
}

impl Editor {
  pub fn new() -> Self {
    Self {
      buffer: TextBuffer::new(),
      cursor: Cursor::new(),
    }
  }

  pub fn insert_char(&mut self, ch: char) {
    let mut buf = [0; 4];
    let s = ch.encode_utf8(&mut buf);
    self.buffer.insert(self.cursor.index, s);
    self.cursor.index += s.len();
  }

  pub fn backspace(&mut self) {
    if self.cursor.index > 0 {
      self.cursor.index -= 1;
      self.buffer.delete(self.cursor.index, 1);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_editor() {
    let editor = Editor::new();
    assert_eq!(editor.buffer.len(), 0);
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_insert_char() {
    let mut editor = Editor::new();

    editor.insert_char('H');
    assert_eq!(editor.buffer.as_str(), "H");
    assert_eq!(editor.cursor.index, 1);

    editor.insert_char('i');
    assert_eq!(editor.buffer.as_str(), "Hi");
    assert_eq!(editor.cursor.index, 2);
  }

  #[test]
  fn test_insert_char_in_middle() {
    let mut editor = Editor::new();
    editor.insert_char('H');
    editor.insert_char('e');
    editor.insert_char('l');
    editor.insert_char('l');
    editor.insert_char('o');

    // Move cursor back to middle
    editor.cursor.index = 2;
    editor.insert_char('X');

    assert_eq!(editor.buffer.as_str(), "HeXllo");
    assert_eq!(editor.cursor.index, 3);
  }

  #[test]
  fn test_backspace() {
    let mut editor = Editor::new();
    editor.insert_char('H');
    editor.insert_char('i');

    editor.backspace();
    assert_eq!(editor.buffer.as_str(), "H");
    assert_eq!(editor.cursor.index, 1);

    editor.backspace();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_backspace_at_start() {
    let mut editor = Editor::new();

    editor.backspace();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_backspace_in_middle() {
    let mut editor = Editor::new();
    editor.insert_char('H');
    editor.insert_char('e');
    editor.insert_char('l');
    editor.insert_char('l');
    editor.insert_char('o');

    // Move cursor to middle
    editor.cursor.index = 3;
    editor.backspace();

    assert_eq!(editor.buffer.as_str(), "Helo");
    assert_eq!(editor.cursor.index, 2);
  }

  #[test]
  fn test_multiline_editing() {
    let mut editor = Editor::new();

    editor.insert_char('L');
    editor.insert_char('i');
    editor.insert_char('n');
    editor.insert_char('e');
    editor.insert_char(' ');
    editor.insert_char('1');
    editor.insert_char('\n');
    editor.insert_char('L');
    editor.insert_char('i');
    editor.insert_char('n');
    editor.insert_char('e');
    editor.insert_char(' ');
    editor.insert_char('2');

    assert_eq!(editor.buffer.as_str(), "Line 1\nLine 2");
    assert_eq!(editor.cursor.index, 13);
  }

  #[test]
  fn test_unicode_insertion() {
    let mut editor = Editor::new();

    editor.insert_char('H');
    editor.insert_char('i');
    editor.insert_char(' ');
    editor.insert_char('🌍');

    assert_eq!(editor.buffer.as_str(), "Hi 🌍");
    // Emoji is 4 bytes in UTF-8
    assert_eq!(editor.cursor.index, 7);
  }

  #[test]
  fn test_cursor_movement_with_editor() {
    let mut editor = Editor::new();
    editor.insert_char('A');
    editor.insert_char('B');
    editor.insert_char('C');

    assert_eq!(editor.cursor.index, 3);

    editor.cursor.move_left();
    editor.insert_char('X');

    assert_eq!(editor.buffer.as_str(), "ABXC");
    assert_eq!(editor.cursor.index, 3);
  }

  #[test]
  fn test_complex_editing_workflow() {
    let mut editor = Editor::new();

    // Type "Hello World"
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }
    assert_eq!(editor.buffer.as_str(), "Hello World");

    // Delete " World" by backspacing 6 times
    for _ in 0..6 {
      editor.backspace();
    }
    assert_eq!(editor.buffer.as_str(), "Hello");

    // Add "!"
    editor.insert_char('!');
    assert_eq!(editor.buffer.as_str(), "Hello!");
  }
}
