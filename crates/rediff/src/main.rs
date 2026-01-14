use editor::*;
use gpui::{
  App, Application, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, Window,
  WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};

const INITIAL_WINDOW_WIDTH: f32 = 1200.0;
const INITIAL_WINDOW_HEIGHT: f32 = 800.0;

struct EditorExample {
  editor: Entity<Editor>,
  focus_handle: FocusHandle,
}

impl Focusable for EditorExample {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

impl Render for EditorExample {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .bg(rgb(0xaaaaaa))
      .track_focus(&self.focus_handle(cx))
      .flex()
      .flex_col()
      .size_full()
      .child(self.editor.clone())
  }
}

pub fn generate_rust_test_content_100k() -> String {
  let base_content = r#"// Rust example with syntax highlighting - THIS IS A VERY LONG LINE TO TEST HORIZONTAL SCROLLING ============================================================================================================================================================================
fn main() {
    let x = 42;
    let name = "World";
    println!("Hello, {}! The answer is {}", name, x);
    let very_long_variable_name_to_test_horizontal_scrolling = "This is a very long string that should cause horizontal scrolling when displayed in the editor viewport because it exceeds the normal width of the editor window";

    // Test various token types with a very long comment that goes on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on
    let mut counter = 0;
    for i in 0..10 {
        counter += i;
    }

    if counter > 20 {
        println!("Counter is greater than 20: {} - and here is some extra text to make this line very long so we can test horizontal scrolling in the editor viewport", counter);
    }
}

struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: &str, age: u32) -> Self {
        Self {
            name: name.to_string(),
            age,
        }
    }

    fn greet(&self) {
        println!("Hi, I'm {} and I'm {} years old - and this is another very long line that should demonstrate horizontal scrolling capabilities in the text editor with the sticky gutter feature", self.name, self.age);
    }
}

// Test with more lines for scrolling
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2), // Recursive call with a long comment ===============================================================================================
    }
}

#[derive(Debug, Clone)]
enum Color {
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8), // RGB color variant with three u8 values representing red, green, and blue components respectively in the standard RGB color model used in computer graphics
}

trait Drawable {
    fn draw(&self);
}
"#;

  // Repeat content to reach 100K+ lines
  let mut content = String::new();
  let base_line_count = base_content.lines().count();
  let repetitions = (100_000 / base_line_count) + 1;

  for i in 0..repetitions {
    content.push_str(&format!("// ===== Repetition {} =====\n", i + 1));
    content.push_str(base_content);
    content.push('\n');
  }

  content
}

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(
      None,
      size(px(INITIAL_WINDOW_WIDTH), px(INITIAL_WINDOW_HEIGHT)),
      cx,
    );

    cx.bind_keys([
      KeyBinding::new("enter", Enter, None),
      KeyBinding::new("tab", Tab, None),
      KeyBinding::new("backspace", Backspace, None),
      KeyBinding::new("alt-backspace", BackspaceWord, None),
      KeyBinding::new("cmd-backspace", BackspaceAll, None),
      KeyBinding::new("delete", Delete, None),
      KeyBinding::new("up", Up, None),
      KeyBinding::new("down", Down, None),
      KeyBinding::new("left", Left, None),
      KeyBinding::new("alt-left", AltLeft, None),
      KeyBinding::new("cmd-left", CmdLeft, None),
      KeyBinding::new("right", Right, None),
      KeyBinding::new("alt-right", AltRight, None),
      KeyBinding::new("cmd-right", CmdRight, None),
      KeyBinding::new("cmd-up", CmdUp, None),
      KeyBinding::new("cmd-down", CmdDown, None),
      KeyBinding::new("shift-up", SelectUp, None),
      KeyBinding::new("shift-down", SelectDown, None),
      KeyBinding::new("shift-cmd-left", SelectCmdLeft, None),
      KeyBinding::new("shift-cmd-right", SelectCmdRight, None),
      KeyBinding::new("shift-cmd-up", SelectCmdUp, None),
      KeyBinding::new("shift-cmd-down", SelectCmdDown, None),
      KeyBinding::new("shift-left", SelectLeft, None),
      KeyBinding::new("shift-alt-left", SelectWordLeft, None),
      KeyBinding::new("shift-right", SelectRight, None),
      KeyBinding::new("shift-alt-right", SelectWordRight, None),
      KeyBinding::new("cmd-a", SelectAll, None),
      KeyBinding::new("cmd-v", Paste, None),
      KeyBinding::new("cmd-c", Copy, None),
      KeyBinding::new("cmd-x", Cut, None),
      KeyBinding::new("cmd-z", Undo, None),
      KeyBinding::new("cmd-shift-z", Redo, None),
      KeyBinding::new("home", Home, None),
      KeyBinding::new("end", End, None),
      KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
    ]);

    let window = cx
      .open_window(
        WindowOptions {
          window_bounds: Some(WindowBounds::Windowed(bounds)),
          ..Default::default()
        },
        |_, cx| {
          let content = generate_rust_test_content_100k();
          let base_content = content.clone();
          cx.new(|cx| EditorExample {
            editor: cx.new(move |cx| Editor::new(&content, Some(&base_content), Some("rs"), cx)),
            focus_handle: cx.focus_handle(),
          })
        },
      )
      .unwrap();

    window
      .update(cx, |view, window, cx| {
        window.focus(&view.editor.focus_handle(cx), cx);
        cx.activate(true);
      })
      .unwrap();

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
  });
}
