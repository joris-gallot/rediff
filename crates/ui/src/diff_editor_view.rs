use crate::line_cache::LineCache;
use crate::line_element::{EditorState, LineConfig, LineElement};
use editor::Editor;
use gpui::{
  App, Context, FocusHandle, Focusable, Font, KeyDownEvent, MouseButton, MouseDownEvent,
  MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, TextRun, UniformListScrollHandle, Window,
  black, div, opaque_grey, prelude::*, px, uniform_list, white,
};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use text::TextBuffer;

const LINE_NUMBERS_WIDTH: f32 = 60.0;
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
}

pub struct DiffEditorView {
  editor: Editor,
  focus_handle: FocusHandle,
  config: EditorConfig,
  scroll_handle: UniformListScrollHandle,
  is_selecting: bool,
  selection_anchor: Option<usize>,
  line_cache: Arc<Mutex<LineCache>>,
}

impl DiffEditorView {
  pub fn new(config: Option<EditorConfig>, cx: &mut Context<Self>) -> Self {
    let focus_handle = cx.focus_handle();

    Self {
      editor: Editor::new(),
      focus_handle,
      config: config.unwrap_or_default(),
      scroll_handle: UniformListScrollHandle::new(),
      is_selecting: false,
      selection_anchor: None,
      line_cache: Arc::new(Mutex::new(LineCache::new())),
    }
  }

  pub fn editor(&mut self) -> &mut Editor {
    &mut self.editor
  }

  fn calculate_index_from_position(&self, mouse_pos: Point<Pixels>, window: &mut Window) -> usize {
    let line_height = px(self.config.line_height());
    let line_numbers_width = px(LINE_NUMBERS_WIDTH);
    let padding = px(EDITOR_PADDING);

    let clicked_line = (mouse_pos.y / line_height).floor() as usize;

    let buffer = &self.editor.buffer;

    if clicked_line >= buffer.line_count() {
      return buffer.len();
    }

    let text = buffer
      .line(clicked_line)
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

    let shaped_line = window
      .text_system()
      .shape_line(text.into(), font_size, &[text_run], None);

    let relative_x = mouse_pos.x - line_numbers_width - padding;
    let col = shaped_line.closest_index_for_x(relative_x);

    let mut offset = 0;
    for i in 0..clicked_line {
      if let Some(line) = buffer.line(i) {
        offset += line.len();
      }
    }
    offset += col.min(buffer.line(clicked_line).unwrap_or_default().len());
    offset.min(buffer.len())
  }

  fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
    let index = self.calculate_index_from_position(event.position, window);

    match event.click_count {
      1 => {
        self.editor.cursor.index = index;
        self.editor.clear_selection();
        self.is_selecting = true;
        self.selection_anchor = Some(index);
      }
      2 => {
        self.editor.select_word_at(index);
        self.is_selecting = false;
      }
      3 => {
        self.editor.select_line_at(index);
        self.is_selecting = false;
      }
      _ => {}
    }
    cx.notify();
  }

  fn on_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
    if self.is_selecting || event.pressed_button == Some(MouseButton::Left) {
      let index = self.calculate_index_from_position(event.position, window);

      if let Some(anchor) = self.selection_anchor {
        self.editor.select_range(anchor, index);
      } else {
        self.editor.select_range(self.editor.cursor.index, index);
      }
      self.editor.cursor.index = index;
      cx.notify();
    }
  }

  fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
    self.is_selecting = false;
    self.selection_anchor = None;
    cx.notify();
  }

  fn on_mouse_up_out(
    &mut self,
    _event: &MouseUpEvent,
    _window: &mut Window,
    _cx: &mut Context<Self>,
  ) {
    self.is_selecting = false;
    self.selection_anchor = None;
  }

  fn render_line_numbers(
    &self,
    line_count: usize,
    scroll_handle: UniformListScrollHandle,
  ) -> impl IntoElement {
    let line_height = self.config.line_height();

    uniform_list(
      "line-numbers",
      line_count,
      move |range: Range<usize>, _window, _cx| {
        range
          .map(|line_idx| {
            div()
              .w(px(LINE_NUMBERS_WIDTH))
              .h(px(line_height))
              .flex()
              .items_end()
              .justify_end()
              .pr_2()
              .text_color(opaque_grey(0.5, 1.0))
              .child((line_idx + 1).to_string())
          })
          .collect::<Vec<_>>()
      },
    )
    .w(px(LINE_NUMBERS_WIDTH))
    .bg(opaque_grey(0.95, 1.0))
    .track_scroll(scroll_handle)
  }

  fn render_editor(
    &self,
    line_count: usize,
    buffer: Arc<TextBuffer>,
    editor_state: EditorState,
    scroll_handle: UniformListScrollHandle,
  ) -> impl IntoElement {
    let line_cache = self.line_cache.clone();
    let line_height = self.config.line_height();
    let font_size = self.config.font_size;

    let line_config = LineConfig {
      font_size,
      line_height,
    };

    uniform_list(
      "editor-lines",
      line_count,
      move |range: Range<usize>, _window, _cx| {
        range
          .map(|line_idx| {
            LineElement::new(
              line_idx,
              buffer.clone(),
              editor_state.clone(),
              line_cache.clone(),
              line_config.clone(),
            )
          })
          .collect::<Vec<_>>()
      },
    )
    .w_full()
    .px(px(EDITOR_PADDING))
    .track_scroll(scroll_handle)
  }

  fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
    let shift = event.keystroke.modifiers.shift;
    let cmd = event.keystroke.modifiers.platform;
    let alt = event.keystroke.modifiers.alt;

    match event.keystroke.key.as_str() {
      "left" => {
        if cmd && shift {
          self.editor.extend_selection_to_line_start();
        } else if cmd {
          self.editor.clear_selection();
          self.editor.cursor.move_to_line_start(&self.editor.buffer);
        } else if alt && shift {
          self.editor.extend_selection_word_left();
        } else if alt {
          self.editor.clear_selection();
          self.editor.cursor.move_word_left(&self.editor.buffer);
        } else if shift {
          self.editor.extend_selection_left();
        } else {
          self.editor.clear_selection();
          self.editor.cursor.move_left();
        }
      }
      "right" => {
        if cmd && shift {
          self.editor.extend_selection_to_line_end();
        } else if cmd {
          self.editor.clear_selection();
          self.editor.cursor.move_to_line_end(&self.editor.buffer);
        } else if alt && shift {
          self.editor.extend_selection_word_right();
        } else if alt {
          self.editor.clear_selection();
          self.editor.cursor.move_word_right(&self.editor.buffer);
        } else if shift {
          self.editor.extend_selection_right();
        } else {
          self.editor.clear_selection();
          self.editor.cursor.move_right(self.editor.buffer.len());
        }
      }
      "up" => {
        if cmd && shift {
          self.editor.extend_selection_to_buffer_start();
        } else if cmd {
          self.editor.clear_selection();
          self.editor.cursor.move_to_buffer_start();
        } else if shift {
          self.editor.extend_selection_up();
        } else {
          self.editor.clear_selection();
          self.editor.cursor.move_up(&self.editor.buffer);
        }
      }
      "down" => {
        if cmd && shift {
          self.editor.extend_selection_to_buffer_end();
        } else if cmd {
          self.editor.clear_selection();
          self.editor.cursor.move_to_buffer_end(&self.editor.buffer);
        } else if shift {
          self.editor.extend_selection_down();
        } else {
          self.editor.clear_selection();
          self.editor.cursor.move_down(&self.editor.buffer);
        }
      }
      "backspace" => {
        if self.editor.has_selection() {
          self.editor.delete_selection();
        } else if cmd {
          self.editor.delete_line();
        } else if alt {
          self.editor.delete_word();
        } else {
          self.editor.backspace();
        }
      }
      "enter" => {
        self.editor.delete_selection();
        self.editor.insert_char('\n');
      }
      "a" if cmd => {
        self.editor.select_all();
      }
      "c" if cmd => {
        if let Some(text) = self.editor.copy() {
          cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
      }
      "x" if cmd => {
        if let Some(text) = self.editor.cut() {
          cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
      }
      "v" if cmd => {
        if let Some(item) = cx.read_from_clipboard()
          && let Some(text) = item.text()
        {
          self.editor.paste(&text);
        }
      }
      "space" => {
        self.editor.delete_selection();
        self.editor.insert_char(' ');
      }
      key => {
        if key.len() == 1
          && !cmd
          && !event.keystroke.modifiers.control
          && let Some(c) = key.chars().next()
        {
          self.editor.delete_selection();
          self.editor.insert_char(c);
        }
      }
    }
    cx.notify();
  }
}

impl Focusable for DiffEditorView {
  fn focus_handle(&self, _cx: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

impl Render for DiffEditorView {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let line_count = self.editor.buffer.line_count();
    let font_size = self.config.font_size;
    let focus_handle = self.focus_handle.clone();
    let scroll_handle = self.scroll_handle.clone();
    let scroll_handle2 = self.scroll_handle.clone();

    let buffer = Arc::new(self.editor.buffer.clone());
    let editor_state = EditorState {
      cursor_index: self.editor.cursor.index,
      selection_range: self.editor.selection_range(),
    };

    div()
      .id("editor-view")
      .track_focus(&focus_handle)
      .size_full()
      .bg(white())
      .text_size(px(font_size))
      .on_key_down(cx.listener(Self::on_key_down))
      .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
      .on_mouse_move(cx.listener(Self::on_mouse_move))
      .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
      .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up_out))
      .child(
        div()
          .flex()
          .size_full()
          .child(self.render_line_numbers(line_count, scroll_handle))
          .child(self.render_editor(line_count, buffer, editor_state, scroll_handle2)),
      )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_editor_config_default() {
    let config = EditorConfig::default();
    assert_eq!(config.font_size, 16.0);
  }

  #[test]
  fn test_editor_config_line_height() {
    let config = EditorConfig { font_size: 16.0 };
    assert_eq!(config.line_height(), 24.0);

    let config = EditorConfig { font_size: 20.0 };
    assert_eq!(config.line_height(), 30.0);
  }

  #[test]
  fn test_editor_state_creation() {
    let editor_state = EditorState {
      cursor_index: 42,
      selection_range: None,
    };
    assert_eq!(editor_state.cursor_index, 42);
    assert!(editor_state.selection_range.is_none());
  }

  #[test]
  fn test_editor_state_with_selection() {
    let editor_state = EditorState {
      cursor_index: 10,
      selection_range: Some(5..10),
    };
    assert_eq!(editor_state.cursor_index, 10);
    assert_eq!(editor_state.selection_range, Some(5..10));
  }

  #[test]
  fn test_editor_state_clone() {
    let editor_state = EditorState {
      cursor_index: 100,
      selection_range: Some(50..100),
    };
    let cloned = editor_state.clone();
    assert_eq!(cloned.cursor_index, 100);
    assert_eq!(cloned.selection_range, Some(50..100));
  }
}
