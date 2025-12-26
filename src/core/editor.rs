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
    self.cursor.index += 1; // Increment by 1 character, not bytes
  }

  pub fn backspace(&mut self) {
    if self.cursor.index > 0 {
      self.cursor.index -= 1;
      self.buffer.delete(self.cursor.index, 1);
    }
  }

  pub fn delete_word(&mut self) {
    if self.cursor.index == 0 {
      return;
    }

    let start_index = self.cursor.index;
    let (current_line, current_col) = self.buffer.char_to_line_col(start_index);
    let line_start = self.buffer.line_col_to_char(current_line, 0);

    self.cursor.move_word_left(&self.buffer);
    let end_index = self.cursor.index;

    // If we're at the start of a line (col 0), allow deleting the newline
    // Otherwise, don't delete across line boundaries
    let delete_from = if current_col == 0 {
      end_index
    } else {
      end_index.max(line_start)
    };

    let count = start_index - delete_from;

    self.buffer.delete(delete_from, count);
    self.cursor.index = delete_from;
  }

  pub fn delete_line(&mut self) {
    let (line, _col) = self.buffer.char_to_line_col(self.cursor.index);
    let line_start = self.buffer.line_col_to_char(line, 0);

    // Calculate line length including the newline if it exists
    let line_content = self.buffer.line(line).unwrap_or_default();
    let line_len = line_content.chars().count();

    // Delete the entire line including newline
    self.buffer.delete(line_start, line_len);

    // Position cursor at the start of what's now at this line
    self.cursor.index = line_start;
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
    // We use character indices, not byte indices: 'H', 'i', ' ', '🌍' = 4 characters
    assert_eq!(editor.cursor.index, 4);
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

  #[test]
  fn test_delete_word() {
    let mut editor = Editor::new();
    for ch in "hello world".chars() {
      editor.insert_char(ch);
    }
    assert_eq!(editor.buffer.as_str(), "hello world");
    assert_eq!(editor.cursor.index, 11);

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello ");
    assert_eq!(editor.cursor.index, 6);

    // Delete space
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 5);

    // Delete "hello"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_word_at_start() {
    let mut editor = Editor::new();
    for ch in "hello".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 0;

    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_line() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to middle of line2
    editor.cursor.index = 9;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline3");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_line_first() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to first line
    editor.cursor.index = 2;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line2\nline3");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_line_last() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to last line
    editor.cursor.index = 15;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline2\n");
    assert_eq!(editor.cursor.index, 12);
  }

  #[test]
  fn test_delete_line_single() {
    let mut editor = Editor::new();
    for ch in "hello".chars() {
      editor.insert_char(ch);
    }

    editor.cursor.index = 2;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_word_with_punctuation() {
    let mut editor = Editor::new();
    for ch in "hello.world.test".chars() {
      editor.insert_char(ch);
    }
    assert_eq!(editor.buffer.as_str(), "hello.world.test");

    // Delete "test"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello.world.");
    assert_eq!(editor.cursor.index, 12);

    // Delete "."
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello.world");
    assert_eq!(editor.cursor.index, 11);

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello.");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_word_with_spaces() {
    let mut editor = Editor::new();
    for ch in "hello   world".chars() {
      editor.insert_char(ch);
    }
    assert_eq!(editor.buffer.as_str(), "hello   world");

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello   ");
    assert_eq!(editor.cursor.index, 8);

    // Delete the three spaces
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 5);
  }

  #[test]
  fn test_delete_word_multiline() {
    let mut editor = Editor::new();
    for ch in "hello\nworld".chars() {
      editor.insert_char(ch);
    }
    assert_eq!(editor.buffer.as_str(), "hello\nworld");

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello\n");
    assert_eq!(editor.cursor.index, 6);

    // Delete newline
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 5);
  }

  #[test]
  fn test_delete_word_in_middle() {
    let mut editor = Editor::new();
    for ch in "hello world test".chars() {
      editor.insert_char(ch);
    }

    // Position cursor at end of "world"
    editor.cursor.index = 11;

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello  test");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_line_at_start() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to start of first line
    editor.cursor.index = 0;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line2\nline3");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_line_at_end() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to end of last line
    editor.cursor.index = 17;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline2\n");
    assert_eq!(editor.cursor.index, 12);
  }

  #[test]
  fn test_delete_line_empty_line() {
    let mut editor = Editor::new();
    for ch in "line1\n\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to empty line
    editor.cursor.index = 6;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline3");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_line_multiple_times() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3\nline4".chars() {
      editor.insert_char(ch);
    }

    // Delete line 2
    editor.cursor.index = 8;
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline3\nline4");

    // Delete line 1 (now at cursor position 0)
    editor.cursor.index = 0;
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line3\nline4");

    // Delete remaining line
    editor.cursor.index = 0;
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line4");
  }

  #[test]
  fn test_delete_word_underscore() {
    let mut editor = Editor::new();
    for ch in "hello_world_test".chars() {
      editor.insert_char(ch);
    }

    // Delete entire word with underscores (underscores are word chars)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_complex_delete_word_workflow() {
    let mut editor = Editor::new();
    for ch in "hello world test".chars() {
      editor.insert_char(ch);
    }

    // Delete "test"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello world ");

    // Delete " " (space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello world");

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello ");

    // Delete " "
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");

    // Delete "hello"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
  }

  #[test]
  fn test_delete_line_with_leading_spaces() {
    let mut editor = Editor::new();
    for ch in "line1\n  indented line\nline3".chars() {
      editor.insert_char(ch);
    }

    // Move to indented line
    editor.cursor.index = 10;

    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "line1\nline3");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_word_unicode() {
    let mut editor = Editor::new();
    for ch in "hello world".chars() {
      editor.insert_char(ch);
    }

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello ");
    assert_eq!(editor.cursor.index, 6);

    // Delete space
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 5);

    // Delete "hello"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_word_with_emoji() {
    let mut editor = Editor::new();
    for ch in "hello 🌍 world".chars() {
      editor.insert_char(ch);
    }

    // Delete "world"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello 🌍 ");
    assert_eq!(editor.cursor.index, 8);

    // Delete " " (trailing space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello 🌍");
    assert_eq!(editor.cursor.index, 7);

    // Delete "🌍" (emoji as separate segment)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello ");
    assert_eq!(editor.cursor.index, 6);

    // Delete " " (space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor.index, 5);

    // Delete "hello"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }

  #[test]
  fn test_delete_word_stops_at_line_boundary() {
    let mut editor = Editor::new();
    for ch in "line1\nline2\nline3".chars() {
      editor.insert_char(ch);
    }

    // Position: at end of "line3"
    assert_eq!(editor.cursor.index, 17);

    // Delete "line3" - should not cross line boundary
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "line1\nline2\n");
    assert_eq!(editor.cursor.index, 12); // At start of line 3

    // Now at start of line 3, delete_word should delete the newline
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "line1\nline2");
    assert_eq!(editor.cursor.index, 11); // At end of line2

    // Delete "line2" - should not cross into line1
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "line1\n");
    assert_eq!(editor.cursor.index, 6);
  }

  #[test]
  fn test_delete_word_with_emoji_multiline() {
    let mut editor = Editor::new();
    // Create: "z\n🌍\n🌍\n🌍🌍"
    for ch in "z\n🌍\n🌍\n🌍🌍".chars() {
      editor.insert_char(ch);
    }

    // Position: at end (after "🌍🌍")
    // delete_word should delete the two emojis on the current line only
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "z\n🌍\n🌍\n");
    assert_eq!(editor.cursor.index, 6); // At start of last line

    // At start of line, delete_word should delete the newline
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "z\n🌍\n🌍");
    assert_eq!(editor.cursor.index, 5); // At end of previous line

    // Delete emoji on this line
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "z\n🌍\n");
    assert_eq!(editor.cursor.index, 4);
  }

  #[test]
  fn test_delete_word_with_emoji_and_spaces() {
    let mut editor = Editor::new();
    for ch in "word 🌍 🌍 test".chars() {
      editor.insert_char(ch);
    }

    // Delete "test"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word 🌍 🌍 ");

    // Delete " " (trailing space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word 🌍 🌍");

    // Delete "🌍" (second emoji)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word 🌍 ");

    // Delete " " (space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word 🌍");

    // Delete "🌍" (first emoji)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word ");

    // Delete " " (space)
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "word");

    // Delete "word"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
  }

  #[test]
  fn test_delete_word_with_separated_emojis() {
    // Test case from image: emojis separated by spaces should delete one at a time
    let mut editor = Editor::new();
    for ch in "🗿 🗿 🗿".chars() {
      editor.insert_char(ch);
    }

    // At end of line: "🗿 🗿 🗿|"
    assert_eq!(editor.buffer.as_str(), "🗿 🗿 🗿");
    assert_eq!(editor.cursor.index, 5);

    // Delete last emoji "🗿"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "🗿 🗿 ");
    assert_eq!(editor.cursor.index, 4);

    // Delete trailing space " "
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "🗿 🗿");
    assert_eq!(editor.cursor.index, 3);

    // Delete second emoji "🗿"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "🗿 ");
    assert_eq!(editor.cursor.index, 2);

    // Delete space " "
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "🗿");
    assert_eq!(editor.cursor.index, 1);

    // Delete first emoji "🗿"
    editor.delete_word();
    assert_eq!(editor.buffer.as_str(), "");
  }

  #[test]
  fn test_delete_line_with_emoji() {
    // Test that delete_line correctly handles lines with emojis
    let mut editor = Editor::new();
    for ch in "🗿 🗿 🗿 🗿 🗿\ntest line\n🗿 🗿".chars() {
      editor.insert_char(ch);
    }

    // Delete first line with emojis
    editor.cursor.index = 3; // Middle of first line
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "test line\n🗿 🗿");
    assert_eq!(editor.cursor.index, 0);

    // Delete "test line"
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "🗿 🗿");
    assert_eq!(editor.cursor.index, 0);

    // Delete last line with emojis
    editor.delete_line();
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor.index, 0);
  }
}
