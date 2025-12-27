use ui::DiffEditorView;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use std::path::PathBuf;

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

    let file_path = Some(PathBuf::from("some-file.txt"));

    let compare_file_path = PathBuf::from("some-another-file.txt");

    let compare_content =
      std::fs::read_to_string(&compare_file_path).expect("Failed to read compare file");

    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_, cx| cx.new(|cx| DiffEditorView::new(file_path, compare_content, None, cx)),
    )
    .unwrap();

    cx.activate(true);
  });
}
