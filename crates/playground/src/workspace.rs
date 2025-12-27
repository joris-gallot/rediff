use rediff::{DiffEditor, EditorConfig};

use gpui::{App, Entity, FontWeight, KeyBinding, Window, actions, div, prelude::*, px, rgb, white};
use std::path::PathBuf;

actions!(playground, [Quit]);

pub struct Workspace {
  editor: Entity<DiffEditor>,
  files: Vec<PathBuf>,
  current_file_path: PathBuf,
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
    config: Option<EditorConfig>,
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

    let first_path = files.first().cloned().unwrap_or_else(|| path.clone());

    let editor =
      cx.new(|cx| DiffEditor::new(first_path.clone(), compare_content.clone(), config, cx));

    Self {
      editor,
      files,
      current_file_path: first_path,
    }
  }

  fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
    cx.quit();
  }

  pub fn register(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
  }

  fn set_file_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
    self.current_file_path = path.clone();
    self.editor.update(cx, |editor, cx| {
      editor.set_file_path(path, cx);
    });
  }

  pub fn render_files_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
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
            this.set_file_path(path_clone.clone(), cx);
          }))
          .when_else(
            self.current_file_path == *path,
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
