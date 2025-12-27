use rediff::{DiffEditor, EditorConfig};

use gpui::{
  App, Application, Bounds, Entity, FontWeight, KeyBinding, Window, WindowBounds, WindowOptions,
  actions, div, prelude::*, px, rgb, size, white,
};
use std::path::PathBuf;

actions!(playground, [Quit]);

struct PlaygroundEditor {
  editor: Entity<DiffEditor>,
  file_path: Option<PathBuf>,
}

impl PlaygroundEditor {
  pub fn new(
    file_path: Option<PathBuf>,
    compare_content: String,
    config: Option<EditorConfig>,
    cx: &mut Context<Self>,
  ) -> Self {
    let editor =
      cx.new(|cx| DiffEditor::new(file_path.clone(), compare_content.clone(), config, cx));

    Self { editor, file_path }
  }

  fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
    cx.quit();
  }
}

impl Render for PlaygroundEditor {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let path_without_file_name = self
      .file_path
      .as_ref()
      .and_then(|p| p.parent())
      .map(|p| p.to_path_buf());

    let file_name = self
      .file_path
      .as_ref()
      .and_then(|p| p.file_name())
      .and_then(|f| f.to_str().map(|s| s.to_string()));

    div()
      .on_action(cx.listener(Self::quit))
      .flex()
      .flex_col()
      .size_full()
      .bg(white())
      .child(
        div()
          .border_b_1()
          .border_color(rgb(0xe0e0e0))
          .px(px(10.0))
          .py(px(5.0))
          .flex()
          .items_center()
          .when(path_without_file_name.is_some(), |d| {
            d.children(path_without_file_name.as_ref().map(|p| {
              div()
                .text_color(rgb(0x666666))
                .child(format!("{}/", p.display()))
            }))
          })
          .when_else(
            file_name.is_some(),
            |d| {
              d.child(
                div()
                  .font_weight(FontWeight::SEMIBOLD)
                  .child(file_name.unwrap()),
              )
            },
            |d| d.child("Untitled"),
          ),
      )
      .child(self.editor.clone())
  }
}

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

    let file_path = Some(PathBuf::from(
      "/Users/joris/workspace/git-playground/src/App.vue",
    ));

    let compare_file_path = PathBuf::from("/Users/joris/workspace/git-playground/src/AppOld.vue");

    let compare_content =
      std::fs::read_to_string(&compare_file_path).expect("Failed to read compare file");

    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_, cx| cx.new(|cx| PlaygroundEditor::new(file_path, compare_content, None, cx)),
    )
    .unwrap();

    cx.activate(true);
  });
}
