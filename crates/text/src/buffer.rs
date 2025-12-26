use ropey::Rope;

#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
  rope: Rope,
}

impl TextBuffer {
  pub fn new() -> Self {
    Self { rope: Rope::new() }
  }

  pub fn insert(&mut self, index: usize, content: &str) {
    self.rope.insert(index, content);
  }

  pub fn delete(&mut self, index: usize, len: usize) {
    let end = (index + len).min(self.rope.len_chars());
    self.rope.remove(index..end);
  }

  pub fn as_str(&self) -> String {
    self.rope.to_string()
  }

  pub fn len(&self) -> usize {
    self.rope.len_chars()
  }

  pub fn is_empty(&self) -> bool {
    self.rope.len_chars() == 0
  }

  pub fn line_count(&self) -> usize {
    self.rope.len_lines()
  }

  pub fn line(&self, line_idx: usize) -> Option<String> {
    if line_idx < self.rope.len_lines() {
      Some(self.rope.line(line_idx).to_string())
    } else {
      None
    }
  }

  pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
    let char_idx = char_idx.min(self.rope.len_chars());
    let line = self.rope.char_to_line(char_idx);
    let line_start = self.rope.line_to_char(line);
    let col = char_idx - line_start;
    (line, col)
  }

  pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
    if line >= self.rope.len_lines() {
      return self.rope.len_chars();
    }
    let line_start = self.rope.line_to_char(line);
    let line_end = if line + 1 < self.rope.len_lines() {
      self.rope.line_to_char(line + 1)
    } else {
      self.rope.len_chars()
    };
    (line_start + col).min(line_end)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_buffer() {
    let buffer = TextBuffer::new();
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.as_str(), "");
    assert_eq!(buffer.line_count(), 1); // Empty buffer has 1 line
  }

  #[test]
  fn test_insert() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello");
    assert_eq!(buffer.as_str(), "Hello");
    assert_eq!(buffer.len(), 5);

    buffer.insert(5, " World");
    assert_eq!(buffer.as_str(), "Hello World");
    assert_eq!(buffer.len(), 11);

    buffer.insert(5, ",");
    assert_eq!(buffer.as_str(), "Hello, World");
    assert_eq!(buffer.len(), 12);
  }

  #[test]
  fn test_delete() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello World");

    buffer.delete(5, 6); // Delete " World"
    assert_eq!(buffer.as_str(), "Hello");
    assert_eq!(buffer.len(), 5);

    buffer.delete(0, 2); // Delete "He"
    assert_eq!(buffer.as_str(), "llo");
    assert_eq!(buffer.len(), 3);
  }

  #[test]
  fn test_delete_out_of_bounds() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello");

    // Try to delete beyond buffer length - should not panic
    buffer.delete(3, 100);
    assert_eq!(buffer.as_str(), "Hel");
  }

  #[test]
  fn test_line_count() {
    let mut buffer = TextBuffer::new();
    assert_eq!(buffer.line_count(), 1);

    buffer.insert(0, "Line 1\n");
    assert_eq!(buffer.line_count(), 2);

    buffer.insert(7, "Line 2\n");
    assert_eq!(buffer.line_count(), 3);

    buffer.insert(14, "Line 3");
    assert_eq!(buffer.line_count(), 3);
  }

  #[test]
  fn test_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "First\nSecond\nThird");

    assert_eq!(buffer.line(0), Some("First\n".to_string()));
    assert_eq!(buffer.line(1), Some("Second\n".to_string()));
    assert_eq!(buffer.line(2), Some("Third".to_string()));
    assert_eq!(buffer.line(3), None);
  }

  #[test]
  fn test_char_to_line_col() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello\nWorld\nTest");

    // "Hello\n" = 6 chars (0-5)
    assert_eq!(buffer.char_to_line_col(0), (0, 0)); // 'H'
    assert_eq!(buffer.char_to_line_col(4), (0, 4)); // 'o'
    assert_eq!(buffer.char_to_line_col(5), (0, 5)); // '\n'

    // "World\n" = 6 chars (6-11)
    assert_eq!(buffer.char_to_line_col(6), (1, 0)); // 'W'
    assert_eq!(buffer.char_to_line_col(10), (1, 4)); // 'd'

    // "Test" = 4 chars (12-15)
    assert_eq!(buffer.char_to_line_col(12), (2, 0)); // 'T'
    assert_eq!(buffer.char_to_line_col(15), (2, 3)); // 't'
  }

  #[test]
  fn test_line_col_to_char() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello\nWorld\nTest");

    assert_eq!(buffer.line_col_to_char(0, 0), 0); // Start of "Hello"
    assert_eq!(buffer.line_col_to_char(0, 5), 5); // '\n' after "Hello"
    assert_eq!(buffer.line_col_to_char(1, 0), 6); // Start of "World"
    assert_eq!(buffer.line_col_to_char(2, 0), 12); // Start of "Test"
    assert_eq!(buffer.line_col_to_char(2, 4), 16); // End of buffer
  }

  #[test]
  fn test_line_col_to_char_out_of_bounds() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello\nWorld");

    // Line out of bounds - should return buffer length
    assert_eq!(buffer.line_col_to_char(10, 0), buffer.len());

    // Column beyond line length - should clamp to line end
    assert_eq!(buffer.line_col_to_char(0, 100), 6); // End of first line including \n
  }

  #[test]
  fn test_char_to_line_col_round_trip() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Line 1\nLine 2\nLine 3\n");

    // Test round trip conversions
    for i in 0..buffer.len() {
      let (line, col) = buffer.char_to_line_col(i);
      let char_idx = buffer.line_col_to_char(line, col);
      assert_eq!(char_idx, i, "Round trip failed for index {}", i);
    }
  }

  #[test]
  fn test_unicode_handling() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "Hello 🌍 World");

    // The emoji is one character (Rope counts characters, not bytes)
    // "Hello " = 6 chars, "🌍" = 1 char, " World" = 6 chars
    assert_eq!(buffer.len(), 13);

    buffer.insert(6, "😀");
    assert_eq!(buffer.as_str(), "Hello 😀🌍 World");
    assert_eq!(buffer.len(), 14);
  }
}
