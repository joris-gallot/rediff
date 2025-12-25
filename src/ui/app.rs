use crate::core::Editor;

use gpui::{
    App, Context, FocusHandle, Focusable, KeyDownEvent, Render, Window, div, prelude::*, px, red,
    white,
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
}

impl Focusable for EditorView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.editor.buffer.as_str();

        div()
            .p(px(16.0))
            .border_1()
            .size_full()
            .bg(white())
            .font_family("monospace")
            .track_focus(&self.focus_handle)
            .child(text.to_string())
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
