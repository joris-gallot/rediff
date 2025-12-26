// Cursor movement and word boundary detection
//
// This module provides cursor navigation functionality including:
// - Basic movement (left, right, up, down)
// - Line-based movement (start/end of line, start/end of buffer)
// - Word-based movement and boundary detection
//
// # Word Boundaries
//
// Word boundaries are defined consistently across the codebase using `is_word_char()`:
// - **Word characters**: alphanumeric (a-z, A-Z, 0-9) and underscore (_)
// - **Non-word characters**: everything else (punctuation, emoji, etc.)
// - **Whitespace**: spaces and tabs (treated as separate segments)
// - **Newlines**: always treated as their own segment
//
// Adjacent word characters are grouped together. Adjacent non-word characters (like
// punctuation or emojis) are grouped together, but **whitespace acts as a separator**.
// This means "🗿 🗿 🗿" is treated as 5 segments: emoji, space, emoji, space, emoji.
//
// The `find_word_boundaries()` function is the shared implementation used by:
// - Double-click word selection in the UI
// - Option+Arrow word navigation
// - Option+Backspace word deletion

use crate::goal::CursorGoal;
use text::TextBuffer;

/// Tracks the desired horizontal position during vertical movement
#[derive(Default, Copy, Clone, Debug, PartialEq)]

pub struct Cursor {
  pub index: usize,
  pub goal: CursorGoal,
}

impl Cursor {
  pub fn new() -> Self {
    Self {
      index: 0,
      goal: CursorGoal::None,
    }
  }

  pub fn move_left(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    }

    self.goal = CursorGoal::None;
  }

  pub fn move_right(&mut self, max: usize) {
    if self.index < max {
      self.index += 1;
    }

    self.goal = CursorGoal::None;
  }

  pub fn move_up(&mut self, buffer: &TextBuffer) {
    let (line, col) = buffer.char_to_line_col(self.index);

    let goal_col = match self.goal {
      CursorGoal::None => col,
      CursorGoal::Column(c) => c,
    };

    if line > 0 {
      let new_line = line - 1;
      let line_len = buffer
        .line(new_line)
        .map(|l| l.trim_end_matches('\n').chars().count())
        .unwrap_or(0);
      let new_col = goal_col.min(line_len);
      self.index = buffer.line_col_to_char(new_line, new_col);
    } else {
      self.index = 0;
    }

    self.goal = CursorGoal::Column(goal_col);
  }

  pub fn move_down(&mut self, buffer: &TextBuffer) {
    let (line, col) = buffer.char_to_line_col(self.index);

    let goal_col = match self.goal {
      CursorGoal::None => col,
      CursorGoal::Column(c) => c,
    };

    if line < buffer.line_count() - 1 {
      let new_line = line + 1;
      let line_len = buffer
        .line(new_line)
        .map(|l| l.trim_end_matches('\n').chars().count())
        .unwrap_or(0);
      let new_col = goal_col.min(line_len);
      self.index = buffer.line_col_to_char(new_line, new_col);
    } else {
      self.index = buffer.len();
    }

    self.goal = CursorGoal::Column(goal_col);
  }

  pub fn move_to_line_start(&mut self, buffer: &TextBuffer) {
    self.goal = CursorGoal::None;
    let (line, _col) = buffer.char_to_line_col(self.index);
    self.index = buffer.line_col_to_char(line, 0);
  }

  pub fn move_to_line_end(&mut self, buffer: &TextBuffer) {
    self.goal = CursorGoal::None;
    let (line, _col) = buffer.char_to_line_col(self.index);
    let line_len = buffer
      .line(line)
      .map(|l| l.trim_end_matches('\n').chars().count())
      .unwrap_or(0);
    self.index = buffer.line_col_to_char(line, line_len);
  }

  pub fn move_to_buffer_start(&mut self) {
    self.index = 0;
    self.goal = CursorGoal::None;
  }

  pub fn move_to_buffer_end(&mut self, buffer: &TextBuffer) {
    self.index = buffer.len();
    self.goal = CursorGoal::None;
  }

  /// Move to previous word boundary (stop at each transition)
  /// Does not move across line boundaries unless at the start of a line
  pub fn move_word_left(&mut self, buffer: &TextBuffer) {
    self.goal = CursorGoal::None;
    if self.index == 0 {
      return;
    }

    let text = buffer.as_str();
    let chars: Vec<char> = text.chars().collect();

    if self.index > chars.len() {
      self.index = chars.len();
      return;
    }

    if chars.is_empty() {
      return;
    }

    // Get current line and column
    let (current_line, current_col) = buffer.char_to_line_col(self.index);
    let line_start = buffer.line_col_to_char(current_line, 0);

    // Find the word boundaries at the position to the left
    let (start, _end) = Self::find_word_boundaries(buffer, self.index - 1);

    // If we're not at the start of a line (col > 0), don't cross line boundaries
    let new_index = if current_col > 0 {
      start.max(line_start)
    } else {
      start
    };

    self.index = new_index;
  }

  /// Find the word boundaries at a given position in the buffer.
  ///
  /// Returns `(start_index, end_index)` of the word segment at the given position.
  ///
  /// Segments are defined as follows:
  /// - **Word characters** (alphanumeric + underscore): grouped together
  /// - **Whitespace** (spaces, tabs): grouped together as separate segments
  /// - **Newlines**: always their own segment
  /// - **Other characters** (punctuation, emoji): grouped together, but separated by whitespace
  ///
  /// This means "🗿 🗿 🗿" is segmented as: `🗿`, ` `, `🗿`, ` `, `🗿`
  ///
  /// # Examples
  ///
  /// // "hello world" at position 2 returns (0, 5) for "hello"
  /// // "hello 🌍 world" at position 6 returns (6, 7) for "🌍"
  /// // "hello 🌍 world" at position 5 returns (5, 6) for " " (space before emoji)
  pub fn find_word_boundaries(buffer: &TextBuffer, position: usize) -> (usize, usize) {
    let text = buffer.as_str();
    let chars: Vec<char> = text.chars().collect();
    let clamped_pos = position.min(chars.len());

    if chars.is_empty() {
      return (0, 0);
    }

    // If we're at the end, step back one
    let start_pos = if clamped_pos == chars.len() && clamped_pos > 0 {
      clamped_pos - 1
    } else {
      clamped_pos
    };

    if start_pos >= chars.len() {
      return (chars.len(), chars.len());
    }

    // Get the character type at current position
    let current_char = chars[start_pos];

    // Special case: if current char is a newline, it's its own segment
    if current_char == '\n' {
      return (start_pos, start_pos + 1);
    }

    // Special case: if current char is whitespace (not newline), it's its own segment
    if current_char.is_whitespace() {
      // Group consecutive whitespace together
      let mut start = start_pos;
      while start > 0 && chars[start - 1].is_whitespace() && chars[start - 1] != '\n' {
        start -= 1;
      }
      let mut end = start_pos + 1;
      while end < chars.len() && chars[end].is_whitespace() && chars[end] != '\n' {
        end += 1;
      }
      return (start, end);
    }

    let current_is_word = Self::is_word_char(current_char);

    // Find start of word (scan backwards)
    let mut start = start_pos;
    while start > 0 {
      let ch = chars[start - 1];
      // Stop at newlines or whitespace
      if ch == '\n' || ch.is_whitespace() {
        break;
      }
      let is_word = Self::is_word_char(ch);
      if is_word != current_is_word {
        break;
      }
      start -= 1;
    }

    // Find end of word (scan forwards)
    let mut end = start_pos;
    while end < chars.len() {
      let ch = chars[end];
      // Stop at newlines or whitespace
      if ch == '\n' || ch.is_whitespace() {
        break;
      }
      let is_word = Self::is_word_char(ch);
      if is_word != current_is_word {
        break;
      }
      end += 1;
    }

    (start, end)
  }

  /// Move to next word boundary (stop at each transition)
  /// Does not move across line boundaries
  pub fn move_word_right(&mut self, buffer: &TextBuffer) {
    self.goal = CursorGoal::None;
    let text = buffer.as_str();
    let chars: Vec<char> = text.chars().collect();
    let text_len = chars.len();

    if self.index >= text_len {
      return;
    }

    // Get current line
    let (current_line, _) = buffer.char_to_line_col(self.index);
    let line_end_index = if current_line + 1 < buffer.line_count() {
      buffer.line_col_to_char(current_line + 1, 0)
    } else {
      text_len
    };

    // Find the word boundaries at the current position
    let (_start, end) = Self::find_word_boundaries(buffer, self.index);

    // Don't cross line boundaries
    let new_index = end.min(line_end_index);

    self.index = new_index;
  }

  /// Determines if a character is a word character.
  ///
  /// Word characters: alphanumeric (a-z, A-Z, 0-9) and underscore (_)
  /// Non-word characters: everything else (spaces, punctuation, newlines, etc.)
  ///
  /// This can be easily modified to change word boundary behavior.
  pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_cursor() {
    let cursor = Cursor::new();
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_move_left() {
    let mut cursor = Cursor::new();
    cursor.index = 5;

    cursor.move_left();
    assert_eq!(cursor.index, 4);

    cursor.move_left();
    assert_eq!(cursor.index, 3);
  }

  #[test]
  fn test_move_left_at_start() {
    let mut cursor = Cursor::new();
    cursor.index = 0;

    cursor.move_left();
    assert_eq!(cursor.index, 0); // Should stay at 0
  }

  #[test]
  fn test_move_right() {
    let mut cursor = Cursor::new();

    cursor.move_right(10);
    assert_eq!(cursor.index, 1);

    cursor.move_right(10);
    assert_eq!(cursor.index, 2);
  }

  #[test]
  fn test_move_right_at_end() {
    let mut cursor = Cursor::new();
    cursor.index = 5;

    cursor.move_right(5);
    assert_eq!(cursor.index, 5); // Should not go beyond max
  }

  #[test]
  fn test_move_up() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Line 1\nLine 2\nLine 3");

    let mut cursor = Cursor::new();
    cursor.index = 10; // Middle of "Line 2"

    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 3); // Same column in "Line 1"
  }

  #[test]
  fn test_move_up_at_first_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Line 1\nLine 2");

    let mut cursor = Cursor::new();
    cursor.index = 3; // In first line

    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 0); // Should go to start
  }

  #[test]
  fn test_move_up_shorter_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hi\nLonger line");

    let mut cursor = Cursor::new();
    cursor.index = 10; // Near end of "Longer line"

    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 2); // Should clamp to end of "Hi" (before \n)
  }

  #[test]
  fn test_move_down() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Line 1\nLine 2\nLine 3");

    let mut cursor = Cursor::new();
    cursor.index = 3; // Middle of "Line 1"

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 10); // Same column in "Line 2"
  }

  #[test]
  fn test_move_down_at_last_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Line 1\nLine 2");

    let mut cursor = Cursor::new();
    cursor.index = 10; // In last line

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, buffer.len()); // Should go to end
  }

  #[test]
  fn test_move_down_shorter_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Longer line\nHi");

    let mut cursor = Cursor::new();
    cursor.index = 8; // Near end of "Longer line"

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 14); // Should clamp to end of "Hi"
  }

  #[test]
  fn test_vertical_movement_preserves_column() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "AAAA\nBBBB\nCCCC");

    let mut cursor = Cursor::new();
    cursor.index = 2; // Column 2 in first line

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 7); // Column 2 in second line

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 12); // Column 2 in third line

    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 7); // Back to column 2 in second line

    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 2); // Back to column 2 in first line
  }

  #[test]
  fn test_move_to_line_start() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();
    cursor.index = 5;

    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_move_to_line_start_multiline() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\nline2\nline3");
    let mut cursor = Cursor::new();
    cursor.index = 14; // middle of line3

    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.index, 12); // start of line3
  }

  #[test]
  fn test_move_to_line_end() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();
    cursor.index = 5;

    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.index, 11);
  }

  #[test]
  fn test_move_to_line_end_multiline() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\nline2\nline3");
    let mut cursor = Cursor::new();
    cursor.index = 8; // middle of line2

    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.index, 11); // end of line2 (before \n)
  }

  #[test]
  fn test_move_to_line_end_excludes_newline() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld");
    let mut cursor = Cursor::new();
    cursor.index = 2; // in "hello"

    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.index, 5); // before \n, not at 6 (which is \n)
  }

  #[test]
  fn test_move_to_buffer_start() {
    let mut cursor = Cursor::new();
    cursor.index = 100;
    cursor.move_to_buffer_start();
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_move_to_buffer_end() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\ntest");
    let mut cursor = Cursor::new();
    cursor.index = 5;
    cursor.move_to_buffer_end(&buffer);
    assert_eq!(cursor.index, buffer.len());
    assert_eq!(cursor.goal, CursorGoal::None);
  }

  #[test]
  fn test_move_to_line_start_already_at_start() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();

    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_move_to_line_end_already_at_end() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();
    cursor.index = 11;

    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.index, 11);
  }

  #[test]
  fn test_move_to_line_start_empty_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\n\nline3");
    let mut cursor = Cursor::new();
    cursor.index = 6; // on empty line

    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.index, 6); // stays at start of empty line
  }

  #[test]
  fn test_move_to_line_end_empty_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\n\nline3");
    let mut cursor = Cursor::new();
    cursor.index = 6; // on empty line

    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.index, 6); // stays at same position (line is empty)
  }

  #[test]
  fn test_move_word_right_simple() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();

    // From start of "hello" to end of "hello"
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5);

    // From end of "hello" (space) to end of space
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 6);

    // From start of "world" to end of "world"
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 11);
  }

  #[test]
  fn test_move_word_left_simple() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();
    cursor.index = 11; // End of "world"

    // From end of "world" to start of "world"
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 6);

    // From start of "world" (was space) to start of space
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 5);

    // From end of "hello" to start of "hello"
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_move_word_right_with_punctuation() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello.world");
    let mut cursor = Cursor::new();

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5); // End of "hello"

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 6); // End of "."

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 11); // End of "world"
  }

  #[test]
  fn test_move_word_right_multiple_spaces() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello   world");
    let mut cursor = Cursor::new();

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5); // End of "hello"

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 8); // End of "   " (all spaces are one segment)

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 13); // End of "world"
  }

  #[test]
  fn test_word_movement_example() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Word Movement Examples");
    let mut cursor = Cursor::new();

    // Position 0 -> 4 (end of "Word")
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 4);

    // Position 4 -> 5 (end of space)
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5);

    // Position 5 -> 13 (end of "Movement")
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 13);

    // Now go back
    // Position 13 -> 5 (start of "Movement")
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 5);

    // Position 5 -> 4 (start of space)
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 4);

    // Position 4 -> 0 (start of "Word")
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_is_word_char() {
    assert!(Cursor::is_word_char('a'));
    assert!(Cursor::is_word_char('Z'));
    assert!(Cursor::is_word_char('0'));
    assert!(Cursor::is_word_char('_'));

    assert!(!Cursor::is_word_char(' '));
    assert!(!Cursor::is_word_char('.'));
    assert!(!Cursor::is_word_char('-'));
    assert!(!Cursor::is_word_char('\n'));
  }

  #[test]
  fn test_move_word_boundaries_underscore() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "foo_bar");
    let mut cursor = Cursor::new();

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 7); // "foo_bar" is one word (underscore is word char)
  }

  #[test]
  fn test_move_word_at_boundaries() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");
    let mut cursor = Cursor::new();

    // At start
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0); // Stay at start

    // At end
    cursor.index = 11;
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 11); // Stay at end
  }

  #[test]
  fn test_move_word_with_newlines() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld");
    let mut cursor = Cursor::new();

    // 0 -> 5 (end of "hello")
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5);

    // 5 -> 6 (end of "\n")
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 6);

    // 6 -> 11 (end of "world")
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 11);

    // Now go back
    // 11 -> 6 (start of "world")
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 6);

    // 6 -> 5 (start of "\n")
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 5);

    // 5 -> 0 (start of "hello")
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0);
  }

  #[test]
  fn test_find_word_boundaries_simple() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world test");

    // In middle of "hello"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 2);
    assert_eq!(start, 0);
    assert_eq!(end, 5);

    // At start of "world"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 6);
    assert_eq!(start, 6);
    assert_eq!(end, 11);

    // In middle of "world"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 8);
    assert_eq!(start, 6);
    assert_eq!(end, 11);

    // At end of buffer
    let (start, end) = Cursor::find_word_boundaries(&buffer, 16);
    assert_eq!(start, 12);
    assert_eq!(end, 16);
  }

  #[test]
  fn test_find_word_boundaries_with_punctuation() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello.world");

    // On "hello"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 2);
    assert_eq!(start, 0);
    assert_eq!(end, 5);

    // On the dot
    let (start, end) = Cursor::find_word_boundaries(&buffer, 5);
    assert_eq!(start, 5);
    assert_eq!(end, 6);

    // On "world"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 8);
    assert_eq!(start, 6);
    assert_eq!(end, 11);
  }

  #[test]
  fn test_find_word_boundaries_with_spaces() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello   world");

    // On "hello"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 2);
    assert_eq!(start, 0);
    assert_eq!(end, 5);

    // On first space
    let (start, end) = Cursor::find_word_boundaries(&buffer, 5);
    assert_eq!(start, 5);
    assert_eq!(end, 8); // All spaces grouped together

    // On "world"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 8);
    assert_eq!(start, 8);
    assert_eq!(end, 13);
  }

  #[test]
  fn test_find_word_boundaries_with_emoji() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello 🌍 world");

    // On "hello"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 2);
    assert_eq!(start, 0);
    assert_eq!(end, 5);

    // On space before emoji - now whitespace is its own segment
    let (start, end) = Cursor::find_word_boundaries(&buffer, 5);
    assert_eq!(start, 5);
    assert_eq!(end, 6); // Just the space

    // On emoji (emoji is not a word char, but separate from whitespace)
    let (start, end) = Cursor::find_word_boundaries(&buffer, 6);
    assert_eq!(start, 6);
    assert_eq!(end, 7); // Just the emoji

    // On space after emoji
    let (start, end) = Cursor::find_word_boundaries(&buffer, 7);
    assert_eq!(start, 7);
    assert_eq!(end, 8); // Just the space

    // On "world"
    let (start, end) = Cursor::find_word_boundaries(&buffer, 8);
    assert_eq!(start, 8);
    assert_eq!(end, 13);
  }

  #[test]
  fn test_find_word_boundaries_at_edges() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "word");

    // At start
    let (start, end) = Cursor::find_word_boundaries(&buffer, 0);
    assert_eq!(start, 0);
    assert_eq!(end, 4);

    // At end
    let (start, end) = Cursor::find_word_boundaries(&buffer, 4);
    assert_eq!(start, 0);
    assert_eq!(end, 4);
  }

  #[test]
  fn test_find_word_boundaries_empty_buffer() {
    let buffer = TextBuffer::new();
    let (start, end) = Cursor::find_word_boundaries(&buffer, 0);
    assert_eq!(start, 0);
    assert_eq!(end, 0);
  }

  #[test]
  fn test_move_word_left_stops_at_line_boundary() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\nline2\nline3");
    let mut cursor = Cursor::new();
    cursor.index = 17; // End of "line3"

    // Move word left should stop at "line" on same line
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 12); // Start of "line3"

    // Now at start of line3, move_word_left should delete the newline
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 11); // On the newline at end of line2

    // Move left again from middle of line2
    cursor.index = 9; // In "line2"
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 6); // Start of "line2", not crossing to line1
  }

  #[test]
  fn test_move_word_right_stops_at_line_boundary() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line1\nline2\nline3");
    let mut cursor = Cursor::new();
    // Start of "line1"

    // Move word right
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5); // At the newline after "line1"

    // From newline, move right goes to next line
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 6); // Start of "line2"

    // From middle of line2
    cursor.index = 8; // In "line2"
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 11); // End of "line2", not crossing to line3
  }

  #[test]
  fn test_move_word_with_emoji_stops_at_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "word\n🌍🌍\ntest");
    let mut cursor = Cursor::new();
    cursor.index = 7; // After emoji on line 2

    // Move left should stop at start of line, not cross to "word"
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 5); // Start of line 2 (after newline)
  }

  #[test]
  fn test_cursor_goal_preserves_column() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\nhi\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 8; // column 8 on line 1 ("hello world")

    // Move down to shorter line "hi"
    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 14); // end of "hi" (column 2)
    assert_eq!(cursor.goal, CursorGoal::Column(8)); // goal is preserved

    // Move down again to "hello again" - should return to column 8
    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 23); // column 8 of "hello again"
    assert_eq!(cursor.goal, CursorGoal::Column(8));
  }

  #[test]
  fn test_cursor_goal_resets_on_horizontal_movement() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\nhi\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 8;

    // Move down to establish a goal
    cursor.move_down(&buffer);
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Move left should reset goal
    cursor.move_left();
    assert_eq!(cursor.goal, CursorGoal::None);

    // Move down again - should use current column, not old goal
    let (_line, col) = buffer.char_to_line_col(cursor.index);
    cursor.move_down(&buffer);
    let new_goal = match cursor.goal {
      CursorGoal::Column(c) => c,
      CursorGoal::None => 0,
    };
    assert_eq!(new_goal, col);
  }

  #[test]
  fn test_cursor_goal_up_then_down() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\nhi\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 23; // column 8 on line 3 ("hello again")

    // Move up to shorter line "hi"
    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 14); // end of "hi" (column 2)
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Move up again to "hello world" - should return to column 8
    cursor.move_up(&buffer);
    assert_eq!(cursor.index, 8); // column 8 of "hello world"
    assert_eq!(cursor.goal, CursorGoal::Column(8));
  }

  #[test]
  fn test_cursor_goal_multiple_short_lines() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\na\nb\nc\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 8; // column 8 on line 1

    // Move down through multiple short lines
    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 13); // end of "a"
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 15); // end of "b"
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 17); // end of "c"
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Finally reach a long line - should return to column 8
    cursor.move_down(&buffer);
    assert_eq!(cursor.index, 26); // column 8 of "hello again"
    assert_eq!(cursor.goal, CursorGoal::Column(8));
  }

  #[test]
  fn test_cursor_goal_resets_on_line_start_end() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\nhi\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 8;

    // Establish a goal
    cursor.move_down(&buffer);
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Move to line start should reset goal
    cursor.move_to_line_start(&buffer);
    assert_eq!(cursor.goal, CursorGoal::None);

    // Establish goal again
    cursor.index = 8;
    cursor.move_down(&buffer);
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Move to line end should reset goal
    cursor.move_to_line_end(&buffer);
    assert_eq!(cursor.goal, CursorGoal::None);
  }

  #[test]
  fn test_cursor_goal_resets_on_word_movement() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world\nhi\nhello again");
    let mut cursor = Cursor::new();
    cursor.index = 8;

    // Establish a goal
    cursor.move_down(&buffer);
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Word movement should reset goal
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.goal, CursorGoal::None);

    // Establish goal again
    cursor.index = 8;
    cursor.move_down(&buffer);
    assert_eq!(cursor.goal, CursorGoal::Column(8));

    // Word movement right should also reset goal
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.goal, CursorGoal::None);
  }

  #[test]
  fn test_move_word_with_separated_emojis() {
    // Test case from image: emojis separated by spaces should navigate one at a time
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "🗿 🗿 🗿");

    // Start at end: "🗿 🗿 🗿|"
    let mut cursor = Cursor::new();
    cursor.index = 5;

    // Move left to third emoji
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 4); // Start of "🗿"

    // Move left to space before third emoji
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 3); // Start of " "

    // Move left to second emoji
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 2); // Start of "🗿"

    // Move left to space before second emoji
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 1); // Start of " "

    // Move left to first emoji
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0); // Start of "🗿"

    // Now test moving right from start
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 1); // End of first "🗿"

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 2); // End of first " "

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 3); // End of second "🗿"

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 4); // End of second " "

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5); // End of third "🗿"
  }

  #[test]
  fn test_comprehensive_word_operations_with_mixed_content() {
    // Comprehensive test: verify all word operations work consistently with mixed content
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello 🗿 world\ntest 123 emoji");

    let mut cursor = Cursor::new();

    // Test word boundaries on mixed line
    let (start, end) = Cursor::find_word_boundaries(&buffer, 0);
    assert_eq!((start, end), (0, 5)); // "hello"

    let (start, end) = Cursor::find_word_boundaries(&buffer, 5);
    assert_eq!((start, end), (5, 6)); // " " (space)

    let (start, end) = Cursor::find_word_boundaries(&buffer, 6);
    assert_eq!((start, end), (6, 7)); // "🗿"

    let (start, end) = Cursor::find_word_boundaries(&buffer, 7);
    assert_eq!((start, end), (7, 8)); // " " (space)

    let (start, end) = Cursor::find_word_boundaries(&buffer, 8);
    assert_eq!((start, end), (8, 13)); // "world"

    // Test navigation from start to end
    cursor.index = 0;
    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 5); // End of "hello"

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 6); // End of space

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 7); // End of emoji

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 8); // End of space

    cursor.move_word_right(&buffer);
    assert_eq!(cursor.index, 13); // End of "world"

    // Test navigation backward
    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 8); // Start of "world"

    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 7); // Start of space

    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 6); // Start of emoji

    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 5); // Start of space

    cursor.move_word_left(&buffer);
    assert_eq!(cursor.index, 0); // Start of "hello"

    // Test that is_word_char is consistent
    assert!(Cursor::is_word_char('h'));
    assert!(Cursor::is_word_char('1'));
    assert!(Cursor::is_word_char('_'));
    assert!(!Cursor::is_word_char(' '));
    assert!(!Cursor::is_word_char('🗿'));
  }

  #[test]
  fn test_move_to_line_end_with_emoji() {
    // Test case from bug report: emojis on line should move to end of line, not next line
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "🗿 🗿 🗿 🗿 🗿      🗿\ntest 1");

    let mut cursor = Cursor::new();
    // Start at beginning of line 1
    cursor.index = 0;

    // Move to end of line should stop at end of line 1 (before newline)
    cursor.move_to_line_end(&buffer);

    let line_text = buffer.line(0).unwrap();
    let expected_pos = line_text.trim_end_matches('\n').chars().count();

    assert_eq!(
      cursor.index, expected_pos,
      "Cursor should be at end of line with emojis (position {}), but was at position {}",
      expected_pos, cursor.index
    );

    // Verify we're on line 0, not line 1
    let (line, _) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 0, "Cursor should still be on line 0");
  }

  #[test]
  fn test_move_up_with_emoji() {
    // Test that move_up correctly handles lines with emojis
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "🗿 🗿 🗿\ntest line");

    let mut cursor = Cursor::new();
    // Start at beginning of line 2 (after newline)
    cursor.index = 6; // Position after "🗿 🗿 🗿\n"

    // Move up should go to line 1 at same column
    cursor.move_up(&buffer);

    // Should be on line 0
    let (line, col) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 0, "Should be on line 0 after moving up");
    assert_eq!(col, 0, "Should preserve column position");
  }

  #[test]
  fn test_move_down_with_emoji() {
    // Test that move_down correctly handles lines with emojis
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "test line\n🗿 🗿 🗿");

    let mut cursor = Cursor::new();
    // Start at beginning of line 1
    cursor.index = 0;

    // Move down should go to line 2
    cursor.move_down(&buffer);

    // Should be on line 1
    let (line, col) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 1, "Should be on line 1 after moving down");
    assert_eq!(col, 0, "Should be at start of line");
  }

  #[test]
  fn test_move_up_down_with_emoji_column_preservation() {
    // Test that moving up/down preserves column with emojis
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "🗿 🗿 🗿 🗿\ntest\n🗿 🗿 🗿");

    let mut cursor = Cursor::new();
    // Start at position 2 on line 1 (third emoji position)
    cursor.index = 2;

    // Move down to line 2 (shorter line)
    cursor.move_down(&buffer);
    let (line, col) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 1);
    assert_eq!(col, 2, "Should preserve column 2");

    // Move down to line 3 with emojis
    cursor.move_down(&buffer);
    let (line, col) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 2);
    assert_eq!(col, 2, "Should preserve column 2 on emoji line");

    // Move back up
    cursor.move_up(&buffer);
    let (line, _col) = buffer.char_to_line_col(cursor.index);
    assert_eq!(line, 1);
  }
}
