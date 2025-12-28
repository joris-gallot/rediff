use gpui::{Hsla, black, blue, green, opaque_grey, red, white};

#[derive(Clone, Debug)]
pub struct EditorThemeGitColor {
  pub line_bg_color: Hsla,
  pub char_highlight_color: Hsla,
  pub gutter_color: Hsla,
}

#[derive(Clone, Debug)]
pub struct EditorThemeGit {
  pub added: EditorThemeGitColor,
  pub removed: EditorThemeGitColor,
  pub modified: EditorThemeGitColor,
}

#[derive(Clone, Debug)]
pub struct EditorThemeCursorColor {
  pub color: Hsla,
  pub selection_color: Hsla,
}

#[derive(Clone, Debug)]
pub struct EditorThemePairColor {
  pub bg_color: Hsla,
  pub text_color: Hsla,
}

#[derive(Clone, Debug)]
pub struct EditorTheme {
  pub cursor: EditorThemeCursorColor,
  pub code: EditorThemePairColor,
  pub line_numbers: EditorThemePairColor,
  pub git: EditorThemeGit,
}

#[derive(Clone, Debug)]
pub struct EditorConfig {
  pub font_size: f32,
  pub tab_size: usize,
  pub theme_light: EditorTheme,
  pub theme_dark: EditorTheme,
}

impl Default for EditorConfig {
  fn default() -> Self {
    Self {
      font_size: 16.0,
      tab_size: 2,
      theme_light: Self::default_theme_light(),
      theme_dark: Self::default_theme_dark(),
    }
  }
}

impl EditorConfig {
  pub fn line_height(&self) -> f32 {
    self.font_size * 1.5
  }

  pub fn default_theme_light() -> EditorTheme {
    EditorTheme {
      cursor: EditorThemeCursorColor {
        color: blue(),
        selection_color: blue(),
      },
      code: EditorThemePairColor {
        bg_color: white(),
        text_color: opaque_grey(0.1, 1.0),
      },
      line_numbers: EditorThemePairColor {
        bg_color: white(),
        text_color: opaque_grey(0.3, 1.0),
      },
      git: EditorThemeGit {
        added: EditorThemeGitColor {
          line_bg_color: green().alpha(0.4),
          char_highlight_color: green().alpha(0.7),
          gutter_color: green().alpha(0.7),
        },
        removed: EditorThemeGitColor {
          line_bg_color: red().alpha(0.4),
          char_highlight_color: red(),
          gutter_color: red().alpha(0.7),
        },
        modified: EditorThemeGitColor {
          line_bg_color: Hsla {
            h: 35.0,
            s: 1.0,
            l: 0.25,
            a: 1.0,
          },
          char_highlight_color: Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
          },
          gutter_color: Hsla {
            h: 35.0,
            s: 1.0,
            l: 0.25,
            a: 1.0,
          },
        },
      },
    }
  }

  pub fn default_theme_dark() -> EditorTheme {
    EditorTheme {
      cursor: EditorThemeCursorColor {
        color: blue(),
        selection_color: blue(),
      },
      code: EditorThemePairColor {
        bg_color: black(),
        text_color: opaque_grey(0.9, 1.0),
      },
      line_numbers: EditorThemePairColor {
        bg_color: black(),
        text_color: opaque_grey(0.7, 1.0),
      },
      git: EditorThemeGit {
        added: EditorThemeGitColor {
          line_bg_color: green().alpha(0.8),
          char_highlight_color: green(),
          gutter_color: green().alpha(0.9),
        },
        removed: EditorThemeGitColor {
          line_bg_color: red().alpha(0.5),
          char_highlight_color: red(),
          gutter_color: red().alpha(0.7),
        },
        modified: EditorThemeGitColor {
          line_bg_color: Hsla {
            h: 35.0,
            s: 1.0,
            l: 0.25,
            a: 1.0,
          },
          char_highlight_color: Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
          },
          gutter_color: Hsla {
            h: 35.0,
            s: 1.0,
            l: 0.25,
            a: 1.0,
          },
        },
      },
    }
  }

  pub fn get_theme(&self, is_dark_mode: bool) -> &EditorTheme {
    if is_dark_mode {
      &self.theme_dark
    } else {
      &self.theme_light
    }
  }
}
