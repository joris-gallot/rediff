use cursor::Cursor;
use std::ops::Range;
use text::TextBuffer;

/// Represents a text selection with start and end positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
  pub start: usize,
  pub end: usize,
  pub reversed: bool, // True if selection was made backwards (right to left)
}

impl Selection {
  /// Create a new selection from start to end
  pub fn new(start: usize, end: usize) -> Self {
    Self {
      start: start.min(end),
      end: start.max(end),
      reversed: start > end,
    }
  }

  /// Get the selection as a Range
  pub fn range(&self) -> Range<usize> {
    self.start..self.end
  }

  /// Check if selection is empty (start == end)
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }

  /// Get the "head" (moving end) of the selection
  pub fn head(&self) -> usize {
    if self.reversed { self.start } else { self.end }
  }

  /// Get the "tail" (anchor/fixed end) of the selection
  pub fn tail(&self) -> usize {
    if self.reversed { self.end } else { self.start }
  }
}

#[derive(Default)]
pub struct Editor {
  pub buffer: TextBuffer,
  pub cursor: Cursor,
  pub selection: Option<Selection>,
}

impl Editor {
  pub fn new() -> Self {
    Self {
      buffer: TextBuffer::new(),
      cursor: Cursor::new(),
      selection: None,
    }
  }

  /// Check if there's an active selection
  pub fn has_selection(&self) -> bool {
    self.selection.is_some()
  }

  /// Get the current selection range
  pub fn selection_range(&self) -> Option<Range<usize>> {
    self.selection.as_ref().map(|s| s.range())
  }

  /// Set selection from start to end
  pub fn select_range(&mut self, start: usize, end: usize) {
    self.selection = Some(Selection::new(start, end));
  }

  /// Select all text in buffer
  pub fn select_all(&mut self) {
    self.selection = Some(Selection::new(0, self.buffer.len()));
  }

  /// Clear the current selection
  pub fn clear_selection(&mut self) {
    self.selection = None;
  }

  /// Delete the selected text and return it
  pub fn delete_selection(&mut self) -> Option<String> {
    if let Some(range) = self.selection_range() {
      let text = self.get_selected_text();
      let len = range.end - range.start;
      self.buffer.delete(range.start, len);
      self.cursor.index = range.start;
      self.clear_selection();
      text
    } else {
      None
    }
  }

  /// Get the currently selected text
  pub fn get_selected_text(&self) -> Option<String> {
    if let Some(range) = self.selection_range() {
      let text = self.buffer.as_str();
      let selected: String = text
        .chars()
        .skip(range.start)
        .take(range.end - range.start)
        .collect();
      Some(selected)
    } else {
      None
    }
  }

  /// Replace the selected text with new content
  pub fn replace_selection(&mut self, replacement: &str) {
    if self.selection_range().is_some() {
      self.delete_selection();
    }
    for ch in replacement.chars() {
      self.insert_char(ch);
    }
  }

  /// Select word at the given index
  pub fn select_word_at(&mut self, index: usize) {
    let (start, end) = Cursor::find_word_boundaries(&self.buffer, index);
    self.select_range(start, end);
  }

  /// Select entire line at the given index
  pub fn select_line_at(&mut self, index: usize) {
    let (line, _col) = self.buffer.char_to_line_col(index);
    let start = self.buffer.line_col_to_char(line, 0);
    let end = if line + 1 < self.buffer.line_count() {
      self.buffer.line_col_to_char(line + 1, 0)
    } else {
      self.buffer.len()
    };
    self.select_range(start, end);
  }

  /// Extend selection left by one character
  pub fn extend_selection_left(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_left();
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection right by one character
  pub fn extend_selection_right(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_right(self.buffer.len());
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection up by one line
  pub fn extend_selection_up(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_up(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection down by one line
  pub fn extend_selection_down(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_down(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection to start of current line
  pub fn extend_selection_to_line_start(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_to_line_start(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection to end of current line
  pub fn extend_selection_to_line_end(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_to_line_end(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection to start of buffer
  pub fn extend_selection_to_buffer_start(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_to_buffer_start();
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection to end of buffer
  pub fn extend_selection_to_buffer_end(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_to_buffer_end(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection left by one word
  pub fn extend_selection_word_left(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_word_left(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Extend selection right by one word
  pub fn extend_selection_word_right(&mut self) {
    if self.selection.is_none() {
      self.selection = Some(Selection::new(self.cursor.index, self.cursor.index));
    }
    self.cursor.move_word_right(&self.buffer);
    if let Some(sel) = &mut self.selection {
      *sel = Selection::new(sel.tail(), self.cursor.index);
    }
  }

  /// Copy selected text (returns text for clipboard)
  pub fn copy(&self) -> Option<String> {
    self.get_selected_text()
  }

  /// Cut selected text (copy + delete, returns text for clipboard)
  pub fn cut(&mut self) -> Option<String> {
    self.delete_selection()
  }

  /// Paste text at cursor (or replace selection)
  pub fn paste(&mut self, text: &str) {
    if self.has_selection() {
      self.delete_selection();
    }
    for ch in text.chars() {
      self.insert_char(ch);
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

  #[test]
  fn test_delete_selection() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(0, 5); // Select "Hello"
    let deleted = editor.delete_selection();
    assert_eq!(deleted, Some("Hello".to_string()));
    assert_eq!(editor.buffer.as_str(), " World");
    assert_eq!(editor.cursor.index, 0);
    assert!(!editor.has_selection());
  }

  #[test]
  fn test_delete_selection_none() {
    let mut editor = Editor::new();
    for ch in "Hello".chars() {
      editor.insert_char(ch);
    }

    let deleted = editor.delete_selection();
    assert_eq!(deleted, None);
    assert_eq!(editor.buffer.as_str(), "Hello");
  }

  #[test]
  fn test_get_selected_text() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(6, 11); // Select "World"
    assert_eq!(editor.get_selected_text(), Some("World".to_string()));
  }

  #[test]
  fn test_get_selected_text_none() {
    let mut editor = Editor::new();
    for ch in "Hello".chars() {
      editor.insert_char(ch);
    }

    assert_eq!(editor.get_selected_text(), None);
  }

  #[test]
  fn test_replace_selection() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(6, 11); // Select "World"
    editor.replace_selection("Rust");
    assert_eq!(editor.buffer.as_str(), "Hello Rust");
    assert!(!editor.has_selection());
  }

  #[test]
  fn test_select_word_at() {
    let mut editor = Editor::new();
    for ch in "Hello World Test".chars() {
      editor.insert_char(ch);
    }

    editor.select_word_at(7); // Middle of "World"
    assert_eq!(editor.selection_range(), Some(6..11));
    assert_eq!(editor.get_selected_text(), Some("World".to_string()));
  }

  #[test]
  fn test_select_line_at() {
    let mut editor = Editor::new();
    for ch in "Line 1\nLine 2\nLine 3".chars() {
      editor.insert_char(ch);
    }

    editor.select_line_at(10); // In "Line 2"
    let selected = editor.get_selected_text();
    assert_eq!(selected, Some("Line 2\n".to_string()));
  }

  #[test]
  fn test_select_line_at_last_line() {
    let mut editor = Editor::new();
    for ch in "Line 1\nLine 2".chars() {
      editor.insert_char(ch);
    }

    editor.select_line_at(10); // In "Line 2" (last line)
    let selected = editor.get_selected_text();
    assert_eq!(selected, Some("Line 2".to_string()));
  }

  #[test]
  fn test_copy() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(0, 5); // Select "Hello"
    let copied = editor.copy();
    assert_eq!(copied, Some("Hello".to_string()));
    assert_eq!(editor.buffer.as_str(), "Hello World"); // Original unchanged
    assert!(editor.has_selection()); // Selection preserved
  }

  #[test]
  fn test_copy_none() {
    let editor = Editor::new();
    assert_eq!(editor.copy(), None);
  }

  #[test]
  fn test_cut() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(0, 5); // Select "Hello"
    let cut = editor.cut();
    assert_eq!(cut, Some("Hello".to_string()));
    assert_eq!(editor.buffer.as_str(), " World");
    assert!(!editor.has_selection());
  }

  #[test]
  fn test_cut_none() {
    let mut editor = Editor::new();
    assert_eq!(editor.cut(), None);
  }

  #[test]
  fn test_paste() {
    let mut editor = Editor::new();
    for ch in "Hello".chars() {
      editor.insert_char(ch);
    }

    editor.cursor.index = 5;
    editor.paste(" World");
    assert_eq!(editor.buffer.as_str(), "Hello World");
  }

  #[test]
  fn test_paste_replace_selection() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_range(6, 11); // Select "World"
    editor.paste("Rust");
    assert_eq!(editor.buffer.as_str(), "Hello Rust");
    assert!(!editor.has_selection());
  }

  #[test]
  fn test_extend_selection_right() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 0;

    editor.extend_selection_right();
    assert_eq!(editor.selection_range(), Some(0..1));

    editor.extend_selection_right();
    assert_eq!(editor.selection_range(), Some(0..2));
  }

  #[test]
  fn test_extend_selection_left() {
    let mut editor = Editor::new();
    for ch in "Hello".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 5;

    editor.extend_selection_left();
    assert_eq!(editor.selection_range(), Some(4..5));

    editor.extend_selection_left();
    assert_eq!(editor.selection_range(), Some(3..5));
  }

  #[test]
  fn test_extend_selection_multi_line() {
    let mut editor = Editor::new();
    for ch in "Line 1\nLine 2\nLine 3".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 7; // Start of "Line 2"

    editor.extend_selection_down();
    assert_eq!(editor.selection_range(), Some(7..14)); // To start of "Line 3"
  }

  #[test]
  fn test_extend_selection_to_line_end() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 0;

    editor.extend_selection_to_line_end();
    assert_eq!(editor.selection_range(), Some(0..11));
  }

  #[test]
  fn test_extend_selection_word_right() {
    let mut editor = Editor::new();
    for ch in "Hello World Test".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 0;

    editor.extend_selection_word_right();
    assert_eq!(editor.selection_range(), Some(0..5)); // "Hello"
  }

  #[test]
  fn test_extend_selection_preserves_anchor() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }
    editor.cursor.index = 5;

    editor.extend_selection_right();
    editor.extend_selection_right();
    assert_eq!(editor.selection_range(), Some(5..7));

    // Now extend left - should contract
    editor.extend_selection_left();
    assert_eq!(editor.selection_range(), Some(5..6));
  }

  #[test]
  fn test_selection_new() {
    let sel = Selection::new(5, 10);
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
    assert!(!sel.reversed);
  }

  #[test]
  fn test_selection_new_reversed() {
    let sel = Selection::new(10, 5);
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
    assert!(sel.reversed);
  }

  #[test]
  fn test_selection_range() {
    let sel = Selection::new(5, 10);
    let range = sel.range();
    assert_eq!(range.start, 5);
    assert_eq!(range.end, 10);
  }

  #[test]
  fn test_selection_is_empty() {
    let sel1 = Selection::new(5, 5);
    assert!(sel1.is_empty());

    let sel2 = Selection::new(5, 10);
    assert!(!sel2.is_empty());
  }

  #[test]
  fn test_selection_head_tail_forward() {
    let sel = Selection::new(5, 10);
    assert_eq!(sel.head(), 10); // Moving end
    assert_eq!(sel.tail(), 5); // Anchor
  }

  #[test]
  fn test_selection_head_tail_reversed() {
    let sel = Selection::new(10, 5);
    assert_eq!(sel.head(), 5); // Moving end (at start because reversed)
    assert_eq!(sel.tail(), 10); // Anchor (at end because reversed)
  }

  #[test]
  fn test_editor_has_selection() {
    let mut editor = Editor::new();
    assert!(!editor.has_selection());

    editor.select_range(0, 5);
    assert!(editor.has_selection());

    editor.clear_selection();
    assert!(!editor.has_selection());
  }

  #[test]
  fn test_editor_selection_range() {
    let mut editor = Editor::new();
    assert_eq!(editor.selection_range(), None);

    editor.select_range(5, 10);
    assert_eq!(editor.selection_range(), Some(5..10));
  }

  #[test]
  fn test_editor_select_all() {
    let mut editor = Editor::new();
    for ch in "Hello World".chars() {
      editor.insert_char(ch);
    }

    editor.select_all();
    assert!(editor.has_selection());
    assert_eq!(editor.selection_range(), Some(0..11));
  }

  #[test]
  fn test_editor_clear_selection() {
    let mut editor = Editor::new();
    editor.select_range(0, 5);
    assert!(editor.has_selection());

    editor.clear_selection();
    assert!(!editor.has_selection());
    assert_eq!(editor.selection_range(), None);
  }
}
