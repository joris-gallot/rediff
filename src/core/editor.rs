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

#[test]
fn insert_and_delete() {
    let mut editor = Editor::new();
    editor.insert_char('H');
    editor.insert_char('i');
    assert_eq!(editor.buffer.as_str(), "Hi");

    editor.backspace();
    assert_eq!(editor.buffer.as_str(), "H");
}
