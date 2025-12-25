mod core;
mod ui;
use crate::ui::DiffEditorView;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_, cx| cx.new(|cx| DiffEditorView::new(None, cx)),
    )
    .unwrap();

    cx.activate(true);
  });
}
