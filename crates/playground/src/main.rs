use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

use std::path::PathBuf;
mod workspace;
use workspace::Workspace;

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

    Workspace::register(cx);

    let project_path = PathBuf::from("/Users/joris/workspace/git-playground/src");

    let compare_file_path = PathBuf::from("/Users/joris/workspace/git-playground/src/AppOld.vue");

    let compare_content =
      std::fs::read_to_string(&compare_file_path).expect("Failed to read compare file");

    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_, cx| cx.new(|cx| Workspace::new(project_path, compare_content, None, cx)),
    )
    .unwrap();

    cx.activate(true);
  });
}
