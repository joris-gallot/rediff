use gpui::{
  App, Entity, FontWeight, Hsla, KeyBinding, Window, actions, div, opaque_grey, prelude::*, px,
  rgb, white,
};

use rediff::{DiffEditor, EditorConfig};
use std::path::PathBuf;

actions!(playground, [Quit]);

pub struct Workspace {
  editor: Entity<DiffEditor>,
  files: Vec<PathBuf>,
  dark_mode: bool,
}

const GRAY_COLOR: Hsla = Hsla {
  h: 0.0,
  s: 0.0,
  l: 0.9,
  a: 1.0,
};

impl Workspace {
  pub fn new(path: PathBuf, compare_content: String, cx: &mut Context<Self>) -> Self {
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

    let editor = cx.new(|cx| {
      DiffEditor::new(
        first_path.clone(),
        compare_content.clone(),
        EditorConfig {
          ..Default::default()
        },
        cx,
      )
    });

    editor.as_mut(cx).toggle_dark_mode();

    Self {
      editor,
      files,
      dark_mode: true,
    }
  }

  fn toggle_dark_mode(&mut self, cx: &mut Context<Self>) {
    self.dark_mode = !self.dark_mode;
    self.editor.as_mut(cx).toggle_dark_mode();
  }

  fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
    cx.quit();
  }

  fn render_files_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
    let current_file_path = self.editor.as_mut(cx).file_path.clone();
    let dark_mode = self.dark_mode;

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
          .flex()
          .items_center()
          .justify_between()
          .border_color(GRAY_COLOR)
          .font_weight(FontWeight::SEMIBOLD)
          .pb(px(5.0))
          .child("Rediff")
          .when_else(
            dark_mode,
            |d| d.text_color(white()),
            |d| d.text_color(rgb(0x333333)),
          )
          .child(
            div()
              .id("dark_mode_toggle")
              .cursor_pointer()
              .on_click(cx.listener(|this, _e, _w, cx| {
                this.toggle_dark_mode(cx);
              }))
              .child(if self.dark_mode { "🌙" } else { "☀️" }),
          ),
      )
      .children({
        self.files.iter().enumerate().map(|(i, path)| {
          let path_clone = path.clone();
          let current_file_path = current_file_path.clone();

          div()
            .id(("file", i))
            .px(px(10.0))
            .py(px(2.0))
            .on_click(cx.listener(move |this, _e, _w, cx| {
              this.editor.update(cx, |editor, cx| {
                editor.set_file_path(path_clone.clone(), cx);
              });
            }))
            .when_else(
              dark_mode,
              |d| {
                d.text_color(white()).when_else(
                  current_file_path == *path,
                  |d| d.bg(opaque_grey(0.5, 1.0)),
                  |d| d.hover(|this| this.bg(opaque_grey(0.3, 1.0))),
                )
              },
              |d| {
                d.text_color(rgb(0x333333)).when_else(
                  current_file_path == *path,
                  |d| d.bg(opaque_grey(0.8, 1.0)),
                  |d| d.hover(|this| this.bg(opaque_grey(0.9, 1.0))),
                )
              },
            )
            .child(
              path
                .file_name()
                .and_then(|name| name.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "Unnamed".to_string()),
            )
        })
      })
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
      .when_else(
        self.dark_mode,
        |d| d.bg(opaque_grey(0.1, 1.0)),
        |d| d.bg(white()),
      )
      .child(self.render_files_panel(cx))
      .child(self.editor.clone())
  }
}
