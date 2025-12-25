use crate::core::Editor;

use gpui::{
    App, Context, Div, FocusHandle, Focusable, KeyDownEvent, Render, Stateful, TextAlign, Window,
    black, div, opaque_grey, prelude::*, px, white,
};

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

pub struct EditorView {
    editor: Editor,
    focus_handle: FocusHandle,
    config: EditorConfig,
}

impl EditorView {
    pub fn new(config: Option<EditorConfig>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            editor: Editor::new(),
            focus_handle,
            config: config.unwrap_or_default(),
        }
    }

    fn get_cursor_position(text: &str, cursor_index: usize) -> (usize, usize) {
        let before_cursor = &text[..cursor_index.min(text.len())];
        let line = before_cursor.matches('\n').count();
        let col = before_cursor
            .rfind('\n')
            .map(|pos| cursor_index - pos - 1)
            .unwrap_or(cursor_index);
        (line, col)
    }

    fn render_editor(&mut self, text: String, cx: &mut Context<Self>) -> Div {
        let cursor_index = self.editor.cursor.index;
        let (cursor_line, cursor_col) = Self::get_cursor_position(&text, cursor_index);
        let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        let config = &self.config;

        div()
            .flex()
            .flex_col()
            .px(px(8.0))
            .w_full()
            .bg(white())
            .font_family("monospace")
            .track_focus(&self.focus_handle)
            .children(lines.into_iter().enumerate().map(|(i, line)| {
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
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => {
                        this.editor.insert_char('\n');
                        cx.notify();
                    }
                    "backspace" => {
                        this.editor.backspace();
                        cx.notify();
                    }
                    "space" => {
                        this.editor.insert_char(' ');
                        cx.notify();
                    }
                    "up" => {
                        this.editor.cursor.move_up(this.editor.buffer.as_str());
                        cx.notify();
                    }
                    "down" => {
                        this.editor.cursor.move_down(this.editor.buffer.as_str());
                        cx.notify();
                    }
                    "left" => {
                        this.editor.cursor.move_left();
                        cx.notify();
                    }
                    "right" => {
                        this.editor
                            .cursor
                            .move_right(this.editor.buffer.as_str().len());
                        cx.notify();
                    }
                    key => {
                        if let Some(ch) = key.chars().next() {
                            this.editor.insert_char(ch);
                            cx.notify();
                        }
                    }
                }
            }))
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.editor.buffer.as_str().to_string();

        let lines: Vec<&str> = text.split('\n').collect();
        let config = &self.config;

        div()
            .id("editor-view")
            .overflow_y_scroll()
            .size_full()
            .bg(white())
            .text_size(px(config.font_size))
            .child(
                div()
                    .flex()
                    .w_full()
                    .child(
                        div()
                            .px(px(4.0))
                            .w(px(40.0))
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
