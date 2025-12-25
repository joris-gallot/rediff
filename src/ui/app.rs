use crate::core::Editor;

use gpui::{
    App, Context, Div, FocusHandle, Focusable, KeyDownEvent, Render, TextAlign, Window, div,
    opaque_grey, prelude::*, px, red, white,
};

pub struct EditorView {
    editor: Editor,
    focus_handle: FocusHandle,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            editor: Editor::new(),
            focus_handle,
        }
    }

    fn render_editor(&mut self, lines: Vec<&str>, cx: &mut Context<Self>) -> Div {
        div()
            .px(px(8.0))
            .size_full()
            .bg(white())
            .font_family("monospace")
            .track_focus(&self.focus_handle)
            .children(
                lines
                    .iter()
                    .map(|line| div().line_height(px(20.0)).child(line.to_string())),
            )
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

        div()
            .flex()
            .size_full()
            .bg(white())
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
                            .line_height(px(20.0))
                            .child((i + 1).to_string())
                    })),
            )
            .child(self.render_editor(lines, cx))
    }
}
