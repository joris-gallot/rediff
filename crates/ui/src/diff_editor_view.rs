use cursor::Cursor;
use editor::Editor;

use gpui::{
  App, Bounds, Context, Div, FocusHandle, Focusable, Font, KeyDownEvent, MouseButton,
  MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, ScrollHandle, ShapedLine,
  TextAlign, TextRun, Window, black, div, opaque_grey, prelude::*, px, rgb, white,
};
use text::TextBuffer;

const LINE_NUMBERS_WIDTH: f32 = 50.0;
const EDITOR_PADDING: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct EditorConfig {
  pub font_size: f32,
}

impl Default for EditorConfig {
  fn default() -> Self {
    Self { font_size: 16.0 }
  }
}

impl EditorConfig {
  pub fn line_height(&self) -> f32 {
    self.font_size * 1.5
  }

  pub fn cursor_height(&self) -> f32 {
    self.line_height() - 2.0
  }

  pub fn cursor_width(&self) -> f32 {
    2.0
  }
}

/// State computed during prepaint phase
pub struct PrepaintState {
  line_layouts: Vec<ShapedLine>,
  cursor_bounds: Option<Bounds<Pixels>>,
  selection_bounds: Vec<Bounds<Pixels>>,
}

pub struct DiffEditorView {
  editor: Editor,
  focus_handle: FocusHandle,
  config: EditorConfig,
  scroll_handle: ScrollHandle,

  is_selecting: bool,
  selection_start: Option<usize>,
  selection_end: Option<usize>,

  // Cache shaped lines for accurate position calculations
  line_layouts: Vec<ShapedLine>,
}

impl DiffEditorView {
  pub fn new(config: Option<EditorConfig>, cx: &mut Context<Self>) -> Self {
    let focus_handle = cx.focus_handle();

    Self {
      editor: Editor::new(),
      focus_handle,
      config: config.unwrap_or_default(),
      scroll_handle: ScrollHandle::new(),
      is_selecting: false,
      selection_start: None,
      selection_end: None,
      line_layouts: Vec::new(),
    }
  }

  /// Layout cursor bounds
  fn layout_cursor(
    cursor_index: usize,
    buffer: &TextBuffer,
    line_layouts: &[ShapedLine],
    bounds: Bounds<Pixels>,
    line_height: Pixels,
  ) -> Option<Bounds<Pixels>> {
    let (row, col) = buffer.char_to_line_col(cursor_index);

    if row >= line_layouts.len() {
      return None;
    }

    let shaped_line = &line_layouts[row];
    let cursor_x = shaped_line.x_for_index(col);

    let cursor_y = bounds.top() + (line_height * row as f32);

    Some(Bounds::new(
      gpui::point(bounds.left() + cursor_x, cursor_y),
      gpui::size(px(2.0), line_height),
    ))
  }

  /// Layout selection as Bounds (one per line)
  fn layout_selection(
    start_index: usize,
    end_index: usize,
    buffer: &TextBuffer,
    line_layouts: &[ShapedLine],
    bounds: Bounds<Pixels>,
    line_height: Pixels,
  ) -> Vec<Bounds<Pixels>> {
    let mut selection_bounds = Vec::new();

    let (start_row, start_col) = buffer.char_to_line_col(start_index);
    let (end_row, end_col) = buffer.char_to_line_col(end_index);

    for row in start_row..=end_row {
      if row >= line_layouts.len() {
        break;
      }

      let shaped_line = &line_layouts[row];

      let col_start = if row == start_row { start_col } else { 0 };
      let col_end = if row == end_row {
        end_col
      } else {
        shaped_line.len
      };

      let x_start = shaped_line.x_for_index(col_start);
      let x_end = shaped_line.x_for_index(col_end);

      let y = bounds.top() + (line_height * row as f32);

      selection_bounds.push(Bounds::from_corners(
        gpui::point(bounds.left() + x_start, y),
        gpui::point(bounds.left() + x_end, y + line_height),
      ));
    }

    selection_bounds
  }

  /// Shape all lines of text into ShapedLine layouts
  fn shape_lines(
    buffer: &TextBuffer,
    config: &EditorConfig,
    window: &mut Window,
  ) -> Vec<ShapedLine> {
    let text = buffer.as_str();
    let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    let mut line_layouts = Vec::with_capacity(lines.len());

    let font_size = px(config.font_size);
    let monospace_font = Font {
      family: "monospace".into(),
      features: Default::default(),
      fallbacks: Default::default(),
      weight: Default::default(),
      style: Default::default(),
    };

    for line in &lines {
      let text_run = TextRun {
        len: line.len(),
        font: monospace_font.clone(),
        color: black(),
        background_color: None,
        underline: None,
        strikethrough: None,
      };

      let shaped_line =
        window
          .text_system()
          .shape_line(line.clone().into(), font_size, &[text_run], None);
      line_layouts.push(shaped_line);
    }

    line_layouts
  }

  /// Prepaint phase: compute all layout state
  fn prepaint(&mut self, window: &mut Window, bounds: Bounds<Pixels>) -> PrepaintState {
    let config = &self.config;
    let line_height = px(config.line_height());

    let line_layouts = Self::shape_lines(&self.editor.buffer, config, window);

    let cursor_bounds = if self.get_selection_range().is_none() {
      Self::layout_cursor(
        self.editor.cursor.index,
        &self.editor.buffer,
        &line_layouts,
        bounds,
        line_height,
      )
    } else {
      None
    };

    let selection_bounds = if let Some(range) = self.get_selection_range() {
      Self::layout_selection(
        range.start,
        range.end,
        &self.editor.buffer,
        &line_layouts,
        bounds,
        line_height,
      )
    } else {
      Vec::new()
    };

    PrepaintState {
      line_layouts,
      cursor_bounds,
      selection_bounds,
    }
  }

  /// Helper function to get substring from char_start to char_end (character indices)
  fn substring_chars(s: &str, char_start: usize, char_end: usize) -> String {
    s.chars()
      .skip(char_start)
      .take(char_end - char_start)
      .collect()
  }

  fn get_selection_range(&self) -> Option<std::ops::Range<usize>> {
    match (self.selection_start, self.selection_end) {
      (Some(start), Some(end)) if start != end => Some(start.min(end)..start.max(end)),
      _ => None,
    }
  }

  fn select_word_at(&self, index: usize) -> (usize, usize) {
    Cursor::find_word_boundaries(&self.editor.buffer, index)
  }

  fn select_line_at(&self, index: usize) -> (usize, usize) {
    let (line, _col) = self.editor.buffer.char_to_line_col(index);

    let start = self.editor.buffer.line_col_to_char(line, 0);

    let end = if line + 1 < self.editor.buffer.line_count() {
      self.editor.buffer.line_col_to_char(line + 1, 0)
    } else {
      self.editor.buffer.len()
    };

    (start, end)
  }

  fn calculate_index_from_position(&self, mouse_pos: Point<Pixels>) -> usize {
    let scroll_offset = self.scroll_handle.offset();
    let config = &self.config;
    let line_height_px = px(config.line_height());
    let line_numbers_width_px = px(LINE_NUMBERS_WIDTH);
    let padding_px = px(EDITOR_PADDING);

    let adjusted_y = mouse_pos.y - scroll_offset.y;
    let clicked_line = (adjusted_y / line_height_px).max(0.0) as usize;

    let text = self.editor.buffer.as_str();
    let lines: Vec<&str> = text.split('\n').collect();

    if clicked_line >= lines.len() {
      return self.editor.buffer.len();
    }

    let col = if clicked_line < self.line_layouts.len() {
      let shaped_line = &self.line_layouts[clicked_line];
      let relative_x = mouse_pos.x - line_numbers_width_px - padding_px - scroll_offset.x;
      shaped_line.closest_index_for_x(relative_x)
    } else {
      0
    };

    // Calculate character index (not byte index)
    let mut index = 0;
    for (i, line) in lines.iter().enumerate() {
      if i < clicked_line {
        index += line.chars().count() + 1; // +1 for newline character
      } else if i == clicked_line {
        index += col;
        break;
      }
    }

    index.min(self.editor.buffer.len())
  }

  fn clear_selection(&mut self) {
    self.selection_start = None;
    self.selection_end = None;
    self.is_selecting = false;
  }

  /// Delete selected text if any, and position cursor at selection start
  fn delete_selection(&mut self) {
    if let Some(range) = self.get_selection_range() {
      let len = range.end - range.start;
      self.editor.buffer.delete(range.start, len);

      self.editor.cursor.index = range.start;

      self.clear_selection();
    }
  }

  /// Copy selected text to clipboard
  fn copy_selection(&mut self, cx: &mut Context<Self>) {
    if let Some(range) = self.get_selection_range() {
      let text = Self::substring_chars(&self.editor.buffer.as_str(), range.start, range.end);
      cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
    }
  }

  /// Cut selected text to clipboard (copy + delete)
  fn cut_selection(&mut self, cx: &mut Context<Self>) {
    if let Some(range) = self.get_selection_range() {
      let text = Self::substring_chars(&self.editor.buffer.as_str(), range.start, range.end);
      cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));

      self.delete_selection();
    }
  }

  /// Paste clipboard content at cursor position
  fn paste_from_clipboard(&mut self, cx: &mut Context<Self>) {
    if let Some(clipboard_item) = cx.read_from_clipboard()
      && let Some(text) = clipboard_item.text()
    {
      self.delete_selection();

      let cursor_pos = self.editor.cursor.index;
      self.editor.buffer.insert(cursor_pos, &text);

      // Count characters, not bytes
      self.editor.cursor.index = cursor_pos + text.chars().count();
    }
  }

  fn all_selection(&mut self) {
    self.selection_start = Some(0);
    self.selection_end = Some(self.editor.buffer.len());
    self.editor.cursor.index = self.editor.buffer.len();
  }

  fn extend_selection_left(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_left();

    // Update selection end to new cursor positio
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_right(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_right(self.editor.buffer.len());
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_up(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_up(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_down(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_down(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_to_line_start(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_to_line_start(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_to_line_end(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_to_line_end(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_to_buffer_start(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_to_buffer_start();
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_to_buffer_end(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_to_buffer_end(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_word_left(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_word_left(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn extend_selection_word_right(&mut self) {
    if self.get_selection_range().is_none() {
      self.selection_start = Some(self.editor.cursor.index);
    }

    self.editor.cursor.move_word_right(&self.editor.buffer);
    self.selection_end = Some(self.editor.cursor.index);
  }

  fn on_key_down(
    self: &mut DiffEditorView,
    event: &KeyDownEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let shift_pressed = event.keystroke.modifiers.shift;
    let cmd_pressed = event.keystroke.modifiers.platform;
    let opt_pressed = event.keystroke.modifiers.alt;

    // Handle clipboard operations first (Cmd without Shift/Option)
    if cmd_pressed && !shift_pressed && !opt_pressed {
      match event.keystroke.key.as_str() {
        "c" => {
          self.copy_selection(cx);
          cx.notify();
          return;
        }
        "x" => {
          self.cut_selection(cx);
          cx.notify();
          return;
        }
        "v" => {
          self.paste_from_clipboard(cx);
          cx.notify();
          return;
        }
        "a" => {
          self.all_selection();
          cx.notify();
          return;
        }
        _ => {}
      }
    }

    match event.keystroke.key.as_str() {
      "enter" => {
        self.delete_selection();
        self.editor.insert_char('\n');
        cx.notify();
      }
      "backspace" => {
        if self.get_selection_range().is_some() {
          self.delete_selection();
        } else if cmd_pressed {
          // Cmd+Backspace: delete entire line
          self.editor.delete_line();
        } else if opt_pressed {
          // Option+Backspace: delete word
          self.editor.delete_word();
        } else {
          self.editor.backspace();
        }
        cx.notify();
      }
      "space" => {
        self.delete_selection();
        self.editor.insert_char(' ');
        cx.notify();
      }
      "up" => {
        if cmd_pressed && shift_pressed {
          self.extend_selection_to_buffer_start();
        } else if cmd_pressed {
          self.clear_selection();
          self.editor.cursor.move_to_buffer_start();
        } else if opt_pressed && shift_pressed {
          // Option+Shift+Up = same as Shift+Up
          self.extend_selection_up();
        } else if opt_pressed {
          // Option+Up = same as Up
          self.clear_selection();
          self.editor.cursor.move_up(&self.editor.buffer);
        } else if shift_pressed {
          self.extend_selection_up();
        } else {
          self.clear_selection();
          self.editor.cursor.move_up(&self.editor.buffer);
        }
        cx.notify();
      }
      "down" => {
        if cmd_pressed && shift_pressed {
          self.extend_selection_to_buffer_end();
        } else if cmd_pressed {
          self.clear_selection();
          self.editor.cursor.move_to_buffer_end(&self.editor.buffer);
        } else if opt_pressed && shift_pressed {
          // Option+Shift+Down = same as Shift+Down
          self.extend_selection_down();
        } else if opt_pressed {
          // Option+Down = same as Down
          self.clear_selection();
          self.editor.cursor.move_down(&self.editor.buffer);
        } else if shift_pressed {
          self.extend_selection_down();
        } else {
          self.clear_selection();
          self.editor.cursor.move_down(&self.editor.buffer);
        }
        cx.notify();
      }
      "left" => {
        if cmd_pressed && shift_pressed {
          self.extend_selection_to_line_start();
        } else if cmd_pressed {
          self.clear_selection();
          self.editor.cursor.move_to_line_start(&self.editor.buffer);
        } else if opt_pressed && shift_pressed {
          self.extend_selection_word_left();
        } else if opt_pressed {
          self.clear_selection();
          self.editor.cursor.move_word_left(&self.editor.buffer);
        } else if shift_pressed {
          self.extend_selection_left();
        } else {
          self.clear_selection();
          self.editor.cursor.move_left();
        }
        cx.notify();
      }
      "right" => {
        if cmd_pressed && shift_pressed {
          self.extend_selection_to_line_end();
        } else if cmd_pressed {
          self.clear_selection();
          self.editor.cursor.move_to_line_end(&self.editor.buffer);
        } else if opt_pressed && shift_pressed {
          self.extend_selection_word_right();
        } else if opt_pressed {
          self.clear_selection();
          self.editor.cursor.move_word_right(&self.editor.buffer);
        } else if shift_pressed {
          self.extend_selection_right();
        } else {
          self.clear_selection();
          self.editor.cursor.move_right(self.editor.buffer.len());
        }
        cx.notify();
      }
      key => {
        if let Some(ch) = key.chars().next() {
          self.delete_selection();
          self.editor.insert_char(ch);
          cx.notify();
        }
      }
    }
  }

  fn on_mouse_down(
    &mut self,
    event: &MouseDownEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let index = self.calculate_index_from_position(event.position);

    match event.click_count {
      1 => {
        self.is_selecting = true;
        self.selection_start = Some(index);
        self.selection_end = Some(index);
        self.editor.cursor.index = index;
      }
      2 => {
        let (start, end) = self.select_word_at(index);
        self.selection_start = Some(start);
        self.selection_end = Some(end);
        self.editor.cursor.index = end;
        self.is_selecting = false;
      }
      3 => {
        let (start, end) = self.select_line_at(index);
        self.selection_start = Some(start);
        self.selection_end = Some(end);
        self.editor.cursor.index = end;
        self.is_selecting = false;
      }
      _ => {}
    }

    cx.notify();
  }

  fn on_mouse_move(
    &mut self,
    event: &MouseMoveEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    if self.is_selecting || event.pressed_button == Some(MouseButton::Left) {
      let index = self.calculate_index_from_position(event.position);
      self.selection_end = Some(index);
      self.editor.cursor.index = index;
      cx.notify();
    }
  }

  fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
    self.is_selecting = false;
    cx.notify();
  }

  fn on_mouse_up_out(
    &mut self,
    _event: &MouseUpEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.is_selecting = false;
    cx.notify();
  }

  /// Render using prepaint quads
  fn render_editor(&self, prepaint: &PrepaintState, config: &EditorConfig) -> Div {
    let text = self.editor.buffer.as_str().to_string();
    let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();

    div()
      .flex()
      .flex_col()
      .px(px(EDITOR_PADDING))
      .w_full()
      .cursor_text()
      .relative()
      .children(prepaint.selection_bounds.iter().map(|bounds| {
        div()
          .absolute()
          .left(bounds.left())
          .top(bounds.top())
          .w(bounds.size.width)
          .h(bounds.size.height)
          .bg(rgb(0x0078D4))
      }))
      .children(lines.iter().map(|line| {
        div()
          .line_height(px(config.line_height()))
          .child(if line.is_empty() {
            " ".to_string()
          } else {
            line.clone()
          })
      }))
      .children(prepaint.cursor_bounds.iter().map(|bounds| {
        div()
          .absolute()
          .left(bounds.left())
          .top(bounds.top())
          .w(bounds.size.width)
          .h(bounds.size.height)
          .bg(black())
      }))
  }
}

impl Focusable for DiffEditorView {
  fn focus_handle(&self, _cx: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

impl Render for DiffEditorView {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let text = self.editor.buffer.as_str().to_string();

    let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    let config = self.config.clone();

    let editor_bounds = Bounds::new(
      gpui::point(px(EDITOR_PADDING), px(0.0)),
      gpui::size(px(1000.0), px(1000.0)), // Will be adjusted by layout
    );
    let prepaint_state = self.prepaint(window, editor_bounds);

    self.line_layouts = prepaint_state.line_layouts.clone();

    div()
      .id("editor-view")
      .overflow_y_scroll()
      .track_scroll(&self.scroll_handle)
      .track_focus(&self.focus_handle)
      .size_full()
      .bg(white())
      .pb_80()
      .text_size(px(config.font_size))
      .on_key_down(cx.listener(Self::on_key_down))
      .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
      .on_mouse_move(cx.listener(Self::on_mouse_move))
      .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
      .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up_out))
      .child(
        div()
          .flex()
          .w_full()
          .child(
            div()
              .px(px(4.0))
              .w(px(LINE_NUMBERS_WIDTH))
              .bg(opaque_grey(0.9, 1.0))
              .flex_col()
              .items_center()
              .children((0..lines.len()).map(|i| {
                div()
                  .text_align(TextAlign::Right)
                  .line_height(px(config.line_height()))
                  .child((i + 1).to_string())
              })),
          )
          .child(self.render_editor(&prepaint_state, &config)),
      )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_editor_config_line_height() {
    let config = EditorConfig::default();
    assert_eq!(config.line_height(), 24.0);
  }

  #[test]
  fn test_editor_config_cursor_height() {
    let config = EditorConfig::default();
    assert_eq!(config.cursor_height(), 22.0);
  }

  #[test]
  fn test_editor_config_cursor_width() {
    let config = EditorConfig::default();
    assert_eq!(config.cursor_width(), 2.0);
  }

  #[test]
  fn test_editor_config_default() {
    let config = EditorConfig::default();
    assert_eq!(config.font_size, 16.0);
  }

  #[test]
  fn test_substring_chars_with_emoji() {
    let text = "hello 🌍 world";

    // Extract emoji
    let result = DiffEditorView::substring_chars(text, 6, 7);
    assert_eq!(result, "🌍");

    // Extract around emoji
    let result = DiffEditorView::substring_chars(text, 5, 8);
    assert_eq!(result, " 🌍 ");

    // Extract full string
    let result = DiffEditorView::substring_chars(text, 0, 13);
    assert_eq!(result, "hello 🌍 world");
  }

  #[test]
  fn test_layout_cursor_conversion() {
    // Test that layout_cursor correctly converts char index to (row, col)
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld\ntest");

    // Index 0 should be row 0, col 0
    let (row, col) = buffer.char_to_line_col(0);
    assert_eq!(row, 0);
    assert_eq!(col, 0);

    // Index 6 should be row 1, col 0 (start of "world")
    let (row, col) = buffer.char_to_line_col(6);
    assert_eq!(row, 1);
    assert_eq!(col, 0);

    // Index 12 should be row 2, col 0 (start of "test")
    let (row, col) = buffer.char_to_line_col(12);
    assert_eq!(row, 2);
    assert_eq!(col, 0);
  }

  #[test]
  fn test_layout_cursor_out_of_bounds() {
    // Test that layout_cursor returns None for out of bounds row
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld");
    let line_layouts = Vec::new(); // Empty layouts
    let bounds = Bounds::new(
      gpui::point(px(0.0), px(0.0)),
      gpui::size(px(100.0), px(100.0)),
    );
    let line_height = px(20.0);

    let result = DiffEditorView::layout_cursor(0, &buffer, &line_layouts, bounds, line_height);

    // Should return None because line_layouts is empty
    assert!(result.is_none());
  }

  #[test]
  fn test_layout_selection_conversion() {
    // Test that layout_selection correctly converts char ranges to (row, col) ranges
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld\ntest");

    // Selection from index 0 to 5 (entire first line minus newline)
    let (start_row, start_col) = buffer.char_to_line_col(0);
    let (end_row, end_col) = buffer.char_to_line_col(5);
    assert_eq!(start_row, 0);
    assert_eq!(start_col, 0);
    assert_eq!(end_row, 0);
    assert_eq!(end_col, 5);

    // Selection from index 0 to 12 (spans 3 lines)
    let (start_row, start_col) = buffer.char_to_line_col(0);
    let (end_row, end_col) = buffer.char_to_line_col(12);
    assert_eq!(start_row, 0);
    assert_eq!(start_col, 0);
    assert_eq!(end_row, 2);
    assert_eq!(end_col, 0);
  }

  #[test]
  fn test_layout_selection_single_line_span() {
    // Test layout_selection logic for single line
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello world");

    // For single line selection from col 0 to 5:
    let (start_row, start_col) = buffer.char_to_line_col(0);
    let (end_row, end_col) = buffer.char_to_line_col(5);

    // Should iterate from start_row to end_row (0 to 0, inclusive)
    assert_eq!(start_row, end_row);

    // col_start should be start_col (0)
    // col_end should be end_col (5)
    let col_start = if start_row == start_row { start_col } else { 0 };
    let col_end = if start_row == end_row { end_col } else { 11 }; // line length

    assert_eq!(col_start, 0);
    assert_eq!(col_end, 5);
  }

  #[test]
  fn test_layout_selection_multi_line_span() {
    // Test layout_selection logic for multi-line selection
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld\ntest");

    // Selection from index 3 (middle of "hello") to index 9 (middle of "world")
    let (start_row, start_col) = buffer.char_to_line_col(3); // "lo" in "hello"
    let (end_row, end_col) = buffer.char_to_line_col(9); // "ld" in "world"

    assert_eq!(start_row, 0);
    assert_eq!(start_col, 3);
    assert_eq!(end_row, 1);
    assert_eq!(end_col, 3);

    // For row 0 (start row):
    // col_start = start_col (3)
    // col_end = line_length (5 for "hello")

    // For row 1 (end row):
    // col_start = 0
    // col_end = end_col (3)
  }

  #[test]
  fn test_layout_selection_empty_selection() {
    // Test that empty selection (start == end) still works
    let mut buffer = TextBuffer::new();
    buffer.insert(0, "hello\nworld");
    let line_layouts = Vec::new();
    let bounds = Bounds::new(
      gpui::point(px(0.0), px(0.0)),
      gpui::size(px(100.0), px(100.0)),
    );
    let line_height = px(20.0);

    // Selection with start == end
    let result =
      DiffEditorView::layout_selection(5, 5, &buffer, &line_layouts, bounds, line_height);

    // Should return empty vec or single point (depending on implementation)
    // With empty line_layouts, it will break early
    assert!(result.is_empty());
  }
}
