use crate::core::Editor;

use gpui::{
    App, Context, Div, FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, ScrollHandle, TextAlign, Window, black,
    div, opaque_grey, prelude::*, px, rgb, white,
};

const LINE_NUMBERS_WIDTH: f32 = 50.0;
const EDITOR_PADDING: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct EditorConfig {
    pub font_size: f32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { font_size: 16.0 }
    }
}

impl EditorConfig {
    pub fn line_height(&self) -> f32 {
        self.font_size * 1.5
    }

    pub fn cursor_height(&self) -> f32 {
        self.line_height() - 2.0
    }
}

pub struct DiffEditorView {
    editor: Editor,
    focus_handle: FocusHandle,
    config: EditorConfig,
    scroll_handle: ScrollHandle,

    is_selecting: bool,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

impl DiffEditorView {
    pub fn new(config: Option<EditorConfig>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            editor: Editor::new(),
            focus_handle,
            config: config.unwrap_or_default(),
            scroll_handle: ScrollHandle::new(),
            is_selecting: false,
            selection_start: None,
            selection_end: None,
        }
    }

    fn get_cursor_position(text: &str, cursor_index: usize) -> (usize, usize) {
        let clamped_index = cursor_index.min(text.len());
        let before_cursor = &text[..clamped_index];
        let line = before_cursor.matches('\n').count();
        let col = before_cursor
            .rfind('\n')
            .map(|pos| clamped_index - pos - 1)
            .unwrap_or(clamped_index);
        (line, col)
    }

    fn get_selection_range(&self) -> Option<std::ops::Range<usize>> {
        match (self.selection_start, self.selection_end) {
            (Some(start), Some(end)) if start != end => Some(start.min(end)..start.max(end)),
            _ => None,
        }
    }

    fn select_word_at(&self, index: usize) -> (usize, usize) {
        let text = self.editor.buffer.as_str();

        let start = text[..index]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = text[index..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| index + i)
            .unwrap_or(text.len());

        (start, end)
    }

    fn select_line_at(&self, index: usize) -> (usize, usize) {
        let (line, _col) = self.editor.buffer.char_to_line_col(index);

        let start = self.editor.buffer.line_col_to_char(line, 0);

        let end = if line + 1 < self.editor.buffer.line_count() {
            self.editor.buffer.line_col_to_char(line + 1, 0)
        } else {
            self.editor.buffer.len()
        };

        (start, end)
    }

    fn calculate_index_from_position(&self, mouse_pos: Point<Pixels>) -> usize {
        let scroll_offset = self.scroll_handle.offset();
        let config = &self.config;
        let line_height_px = px(config.line_height());
        let line_numbers_width_px = px(LINE_NUMBERS_WIDTH);
        let padding_px = px(EDITOR_PADDING);

        let adjusted_y = mouse_pos.y - scroll_offset.y;
        let clicked_line = (adjusted_y / line_height_px).max(0.0) as usize;

        let text = self.editor.buffer.as_str();
        let lines: Vec<&str> = text.split('\n').collect();

        if clicked_line >= lines.len() {
            return text.len();
        }

        let line = lines[clicked_line];
        let x_offset = mouse_pos.x - line_numbers_width_px - padding_px;

        // Calculate character widths dynamically
        // Spaces are rendered narrower, alphanumeric chars are wider
        let space_width = config.font_size * 0.27; // Approximate space width
        let char_width = config.font_size * 0.57; // Approximate letter width

        // Find the column by accumulating widths
        let mut accumulated_width = 0.0;
        let mut col = 0;

        for (i, ch) in line.chars().enumerate() {
            let ch_width = if ch == ' ' { space_width } else { char_width };

            // Check if we've passed the click position
            if accumulated_width + ch_width / 2.0 > x_offset.into() {
                col = i;
                break;
            }

            accumulated_width += ch_width;
            col = i + 1;
        }

        let col = col.min(line.len());

        let mut index = 0;
        for (i, line) in lines.iter().enumerate() {
            if i < clicked_line {
                index += line.len() + 1;
            } else if i == clicked_line {
                index += col;
                break;
            }
        }

        index.min(text.len())
    }

    fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
        self.is_selecting = false;
    }

    /// Delete selected text if any, and position cursor at selection start
    fn delete_selection(&mut self) {
        if let Some(range) = self.get_selection_range() {
            let len = range.end - range.start;
            self.editor.buffer.delete(range.start, len);

            self.editor.cursor.index = range.start;

            self.clear_selection();
        }
    }

    /// Copy selected text to clipboard
    fn copy_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(range) = self.get_selection_range() {
            let text = &self.editor.buffer.as_str()[range.clone()];
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.to_string()));
        }
    }

    /// Cut selected text to clipboard (copy + delete)
    fn cut_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(range) = self.get_selection_range() {
            let text = &self.editor.buffer.as_str()[range.clone()];
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.to_string()));

            self.delete_selection();
        }
    }

    /// Paste clipboard content at cursor position
    fn paste_from_clipboard(&mut self, cx: &mut Context<Self>) {
        if let Some(clipboard_item) = cx.read_from_clipboard()
            && let Some(text) = clipboard_item.text()
        {
            self.delete_selection();

            let cursor_pos = self.editor.cursor.index;
            self.editor.buffer.insert(cursor_pos, &text);

            self.editor.cursor.index = cursor_pos + text.len();
        }
    }

    fn all_selection(&mut self) {
        self.selection_start = Some(0);
        self.selection_end = Some(self.editor.buffer.len());
        self.editor.cursor.index = self.editor.buffer.len();
    }

    fn extend_selection_left(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_left();

        // Update selection end to new cursor positio
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_right(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_right(self.editor.buffer.len());
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_up(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_up(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_down(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_down(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_to_line_start(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_to_line_start(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_to_line_end(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_to_line_end(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_to_buffer_start(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_to_buffer_start();
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_to_buffer_end(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor
            .cursor
            .move_to_buffer_end(self.editor.buffer.len());
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_word_left(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_word_left(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn extend_selection_word_right(&mut self) {
        if self.get_selection_range().is_none() {
            self.selection_start = Some(self.editor.cursor.index);
        }

        self.editor.cursor.move_word_right(&self.editor.buffer);
        self.selection_end = Some(self.editor.cursor.index);
    }

    fn on_key_down(
        self: &mut DiffEditorView,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shift_pressed = event.keystroke.modifiers.shift;
        let cmd_pressed = event.keystroke.modifiers.platform;
        let opt_pressed = event.keystroke.modifiers.alt;

        // Handle clipboard operations first (Cmd without Shift/Option)
        if cmd_pressed && !shift_pressed && !opt_pressed {
            match event.keystroke.key.as_str() {
                "c" => {
                    self.copy_selection(cx);
                    cx.notify();
                    return;
                }
                "x" => {
                    self.cut_selection(cx);
                    cx.notify();
                    return;
                }
                "v" => {
                    self.paste_from_clipboard(cx);
                    cx.notify();
                    return;
                }
                "a" => {
                    self.all_selection();
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }

        match event.keystroke.key.as_str() {
            "enter" => {
                self.delete_selection();
                self.editor.insert_char('\n');
                cx.notify();
            }
            "backspace" => {
                if self.get_selection_range().is_some() {
                    self.delete_selection();
                } else {
                    self.editor.backspace();
                }
                cx.notify();
            }
            "space" => {
                self.delete_selection();
                self.editor.insert_char(' ');
                cx.notify();
            }
            "up" => {
                if cmd_pressed && shift_pressed {
                    self.extend_selection_to_buffer_start();
                } else if cmd_pressed {
                    self.clear_selection();
                    self.editor.cursor.move_to_buffer_start();
                } else if opt_pressed && shift_pressed {
                    // Option+Shift+Up = same as Shift+Up
                    self.extend_selection_up();
                } else if opt_pressed {
                    // Option+Up = same as Up
                    self.clear_selection();
                    self.editor.cursor.move_up(&self.editor.buffer);
                } else if shift_pressed {
                    self.extend_selection_up();
                } else {
                    self.clear_selection();
                    self.editor.cursor.move_up(&self.editor.buffer);
                }
                cx.notify();
            }
            "down" => {
                if cmd_pressed && shift_pressed {
                    self.extend_selection_to_buffer_end();
                } else if cmd_pressed {
                    self.clear_selection();
                    self.editor
                        .cursor
                        .move_to_buffer_end(self.editor.buffer.len());
                } else if opt_pressed && shift_pressed {
                    // Option+Shift+Down = same as Shift+Down
                    self.extend_selection_down();
                } else if opt_pressed {
                    // Option+Down = same as Down
                    self.clear_selection();
                    self.editor.cursor.move_down(&self.editor.buffer);
                } else if shift_pressed {
                    self.extend_selection_down();
                } else {
                    self.clear_selection();
                    self.editor.cursor.move_down(&self.editor.buffer);
                }
                cx.notify();
            }
            "left" => {
                if cmd_pressed && shift_pressed {
                    self.extend_selection_to_line_start();
                } else if cmd_pressed {
                    self.clear_selection();
                    self.editor.cursor.move_to_line_start(&self.editor.buffer);
                } else if opt_pressed && shift_pressed {
                    self.extend_selection_word_left();
                } else if opt_pressed {
                    self.clear_selection();
                    self.editor.cursor.move_word_left(&self.editor.buffer);
                } else if shift_pressed {
                    self.extend_selection_left();
                } else {
                    self.clear_selection();
                    self.editor.cursor.move_left();
                }
                cx.notify();
            }
            "right" => {
                if cmd_pressed && shift_pressed {
                    self.extend_selection_to_line_end();
                } else if cmd_pressed {
                    self.clear_selection();
                    self.editor.cursor.move_to_line_end(&self.editor.buffer);
                } else if opt_pressed && shift_pressed {
                    self.extend_selection_word_right();
                } else if opt_pressed {
                    self.clear_selection();
                    self.editor.cursor.move_word_right(&self.editor.buffer);
                } else if shift_pressed {
                    self.extend_selection_right();
                } else {
                    self.clear_selection();
                    self.editor.cursor.move_right(self.editor.buffer.len());
                }
                cx.notify();
            }
            key => {
                if let Some(ch) = key.chars().next() {
                    self.delete_selection();
                    self.editor.insert_char(ch);
                    cx.notify();
                }
            }
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self.calculate_index_from_position(event.position);

        match event.click_count {
            1 => {
                self.is_selecting = true;
                self.selection_start = Some(index);
                self.selection_end = Some(index);
                self.editor.cursor.index = index;
            }
            2 => {
                let (start, end) = self.select_word_at(index);
                self.selection_start = Some(start);
                self.selection_end = Some(end);
                self.editor.cursor.index = end;
                self.is_selecting = false;
            }
            3 => {
                let (start, end) = self.select_line_at(index);
                self.selection_start = Some(start);
                self.selection_end = Some(end);
                self.editor.cursor.index = end;
                self.is_selecting = false;
            }
            _ => {}
        }

        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting || event.pressed_button == Some(MouseButton::Left) {
            let index = self.calculate_index_from_position(event.position);
            self.selection_end = Some(index);
            self.editor.cursor.index = index;
            cx.notify();
        }
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_selecting = false;
        cx.notify();
    }

    fn on_mouse_up_out(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = false;
        cx.notify();
    }

    fn render_editor(&mut self, text: String, _cx: &mut Context<Self>) -> Div {
        let cursor_index = self.editor.cursor.index;
        let (cursor_line, cursor_col) = Self::get_cursor_position(&text, cursor_index);
        let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        let config = &self.config;
        let selection = self.get_selection_range();

        let mut line_starts = vec![0];
        let mut pos = 0;
        for line in &lines {
            pos += line.len() + 1; // +1 for \n
            line_starts.push(pos);
        }

        div()
            .flex()
            .flex_col()
            .px(px(EDITOR_PADDING))
            .w_full()
            .cursor_text()
            .bg(white())
            .font_family("monospace")
            .children(lines.into_iter().enumerate().map(|(i, line)| {
                let line_start = line_starts[i];
                let line_end = line_start + line.len();

                if let Some(ref sel) = selection {
                    if sel.start >= line_end || sel.end <= line_start {
                        // No selection on this line - render normally
                        if i == cursor_line {
                            let before = line[..cursor_col.min(line.len())].to_string();
                            let after = line[cursor_col.min(line.len())..].to_string();

                            div()
                                .flex()
                                .flex_row()
                                .line_height(px(config.line_height()))
                                .child(before)
                                .child(div().w(px(2.0)).h(px(config.cursor_height())).bg(black()))
                                .child(after)
                        } else {
                            div()
                                .line_height(px(config.line_height()))
                                .child(line.to_string())
                        }
                    } else {
                        // Line has selection
                        let sel_start_in_line =
                            sel.start.saturating_sub(line_start).min(line.len());
                        let sel_end_in_line = sel.end.saturating_sub(line_start).min(line.len());

                        let before_sel = line[..sel_start_in_line].to_string();
                        let selected = line[sel_start_in_line..sel_end_in_line].to_string();
                        let after_sel = line[sel_end_in_line..].to_string();

                        let mut row = div()
                            .flex()
                            .flex_row()
                            .line_height(px(config.line_height()));

                        if !before_sel.is_empty() {
                            row = row.child(before_sel);
                        }

                        // Always render selection background, even for empty lines
                        if !selected.is_empty() {
                            row = row
                                .child(div().bg(rgb(0x0078D4)).text_color(white()).child(selected));
                        } else if sel_start_in_line < sel_end_in_line
                            || (line.is_empty() && sel_start_in_line == 0)
                        {
                            // Empty line or empty selection - render space with background to maintain line height
                            row = row.child(div().bg(rgb(0x0078D4)).text_color(white()).child(" "));
                        }

                        if i == cursor_line {
                            let cursor_pos_in_line = cursor_col.min(line.len());
                            if cursor_pos_in_line >= sel_start_in_line
                                && cursor_pos_in_line <= sel_end_in_line
                            {
                                // Cursor is in selection
                                row = row.child(
                                    div().w(px(2.0)).h(px(config.cursor_height())).bg(white()),
                                );
                            } else if cursor_pos_in_line > sel_end_in_line {
                                // Cursor is after selection
                                let cursor_offset = cursor_pos_in_line
                                    .saturating_sub(sel_end_in_line)
                                    .min(after_sel.len());
                                let before_cursor = after_sel[..cursor_offset].to_string();
                                let after_cursor = after_sel[cursor_offset..].to_string();

                                if !before_cursor.is_empty() {
                                    row = row.child(before_cursor);
                                }
                                row = row.child(
                                    div().w(px(2.0)).h(px(config.cursor_height())).bg(black()),
                                );
                                if !after_cursor.is_empty() {
                                    row = row.child(after_cursor);
                                }
                                return row;
                            }
                        }

                        if !after_sel.is_empty() {
                            row = row.child(after_sel);
                        }

                        row
                    }
                } else {
                    // No selection at all
                    if i == cursor_line {
                        let before = line[..cursor_col.min(line.len())].to_string();
                        let after = line[cursor_col.min(line.len())..].to_string();

                        div()
                            .flex()
                            .flex_row()
                            .line_height(px(config.line_height()))
                            .child(before)
                            .child(div().w(px(2.0)).h(px(config.cursor_height())).bg(black()))
                            .child(after)
                    } else {
                        div()
                            .line_height(px(config.line_height()))
                            .child(line.to_string())
                    }
                }
            }))
    }
}

impl Focusable for DiffEditorView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DiffEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.editor.buffer.as_str().to_string();

        let lines: Vec<&str> = text.split('\n').collect();
        let config = &self.config;

        div()
            .id("editor-view")
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(white())
            .text_size(px(config.font_size))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up_out))
            .child(
                div()
                    .flex()
                    .w_full()
                    .child(
                        div()
                            .px(px(4.0))
                            .w(px(LINE_NUMBERS_WIDTH))
                            .bg(opaque_grey(0.9, 1.0))
                            .flex_col()
                            .items_center()
                            .children(lines.iter().enumerate().map(|(i, _)| {
                                div()
                                    .text_align(TextAlign::Right)
                                    .line_height(px(config.line_height()))
                                    .child((i + 1).to_string())
                            })),
                    )
                    .child(self.render_editor(text, cx)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cursor_position_start() {
        let text = "hello world";
        let (line, col) = DiffEditorView::get_cursor_position(text, 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_cursor_position_middle_of_line() {
        let text = "hello world";
        let (line, col) = DiffEditorView::get_cursor_position(text, 5);
        assert_eq!(line, 0);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_get_cursor_position_end_of_line() {
        let text = "hello world";
        let (line, col) = DiffEditorView::get_cursor_position(text, 11);
        assert_eq!(line, 0);
        assert_eq!(col, 11);
    }

    #[test]
    fn test_get_cursor_position_second_line() {
        let text = "hello\nworld";
        let (line, col) = DiffEditorView::get_cursor_position(text, 6);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_cursor_position_second_line_middle() {
        let text = "hello\nworld";
        let (line, col) = DiffEditorView::get_cursor_position(text, 9);
        assert_eq!(line, 1);
        assert_eq!(col, 3);
    }

    #[test]
    fn test_get_cursor_position_multiple_lines() {
        let text = "line1\nline2\nline3";
        let (line, col) = DiffEditorView::get_cursor_position(text, 12);
        assert_eq!(line, 2);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_cursor_position_empty_lines() {
        let text = "hello\n\nworld";
        let (line, col) = DiffEditorView::get_cursor_position(text, 7);
        assert_eq!(line, 2);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_cursor_position_beyond_text() {
        let text = "hello";
        let (line, col) = DiffEditorView::get_cursor_position(text, 100);
        assert_eq!(line, 0);
        assert_eq!(col, 5); // Clamped to text length
    }

    #[test]
    fn test_editor_config_line_height() {
        let config = EditorConfig { font_size: 16.0 };
        assert_eq!(config.line_height(), 24.0);
    }

    #[test]
    fn test_editor_config_cursor_height() {
        let config = EditorConfig { font_size: 16.0 };
        assert_eq!(config.cursor_height(), 22.0);
    }

    #[test]
    fn test_editor_config_default() {
        let config = EditorConfig::default();
        assert_eq!(config.font_size, 16.0);
    }
}
