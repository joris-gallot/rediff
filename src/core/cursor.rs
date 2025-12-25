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
}
