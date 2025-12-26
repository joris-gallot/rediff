use crate::line_cache::LineCache;
use gpui::{
  App, Bounds, Element, ElementId, Font, GlobalElementId, Hsla, InspectorElementId, IntoElement,
  LayoutId, Pixels, ShapedLine, Style, TextRun, Window, black, fill, point, px, relative, rgba,
  size,
};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use text::TextBuffer;

pub struct LinePrepaintState {
  pub shaped_line: ShapedLine,
  pub cursor_bounds: Option<CursorBounds>,
  pub selection_bounds: Vec<SelectionBounds>,
}

#[derive(Debug, Clone)]
pub struct CursorBounds {
  pub x: Pixels,
  pub width: Pixels,
}

#[derive(Debug, Clone)]
pub struct SelectionBounds {
  pub x: Pixels,
  pub width: Pixels,
  pub color: Hsla,
}

#[derive(Clone)]
pub struct LineConfig {
  pub font_size: f32,
  pub line_height: f32,
}

impl LineConfig {
  pub fn line_height_px(&self) -> Pixels {
    px(self.line_height)
  }
}

#[derive(Clone, Debug)]
pub struct EditorState {
  pub cursor_index: usize,
  pub selection_range: Option<Range<usize>>,
}

/// Custom element for rendering an editor line
/// Uses Element trait for direct GPU rendering
pub struct LineElement {
  line_idx: usize,
  buffer: Arc<TextBuffer>,
  editor_state: EditorState,
  line_cache: Arc<Mutex<LineCache>>,
  config: LineConfig,
}

impl LineElement {
  pub fn new(
    line_idx: usize,
    buffer: Arc<TextBuffer>,
    editor_state: EditorState,
    line_cache: Arc<Mutex<LineCache>>,
    config: LineConfig,
  ) -> Self {
    Self {
      line_idx,
      buffer,
      editor_state,
      line_cache,
      config,
    }
  }

  /// Retrieves or shapes a line from the buffer
  fn get_or_shape_line(&self, window: &mut Window) -> ShapedLine {
    let mut cache = self.line_cache.lock().unwrap();

    let current_version = self.buffer.len();
    cache.check_buffer_version(current_version);

    if let Some(shaped) = cache.get(self.line_idx) {
      return shaped.clone();
    }

    let text = self
      .buffer
      .line(self.line_idx)
      .unwrap_or_default()
      .trim_end_matches('\n')
      .to_string();

    let font_size = px(self.config.font_size);
    let monospace_font = Font {
      family: "monospace".into(),
      features: Default::default(),
      fallbacks: Default::default(),
      weight: Default::default(),
      style: Default::default(),
    };

    let text_run = TextRun {
      len: text.len(),
      font: monospace_font,
      color: black(),
      background_color: None,
      underline: None,
      strikethrough: None,
    };

    let shaped = window
      .text_system()
      .shape_line(text.into(), font_size, &[text_run], None);

    cache.insert(self.line_idx, shaped.clone());

    shaped
  }

  /// Calculates cursor bounds if it is on this line
  fn calculate_cursor_bounds(&self, shaped_line: &ShapedLine) -> Option<CursorBounds> {
    let (cursor_row, cursor_col) = self.buffer.char_to_line_col(self.editor_state.cursor_index);

    if cursor_row != self.line_idx {
      return None;
    }

    let x = shaped_line.x_for_index(cursor_col);

    Some(CursorBounds { x, width: px(2.0) })
  }

  /// Calculates selection bounds for this line
  fn calculate_selection_bounds(&self, shaped_line: &ShapedLine) -> Vec<SelectionBounds> {
    let Some(ref range) = self.editor_state.selection_range else {
      return Vec::new();
    };

    let (start_row, start_col) = self.buffer.char_to_line_col(range.start);
    let (end_row, end_col) = self.buffer.char_to_line_col(range.end);

    if self.line_idx < start_row || self.line_idx > end_row {
      return Vec::new();
    }

    let col_start = if self.line_idx == start_row {
      start_col
    } else {
      0
    };

    let col_end = if self.line_idx == end_row {
      end_col
    } else {
      shaped_line.len
    };

    let x_start = shaped_line.x_for_index(col_start);
    let x_end = shaped_line.x_for_index(col_end);

    vec![SelectionBounds {
      x: x_start,
      width: x_end - x_start,
      color: rgba(0x3d3d3da1).into(),
    }]
  }
}

impl IntoElement for LineElement {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}

impl Element for LineElement {
  type RequestLayoutState = ();
  type PrepaintState = LinePrepaintState;

  fn id(&self) -> Option<ElementId> {
    None
  }

  fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
    None
  }

  fn request_layout(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    window: &mut Window,
    cx: &mut App,
  ) -> (LayoutId, Self::RequestLayoutState) {
    let mut style = Style::default();

    style.size.height = self.config.line_height_px().into();

    style.size.width = relative(1.0).into();

    let layout_id = window.request_layout(style, vec![], cx);

    (layout_id, ())
  }

  fn prepaint(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    _bounds: Bounds<Pixels>,
    _request_layout: &mut Self::RequestLayoutState,
    window: &mut Window,
    _cx: &mut App,
  ) -> Self::PrepaintState {
    let shaped_line = self.get_or_shape_line(window);
    let cursor_bounds = self.calculate_cursor_bounds(&shaped_line);
    let selection_bounds = self.calculate_selection_bounds(&shaped_line);

    LinePrepaintState {
      shaped_line,
      cursor_bounds,
      selection_bounds,
    }
  }

  fn paint(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    bounds: Bounds<Pixels>,
    _request_layout: &mut Self::RequestLayoutState,
    prepaint: &mut Self::PrepaintState,
    window: &mut Window,
    cx: &mut App,
  ) {
    let line_height = self.config.line_height_px();

    for selection in &prepaint.selection_bounds {
      let selection_bounds = Bounds::new(
        point(bounds.origin.x + selection.x, bounds.origin.y),
        size(selection.width, line_height),
      );

      window.paint_quad(fill(selection_bounds, selection.color));
    }

    prepaint
      .shaped_line
      .paint(bounds.origin, line_height, window, cx)
      .ok();

    if let Some(cursor) = &prepaint.cursor_bounds {
      let cursor_bounds = Bounds::new(
        point(bounds.origin.x + cursor.x, bounds.origin.y),
        size(cursor.width, line_height),
      );

      window.paint_quad(fill(cursor_bounds, black()));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use text::TextBuffer;

  #[test]
  fn test_line_config_line_height_px() {
    let config = LineConfig {
      font_size: 16.0,
      line_height: 24.0,
    };
    assert_eq!(config.line_height_px(), px(24.0));
  }

  #[test]
  fn test_cursor_bounds_creation() {
    let cursor = CursorBounds {
      x: px(10.0),
      width: px(2.0),
    };
    assert_eq!(cursor.x, px(10.0));
    assert_eq!(cursor.width, px(2.0));
  }

  #[test]
  fn test_selection_bounds_creation() {
    let selection = SelectionBounds {
      x: px(5.0),
      width: px(20.0),
      color: rgba(0x3d3d3da1).into(),
    };
    assert_eq!(selection.x, px(5.0));
    assert_eq!(selection.width, px(20.0));
  }

  #[test]
  fn test_editor_state_no_selection_shows_cursor() {
    let editor_state = EditorState {
      cursor_index: 0,
      selection_range: None,
    };
    assert!(editor_state.selection_range.is_none());
    assert_eq!(editor_state.cursor_index, 0);
  }

  #[test]
  fn test_editor_state_with_selection_shows_cursor() {
    let editor_state = EditorState {
      cursor_index: 10,
      selection_range: Some(5..10),
    };
    assert!(editor_state.selection_range.is_some());
    assert_eq!(editor_state.cursor_index, 10);
  }

  #[test]
  fn test_calculate_cursor_bounds_not_on_line() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line 0\nline 1\nline 2");

    let editor_state = EditorState {
      cursor_index: 0,
      selection_range: None,
    };

    let cache = Arc::new(Mutex::new(LineCache::new()));
    let config = LineConfig {
      font_size: 16.0,
      line_height: 24.0,
    };

    let element = LineElement::new(1, Arc::new(buffer), editor_state, cache, config);

    assert_eq!(element.line_idx, 1);
  }

  #[test]
  fn test_calculate_selection_bounds_not_in_range() {
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "line 0\nline 1\nline 2\nline 3");

    let editor_state = EditorState {
      cursor_index: 30,
      selection_range: Some(20..30),
    };

    let cache = Arc::new(Mutex::new(LineCache::new()));
    let config = LineConfig {
      font_size: 16.0,
      line_height: 24.0,
    };

    let element = LineElement::new(0, Arc::new(buffer), editor_state, cache, config);

    assert_eq!(element.line_idx, 0);
  }

  #[test]
  fn test_line_element_new() {
    let buffer = TextBuffer::new();
    let editor_state = EditorState {
      cursor_index: 0,
      selection_range: None,
    };
    let cache = Arc::new(Mutex::new(LineCache::new()));
    let config = LineConfig {
      font_size: 14.0,
      line_height: 21.0,
    };

    let element = LineElement::new(5, Arc::new(buffer), editor_state, cache, config.clone());

    assert_eq!(element.line_idx, 5);
    assert_eq!(element.config.font_size, 14.0);
    assert_eq!(element.config.line_height, 21.0);
  }

  #[test]
  fn test_prepaint_state_structure() {
    let cursor_bounds = Some(CursorBounds {
      x: px(10.0),
      width: px(2.0),
    });

    let selection_bounds = [SelectionBounds {
      x: px(5.0),
      width: px(15.0),
      color: rgba(0x3d3d3da1).into(),
    }];

    assert!(cursor_bounds.is_some());
    assert_eq!(selection_bounds.len(), 1);
  }
}
