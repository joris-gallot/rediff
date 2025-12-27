use rediff::{DiffEditor, EditorConfig};

use gpui::{App, Entity, FontWeight, KeyBinding, Window, actions, div, prelude::*, px, rgb, white};
use std::path::PathBuf;

actions!(playground, [Quit]);

pub struct Workspace {
  editor: Entity<DiffEditor>,
  files: Vec<PathBuf>,
}

const GRAY_COLOR: gpui::Hsla = gpui::Hsla {
  h: 0.0,
  s: 0.0,
  l: 0.9,
  a: 1.0,
};

impl Workspace {
  pub fn new(
    path: PathBuf,
    compare_content: String,
    config: EditorConfig,
    cx: &mut Context<Self>,
  ) -> Self {
    let files: Vec<PathBuf> = std::fs::read_dir(&path)
      .ok()
      .map(|entries| {
        entries
          .filter_map(|entry| entry.ok())
          .filter(|entry| entry.file_type().ok().is_some_and(|ft| ft.is_file()))
          .map(|entry| entry.path())
          .collect()
      })
      .unwrap_or_default();

    let first_path = if files.is_empty() {
      panic!("No files found in the provided path: {:?}", path);
    } else {
      files[0].clone()
    };

    let editor =
      cx.new(|cx| DiffEditor::new(first_path.clone(), compare_content.clone(), config, cx));

    Self { editor, files }
  }

  fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
    cx.quit();
  }

  fn render_files_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w(px(200.0))
      .border_r_1()
      .border_color(GRAY_COLOR)
      .py(px(5.0))
      .flex()
      .flex_col()
      .child(
        div()
          .px(px(10.0))
          .border_b_1()
          .border_color(GRAY_COLOR)
          .font_weight(FontWeight::SEMIBOLD)
          .pb(px(5.0))
          .child("Rediff"),
      )
      .children(self.files.iter().enumerate().map(|(i, path)| {
        let path_clone = path.clone();

        div()
          .id(("file", i))
          .px(px(10.0))
          .py(px(2.0))
          .text_color(rgb(0x333333))
          .on_click(cx.listener(move |this, _e, _w, cx| {
            this.editor.update(cx, |editor, cx| {
              editor.set_file_path(path_clone.clone(), cx);
            });
          }))
          .when_else(
            self.editor.as_mut(cx).file_path == *path,
            |d| d.bg(GRAY_COLOR),
            |d| d.hover(|this| this.bg(rgb(0xf5f5f5))),
          )
          .child(
            path
              .file_name()
              .and_then(|name| name.to_str().map(|s| s.to_string()))
              .unwrap_or_else(|| "Unnamed".to_string()),
          )
      }))
  }

  pub fn register(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
  }
}

impl Render for Workspace {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .on_action(cx.listener(Self::quit))
      .flex()
      .size_full()
      .bg(white())
      .child(self.render_files_panel(cx))
      .child(self.editor.clone())
  }
}
