use crate::line_cache::LineCache;
use crate::line_element::{DiffBackground, EditorState, LineConfig, LineElement};
use editor::{DiffLine, DiffLineKind, Differ, Editor};
use gpui::{
  App, ClipboardItem, Context, FocusHandle, Focusable, Font, Hsla, KeyDownEvent, MouseButton,
  MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, TextRun,
  UniformListScrollHandle, Window, black, div, opaque_grey, prelude::*, px, rgba, uniform_list,
  white,
};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use text::TextBuffer;

const LINE_NUMBERS_WIDTH: f32 = 60.0;
const DIFF_GUTTER_WIDTH: f32 = 8.0;
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
  file_path: Option<PathBuf>,
  is_dirty: bool,
  was_focused: bool,
  compare_content: String,
  differ: Differ,
}

impl DiffEditorView {
  pub fn new(
    file_path: Option<PathBuf>,
    compare_content: String,
    config: Option<EditorConfig>,
    cx: &mut Context<Self>,
  ) -> Self {
    let focus_handle = cx.focus_handle();

    let editor = if let Some(ref path) = file_path {
      match TextBuffer::from_file(path) {
        Ok(buffer) => editor::Editor {
          buffer,
          cursor: cursor::Cursor::new(),
          selection: None,
        },
        Err(e) => {
          eprintln!("Failed to load file: {}", e);
          editor::Editor::new()
        }
      }
    } else {
      editor::Editor::new()
    };

    let differ = Differ::new(compare_content.clone());

    Self {
      editor,
      focus_handle,
      config: config.unwrap_or_default(),
      scroll_handle: UniformListScrollHandle::new(),
      is_selecting: false,
      selection_anchor: None,
      line_cache: Arc::new(Mutex::new(LineCache::new())),
      file_path,
      is_dirty: false,
      was_focused: false,
      compare_content,
      differ,
    }
  }

  pub fn editor(&mut self) -> &mut Editor {
    &mut self.editor
  }

  fn compute_diff(&self) -> Vec<DiffLine> {
    self.differ.compute_diff(&self.editor.buffer.as_str())
  }

  pub fn update_compare_content(&mut self, content: String) {
    self.compare_content = content.clone();
    self.differ = Differ::new(content);
  }

  fn mark_dirty(&mut self) {
    self.is_dirty = true;
  }

  fn reload_file(&mut self, cx: &mut Context<Self>) {
    if let Some(ref path) = self.file_path {
      match TextBuffer::from_file(path) {
        Ok(buffer) => {
          let cursor_index = self.editor.cursor.index.min(buffer.len());
          self.editor.buffer = buffer;
          self.editor.cursor.index = cursor_index;
          self.editor.selection = None;
          self.is_dirty = false;
          println!("File reloaded: {:?}", path);
          cx.notify();
        }
        Err(e) => {
          eprintln!("Failed to reload file: {}", e);
        }
      }
    }
  }

  fn calculate_index_from_position(&self, mouse_pos: Point<Pixels>, window: &mut Window) -> usize {
    let line_height = px(self.config.line_height());
    let line_numbers_width = px(LINE_NUMBERS_WIDTH + DIFF_GUTTER_WIDTH);
    let padding = px(EDITOR_PADDING);

    let clicked_visual_line = (mouse_pos.y / line_height).floor() as usize;

    let diff_lines = self.compute_diff();

    if clicked_visual_line >= diff_lines.len() {
      return self.editor.buffer.len();
    }

    let diff_line = &diff_lines[clicked_visual_line];

    // If clicking on a removed line (no line number), ignore the click
    if diff_line.line_number == 0 {
      return self.editor.cursor.index;
    }

    let buffer_line_idx = diff_line.line_number - 1;
    let buffer = &self.editor.buffer;

    if buffer_line_idx >= buffer.line_count() {
      return buffer.len();
    }

    let text = buffer
      .line(buffer_line_idx)
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
    for i in 0..buffer_line_idx {
      if let Some(line) = buffer.line(i) {
        offset += line.len();
      }
    }
    offset += col.min(buffer.line(buffer_line_idx).unwrap_or_default().len());
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

  fn render_diff_gutter(
    &self,
    diff_lines: Vec<DiffLine>,
    scroll_handle: UniformListScrollHandle,
  ) -> impl IntoElement {
    let line_height = self.config.line_height();
    let item_count = diff_lines.len();

    uniform_list(
      "diff-gutter",
      item_count,
      move |range: Range<usize>, _window, _cx| {
        range
          .map(|idx| {
            let line = &diff_lines[idx];
            let bg_color: Hsla = match line.kind {
              DiffLineKind::Added => rgba(0x28a745ff).into(),
              DiffLineKind::Removed => rgba(0xd73a49ff).into(),
              DiffLineKind::Modified if line.line_number == 0 => rgba(0xd73a49ff).into(),
              DiffLineKind::Modified => rgba(0x28a745ff).into(),
              DiffLineKind::Unchanged => opaque_grey(0.95, 1.0),
            };

            div().h(px(line_height)).w_full().bg(bg_color)
          })
          .collect::<Vec<_>>()
      },
    )
    .w(px(DIFF_GUTTER_WIDTH))
    .track_scroll(scroll_handle)
  }

  fn render_line_numbers(
    &self,
    diff_lines: Vec<DiffLine>,
    scroll_handle: UniformListScrollHandle,
  ) -> impl IntoElement {
    let line_height = self.config.line_height();
    let item_count = diff_lines.len();

    uniform_list(
      "line-numbers",
      item_count,
      move |range: Range<usize>, _window, _cx| {
        range
          .map(|idx| {
            let line = &diff_lines[idx];
            let line_num_text = if line.line_number == 0 {
              "".to_string()
            } else {
              line.line_number.to_string()
            };

            div()
              .w(px(LINE_NUMBERS_WIDTH))
              .h(px(line_height))
              .flex()
              .items_end()
              .justify_end()
              .pr_2()
              .text_color(opaque_grey(0.5, 1.0))
              .child(line_num_text)
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
    diff_lines: Vec<DiffLine>,
    buffer: Arc<TextBuffer>,
    editor_state: EditorState,
    scroll_handle: UniformListScrollHandle,
  ) -> impl IntoElement {
    let line_cache = self.line_cache.clone();
    let line_height = self.config.line_height();
    let font_size = self.config.font_size;
    let item_count = diff_lines.len();

    let line_config = LineConfig {
      font_size,
      line_height,
    };

    uniform_list(
      "editor-lines",
      item_count,
      move |range: Range<usize>, _window, _cx| {
        range
          .map(|idx| {
            let line = &diff_lines[idx];

            // For removed/modified lines without line number, don't show cursor
            // Use an impossible line_idx so the cursor won't be calculated for this line
            let line_idx = if line.line_number == 0 {
              usize::MAX
            } else {
              line.line_number - 1
            };

            // Create a modified editor_state that hides cursor on removed lines
            let modified_editor_state = if line.line_number == 0 {
              // Hide cursor by setting it to an impossible position
              EditorState {
                cursor_index: usize::MAX,
                selection_range: editor_state.selection_range.clone(),
              }
            } else {
              editor_state.clone()
            };

            // For removed lines, use text override since they're not in the buffer
            let text_override = match line.kind {
              DiffLineKind::Removed => Some(line.content.clone()),
              DiffLineKind::Modified if line.line_number == 0 => Some(line.content.clone()),
              _ => None,
            };

            let diff_bg = match line.kind {
              DiffLineKind::Added => Some(DiffBackground {
                color: rgba(0x28a74520).into(),
                char_highlights: line.char_changes.clone(),
                highlight_color: rgba(0x28a74560).into(),
              }),
              DiffLineKind::Removed => Some(DiffBackground {
                color: rgba(0xd73a4920).into(),
                char_highlights: line.char_changes.clone(),
                highlight_color: rgba(0xd73a4960).into(),
              }),
              DiffLineKind::Modified if line.line_number == 0 => Some(DiffBackground {
                color: rgba(0xd73a4920).into(),
                char_highlights: line.char_changes.clone(),
                highlight_color: rgba(0xd73a4960).into(),
              }),
              DiffLineKind::Modified => Some(DiffBackground {
                color: rgba(0x28a74520).into(),
                char_highlights: line.char_changes.clone(),
                highlight_color: rgba(0x28a74560).into(),
              }),
              DiffLineKind::Unchanged => None,
            };

            let mut element = LineElement::new(
              line_idx,
              buffer.clone(),
              modified_editor_state,
              line_cache.clone(),
              line_config.clone(),
            );

            if let Some(text) = text_override {
              element = element.with_text_override(text);
            }

            if let Some(bg) = diff_bg {
              element = element.with_diff_background(bg);
            }

            element
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
      "s" if cmd && !shift && !alt => {
        if let Some(ref path) = self.file_path {
          match self.editor.buffer.save_to_file(path) {
            Ok(_) => {
              self.is_dirty = false;
              println!("File saved: {:?}", path);
              cx.notify();
            }
            Err(e) => {
              eprintln!("Failed to save file: {}", e);
            }
          }
        }
        return;
      }
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
        self.mark_dirty();
      }
      "enter" => {
        self.editor.delete_selection();
        self.editor.insert_char('\n');
        self.mark_dirty();
      }
      "a" if cmd => {
        self.editor.select_all();
      }
      "c" if cmd => {
        if let Some(text) = self.editor.copy() {
          cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
      }
      "x" if cmd => {
        if let Some(text) = self.editor.cut() {
          cx.write_to_clipboard(ClipboardItem::new_string(text));
          self.mark_dirty();
        }
      }
      "v" if cmd => {
        if let Some(item) = cx.read_from_clipboard()
          && let Some(text) = item.text()
        {
          self.editor.paste(&text);
          self.mark_dirty();
        }
      }
      "space" => {
        self.editor.delete_selection();
        self.editor.insert_char(' ');
        self.mark_dirty();
      }
      key => {
        if key.len() == 1
          && !cmd
          && !event.keystroke.modifiers.control
          && let Some(c) = key.chars().next()
        {
          self.editor.delete_selection();
          self.editor.insert_char(c);
          self.mark_dirty();
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
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_focused = self.focus_handle.is_focused(window);
    if is_focused && !self.was_focused && !self.is_dirty {
      self.reload_file(cx);
    }
    self.was_focused = is_focused;

    let font_size = self.config.font_size;
    let focus_handle = self.focus_handle.clone();
    let scroll_handle = self.scroll_handle.clone();
    let scroll_handle2 = self.scroll_handle.clone();
    let scroll_handle3 = self.scroll_handle.clone();

    let buffer = Arc::new(self.editor.buffer.clone());
    let editor_state = EditorState {
      cursor_index: self.editor.cursor.index,
      selection_range: self.editor.selection_range(),
    };

    let diff_lines = self.compute_diff();
    let diff_lines2 = diff_lines.clone();
    let diff_lines3 = diff_lines.clone();

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
          .child(self.render_diff_gutter(diff_lines, scroll_handle))
          .child(self.render_line_numbers(diff_lines2, scroll_handle2))
          .child(self.render_editor(diff_lines3, buffer, editor_state, scroll_handle3)),
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
