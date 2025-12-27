use gpui::{App, Application, Bounds, Hsla, WindowBounds, WindowOptions, prelude::*, px, size};

use std::path::PathBuf;
mod workspace;
use rediff::{
  EditorConfig, EditorTheme, EditorThemeGit, EditorThemeGitColor, EditorThemeLinesNumber,
};
use workspace::Workspace;

fn main() {
  Application::new().run(|cx: &mut App| {
    let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

    Workspace::register(cx);

    let project_path = PathBuf::from("/Users/joris/workspace/git-playground/src");
    let compare_file_path = PathBuf::from("/Users/joris/workspace/git-playground/src/AppOld.vue");

    let compare_content =
      std::fs::read_to_string(&compare_file_path).expect("Failed to read compare file");

    let editor_config = EditorConfig {
      theme_light: EditorTheme {
        git: EditorThemeGit {
          added: EditorThemeGitColor {
            char_highlight_color: Hsla {
              h: 0.33,
              s: 1.0,
              l: 0.8,
              a: 1.0,
            },
            gutter_color: Hsla {
              h: 0.63,
              s: 1.0,
              l: 0.5,
              a: 1.0,
            },
            line_bg_color: Hsla {
              h: 0.53,
              s: 1.0,
              l: 0.95,
              a: 1.0,
            },
          },
          removed: EditorThemeGitColor {
            char_highlight_color: Hsla {
              h: 0.0,
              s: 1.0,
              l: 0.8,
              a: 1.0,
            },
            gutter_color: Hsla {
              h: 0.0,
              s: 1.0,
              l: 0.9,
              a: 1.0,
            },
            line_bg_color: Hsla {
              h: 0.0,
              s: 1.0,
              l: 0.95,
              a: 1.0,
            },
          },
          modified: EditorThemeGitColor {
            char_highlight_color: Hsla {
              h: 0.15,
              s: 1.0,
              l: 0.8,
              a: 1.0,
            },
            gutter_color: Hsla {
              h: 0.15,
              s: 1.0,
              l: 0.9,
              a: 1.0,
            },
            line_bg_color: Hsla {
              h: 0.15,
              s: 1.0,
              l: 0.95,
              a: 1.0,
            },
          },
        },
        lines_number: EditorThemeLinesNumber {
          bg_color: Hsla {
            h: 0.63,
            s: 1.0,
            l: 0.95,
            a: 1.0,
          },
          text_color: Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.4,
            a: 1.0,
          },
        },
      },
      ..Default::default()
    };

    // let editor_config_default = EditorConfig {
    //   ..Default::default()
    // };

    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_, cx| cx.new(|cx| Workspace::new(project_path, compare_content, editor_config, cx)),
    )
    .unwrap();

    cx.activate(true);
  });
}
