use super::buffer::TextBuffer;

pub struct Cursor {
    pub index: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn move_left(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn move_right(&mut self, max: usize) {
        if self.index < max {
            self.index += 1;
        }
    }

    pub fn move_up(&mut self, buffer: &TextBuffer) {
        let (line, col) = buffer.char_to_line_col(self.index);

        if line > 0 {
            let new_line = line - 1;
            let line_len = buffer
                .line(new_line)
                .map(|l| l.trim_end_matches('\n').len())
                .unwrap_or(0);
            let new_col = col.min(line_len);
            self.index = buffer.line_col_to_char(new_line, new_col);
        } else {
            self.index = 0;
        }
    }

    pub fn move_down(&mut self, buffer: &TextBuffer) {
        let (line, col) = buffer.char_to_line_col(self.index);

        if line < buffer.line_count() - 1 {
            let new_line = line + 1;
            let line_len = buffer
                .line(new_line)
                .map(|l| l.trim_end_matches('\n').len())
                .unwrap_or(0);
            let new_col = col.min(line_len);
            self.index = buffer.line_col_to_char(new_line, new_col);
        } else {
            self.index = buffer.len();
        }
    }

    pub fn move_to_line_start(&mut self, buffer: &TextBuffer) {
        let (line, _col) = buffer.char_to_line_col(self.index);
        self.index = buffer.line_col_to_char(line, 0);
    }

    pub fn move_to_line_end(&mut self, buffer: &TextBuffer) {
        let (line, _col) = buffer.char_to_line_col(self.index);
        let line_len = buffer
            .line(line)
            .map(|l| l.trim_end_matches('\n').len())
            .unwrap_or(0);
        self.index = buffer.line_col_to_char(line, line_len);
    }

    pub fn move_to_buffer_start(&mut self) {
        self.index = 0;
    }

    pub fn move_to_buffer_end(&mut self, buffer_len: usize) {
        self.index = buffer_len;
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
        let mut cursor = Cursor { index: 5 };

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.index, 0);
    }

    #[test]
    fn test_move_to_line_start_multiline() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "line1\nline2\nline3");
        let mut cursor = Cursor { index: 14 }; // middle of line3

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.index, 12); // start of line3
    }

    #[test]
    fn test_move_to_line_end() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "hello world");
        let mut cursor = Cursor { index: 5 };

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.index, 11);
    }

    #[test]
    fn test_move_to_line_end_multiline() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "line1\nline2\nline3");
        let mut cursor = Cursor { index: 8 }; // middle of line2

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.index, 11); // end of line2 (before \n)
    }

    #[test]
    fn test_move_to_line_end_excludes_newline() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "hello\nworld");
        let mut cursor = Cursor { index: 2 }; // in "hello"

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.index, 5); // before \n, not at 6 (which is \n)
    }

    #[test]
    fn test_move_to_buffer_start() {
        let mut cursor = Cursor { index: 100 };
        cursor.move_to_buffer_start();
        assert_eq!(cursor.index, 0);
    }

    #[test]
    fn test_move_to_buffer_end() {
        let mut cursor = Cursor { index: 5 };
        cursor.move_to_buffer_end(100);
        assert_eq!(cursor.index, 100);
    }

    #[test]
    fn test_move_to_line_start_already_at_start() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "hello world");
        let mut cursor = Cursor { index: 0 };

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.index, 0);
    }

    #[test]
    fn test_move_to_line_end_already_at_end() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "hello world");
        let mut cursor = Cursor { index: 11 };

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.index, 11);
    }

    #[test]
    fn test_move_to_line_start_empty_line() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "line1\n\nline3");
        let mut cursor = Cursor { index: 6 }; // on empty line

        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.index, 6); // stays at start of empty line
    }

    #[test]
    fn test_move_to_line_end_empty_line() {
        let mut buffer = TextBuffer::new();
        buffer.insert(0, "line1\n\nline3");
        let mut cursor = Cursor { index: 6 }; // on empty line

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.index, 6); // stays at same position (line is empty)
    }
}
