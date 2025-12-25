#[derive(Debug, Clone)]
pub struct TextBuffer {
    text: String,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn insert(&mut self, index: usize, content: &str) {
        self.text.insert_str(index, content);
    }

    pub fn delete(&mut self, index: usize, len: usize) {
        self.text.drain(index..index + len);
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}
