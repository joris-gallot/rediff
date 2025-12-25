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

    pub fn move_up(&mut self, text: &str) {
        if self.index == 0 {
            return;
        }

        let before = &text[..self.index];
        let current_line_start = before.rfind('\n').map_or(0, |pos| pos + 1);
        let col = self.index - current_line_start;

        if current_line_start == 0 {
            return;
        }

        let prev_line_start = before[..current_line_start - 1]
            .rfind('\n')
            .map_or(0, |pos| pos + 1);

        let prev_line_len = current_line_start - 1 - prev_line_start;
        self.index = prev_line_start + col.min(prev_line_len);
    }

    pub fn move_down(&mut self, text: &str) {
        if self.index >= text.len() {
            return;
        }

        let before = &text[..self.index];

        let current_line_start = before.rfind('\n').map(|pos| pos + 1).unwrap_or(0);

        let col = self.index - current_line_start;

        let current_line_end = text[self.index..]
            .find('\n')
            .map(|pos| self.index + pos)
            .unwrap_or(text.len());

        if current_line_end >= text.len() {
            return;
        }

        let next_line_start = current_line_end + 1;

        let next_line_end = text[next_line_start..]
            .find('\n')
            .map(|pos| next_line_start + pos)
            .unwrap_or(text.len());

        let next_line_len = next_line_end - next_line_start;
        self.index = next_line_start + col.min(next_line_len);
    }
}
