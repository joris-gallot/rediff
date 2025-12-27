use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
  Unchanged,
  Added,
  Removed,
  Modified, // A pair of removed + added lines
}

#[derive(Debug, Clone)]
pub struct CharRange {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
  pub line_number: usize, // 0 means no line number (for removed lines in modified pairs)
  pub kind: DiffLineKind,
  pub content: String,
  pub char_changes: Vec<CharRange>, // Highlighted character ranges for intra-line diff
  pub is_first_in_group: bool,      // True if this is the first line in a modification group
}

pub struct Differ {
  original: String,
}

impl Differ {
  pub fn new(original: String) -> Self {
    Self { original }
  }

  pub fn compute_diff(&self, modified: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(self.original.as_str(), modified);

    let mut result = Vec::new();
    let mut line_number = 0;
    let mut pending_removes: Vec<String> = Vec::new();
    let mut pending_adds: Vec<String> = Vec::new();

    for change in diff.iter_all_changes() {
      match change.tag() {
        ChangeTag::Equal => {
          Self::flush_pending(
            &mut result,
            &mut line_number,
            &mut pending_removes,
            &mut pending_adds,
          );

          line_number += 1;
          result.push(DiffLine {
            line_number,
            kind: DiffLineKind::Unchanged,
            content: change.to_string(),
            char_changes: vec![],
            is_first_in_group: false,
          });
        }
        ChangeTag::Delete => {
          pending_removes.push(change.to_string());
        }
        ChangeTag::Insert => {
          pending_adds.push(change.to_string());
        }
      }
    }

    Self::flush_pending(
      &mut result,
      &mut line_number,
      &mut pending_removes,
      &mut pending_adds,
    );

    // Ensure all lines from the modified buffer are represented
    // Use split('\n') to correctly count all lines including empty ones
    let modified_lines: Vec<&str> = if modified.is_empty() {
      vec![""]
    } else {
      modified.split('\n').collect()
    };

    let modified_line_count = modified_lines.len();

    while line_number < modified_line_count {
      line_number += 1;
      let line_content = modified_lines.get(line_number - 1).unwrap_or(&"");
      result.push(DiffLine {
        line_number,
        kind: DiffLineKind::Unchanged,
        content: format!("{}\n", line_content),
        char_changes: vec![],
        is_first_in_group: false,
      });
    }

    result
  }

  fn flush_pending(
    result: &mut Vec<DiffLine>,
    line_number: &mut usize,
    pending_removes: &mut Vec<String>,
    pending_adds: &mut Vec<String>,
  ) {
    let remove_count = pending_removes.len();
    let add_count = pending_adds.len();

    // Process based on similarity, not just count matching
    if remove_count > 0 && add_count > 0 {
      let removes_to_process = pending_removes.clone();
      let adds_to_process = pending_adds.clone();
      pending_removes.clear();
      pending_adds.clear();

      // Try to match similar lines for modifications
      let mut matched_pairs = Vec::new();
      let mut processed_removes = vec![false; removes_to_process.len()];
      let mut processed_adds = vec![false; adds_to_process.len()];

      for i in 0..removes_to_process.len() {
        if processed_removes[i] {
          continue;
        }

        let mut best_match_idx = None;
        let mut best_similarity = 0.3; // Minimum 30% similarity threshold

        for j in 0..adds_to_process.len() {
          if processed_adds[j] {
            continue;
          }

          let similarity = Self::calculate_similarity(&removes_to_process[i], &adds_to_process[j]);
          if similarity > best_similarity {
            best_similarity = similarity;
            best_match_idx = Some(j);
          }
        }

        if let Some(j) = best_match_idx {
          // Found a similar line - treat as modification
          processed_removes[i] = true;
          processed_adds[j] = true;
          matched_pairs.push((i, j));
        }
      }

      // Check if ALL lines are matched (1:1 perfect pairing)
      let all_matched = matched_pairs.len() == remove_count && matched_pairs.len() == add_count;

      if all_matched {
        // All lines are paired - show as modifications
        let mut is_first_modification = true;
        for (i, j) in matched_pairs {
          *line_number += 1;

          let removed_content = &removes_to_process[i];
          let added_content = &adds_to_process[j];

          let (removed_ranges, added_ranges) =
            Self::compute_intra_line_diff(removed_content, added_content);

          result.push(DiffLine {
            line_number: 0,
            kind: DiffLineKind::Modified,
            content: removed_content.clone(),
            char_changes: removed_ranges,
            is_first_in_group: is_first_modification,
          });

          result.push(DiffLine {
            line_number: *line_number,
            kind: DiffLineKind::Modified,
            content: added_content.clone(),
            char_changes: added_ranges,
            is_first_in_group: false,
          });

          is_first_modification = false;
        }
      } else {
        // Not all matched - treat entire block as removes then adds
        let mut first_remove = true;
        for removed in removes_to_process.iter() {
          result.push(DiffLine {
            line_number: 0,
            kind: DiffLineKind::Removed,
            content: removed.clone(),
            char_changes: vec![],
            is_first_in_group: first_remove,
          });
          first_remove = false;
        }

        let mut first_add = true;
        for added in adds_to_process.iter() {
          *line_number += 1;
          result.push(DiffLine {
            line_number: *line_number,
            kind: DiffLineKind::Added,
            content: added.clone(),
            char_changes: vec![],
            is_first_in_group: first_add,
          });
          first_add = false;
        }
      }
    } else {
      // Only removes or only adds
      let is_first_remove = !pending_removes.is_empty();
      for (i, removed) in pending_removes.drain(..).enumerate() {
        result.push(DiffLine {
          line_number: 0,
          kind: DiffLineKind::Removed,
          content: removed,
          char_changes: vec![],
          is_first_in_group: is_first_remove && i == 0,
        });
      }

      let is_first_add = !pending_adds.is_empty();
      for (i, added) in pending_adds.drain(..).enumerate() {
        *line_number += 1;
        result.push(DiffLine {
          line_number: *line_number,
          kind: DiffLineKind::Added,
          content: added,
          char_changes: vec![],
          is_first_in_group: is_first_add && i == 0,
        });
      }
    }
  }

  fn calculate_similarity(a: &str, b: &str) -> f32 {
    let a_trimmed = a.trim();
    let b_trimmed = b.trim();

    if a_trimmed == b_trimmed {
      return 1.0;
    }

    if a_trimmed.is_empty() || b_trimmed.is_empty() {
      return 0.0;
    }

    let a_chars: Vec<char> = a_trimmed.chars().collect();
    let b_chars: Vec<char> = b_trimmed.chars().collect();

    let min_len = a_chars.len().min(b_chars.len());
    let max_len = a_chars.len().max(b_chars.len());

    let mut common_chars = 0;
    for i in 0..min_len {
      if a_chars[i] == b_chars[i] {
        common_chars += 1;
      }
    }

    common_chars as f32 / max_len as f32
  }

  fn compute_intra_line_diff(old: &str, new: &str) -> (Vec<CharRange>, Vec<CharRange>) {
    let diff = TextDiff::from_chars(old, new);
    let mut old_ranges = Vec::new();
    let mut new_ranges = Vec::new();
    let mut old_pos = 0;
    let mut new_pos = 0;

    for change in diff.iter_all_changes() {
      let len = change.value().len();
      match change.tag() {
        ChangeTag::Equal => {
          old_pos += len;
          new_pos += len;
        }
        ChangeTag::Delete => {
          old_ranges.push(CharRange {
            start: old_pos,
            end: old_pos + len,
          });
          old_pos += len;
        }
        ChangeTag::Insert => {
          new_ranges.push(CharRange {
            start: new_pos,
            end: new_pos + len,
          });
          new_pos += len;
        }
      }
    }

    (old_ranges, new_ranges)
  }

  pub fn update_original(&mut self, new_original: String) {
    self.original = new_original;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_differ_no_changes() {
    let differ = Differ::new("Hello\nWorld".to_string());
    let diff = differ.compute_diff("Hello\nWorld");
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|line| line.kind == DiffLineKind::Unchanged));
  }

  #[test]
  fn test_differ_added_line() {
    let differ = Differ::new("Hello\nWorld".to_string());
    let diff = differ.compute_diff("Hello\nNew Line\nWorld");
    let added = diff
      .iter()
      .filter(|line| line.kind == DiffLineKind::Added)
      .count();
    assert_eq!(added, 1);
  }

  #[test]
  fn test_differ_removed_line() {
    let differ = Differ::new("Hello\nRemove Me\nWorld".to_string());
    let diff = differ.compute_diff("Hello\nWorld");
    let removed = diff
      .iter()
      .filter(|line| line.kind == DiffLineKind::Removed)
      .count();
    assert_eq!(removed, 1);
  }

  #[test]
  fn test_differ_update_original() {
    let mut differ = Differ::new("Original".to_string());
    differ.update_original("New Original".to_string());
    let diff = differ.compute_diff("New Original");
    assert!(diff.iter().all(|line| line.kind == DiffLineKind::Unchanged));
  }

  #[test]
  fn test_differ_modified_line() {
    let differ = Differ::new("Hello World".to_string());
    let diff = differ.compute_diff("Hello Universe");
    // Should have 2 lines: removed and added as Modified
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|line| line.kind == DiffLineKind::Modified));
  }

  #[test]
  fn test_intra_line_diff() {
    let (old_ranges, new_ranges) = Differ::compute_intra_line_diff("Hello World", "Hello Universe");
    assert!(!old_ranges.is_empty());
    assert!(!new_ranges.is_empty());
  }

  #[test]
  fn test_dissimilar_lines_as_separate_changes() {
    let differ = Differ::new("<div class=\"wrapper\">\n<TheWelcome />".to_string());
    let diff = differ.compute_diff("<HelloWorld msg=\"test\" />\n<adazd />");

    // Should have: 2 removed lines + 2 added lines = 4 lines
    // Find the removed and added lines (ignoring any trailing empty lines)
    let removed_lines: Vec<_> = diff
      .iter()
      .filter(|l| l.kind == DiffLineKind::Removed)
      .collect();
    let added_lines: Vec<_> = diff
      .iter()
      .filter(|l| l.kind == DiffLineKind::Added)
      .collect();

    assert_eq!(removed_lines.len(), 2, "Should have 2 removed lines");
    assert_eq!(added_lines.len(), 2, "Should have 2 added lines");

    // Verify removed lines have no line number
    assert_eq!(removed_lines[0].line_number, 0);
    assert_eq!(removed_lines[1].line_number, 0);

    // Verify added lines have line numbers
    assert_eq!(added_lines[0].line_number, 1);
    assert_eq!(added_lines[1].line_number, 2);

    // Verify order: all removes come before all adds
    let first_remove_idx = diff
      .iter()
      .position(|l| l.kind == DiffLineKind::Removed)
      .unwrap();
    let last_remove_idx = diff
      .iter()
      .rposition(|l| l.kind == DiffLineKind::Removed)
      .unwrap();
    let first_add_idx = diff
      .iter()
      .position(|l| l.kind == DiffLineKind::Added)
      .unwrap();

    assert!(
      last_remove_idx < first_add_idx,
      "All removes should come before adds"
    );
  }

  #[test]
  fn test_similarity_calculation() {
    // Identical lines
    assert_eq!(Differ::calculate_similarity("hello", "hello"), 1.0);

    // Very similar lines
    let sim = Differ::calculate_similarity("<div class=\"container\">", "<div class=\"wrapper\">");
    assert!(sim > 0.5, "Similar lines should have > 50% similarity");

    // Very different lines
    let sim = Differ::calculate_similarity("<main>", "<TheWelcome />");
    assert!(sim < 0.3, "Different lines should have < 30% similarity");
  }

  #[test]
  fn test_mixed_modifications_and_pure_changes() {
    let differ = Differ::new("line1\nold line\nline3\n".to_string());
    let diff = differ.compute_diff("line1\nnew line\nline3\n");

    // Should recognize "old line" -> "new line" as modification (similar)
    let modified_lines: Vec<_> = diff
      .iter()
      .filter(|l| l.kind == DiffLineKind::Modified)
      .collect();
    assert_eq!(modified_lines.len(), 2); // One removed + one added = modification pair
  }

  #[test]
  fn test_order_removes_before_adds() {
    let differ = Differ::new("A\nB\nC\n".to_string());
    let diff = differ.compute_diff("X\nY\nZ\n");

    // All lines are different, should be: 3 removes then 3 adds
    let mut removes_done = false;
    let mut adds_started = false;

    for line in &diff {
      if line.kind == DiffLineKind::Added {
        removes_done = true;
        adds_started = true;
      }
      if line.kind == DiffLineKind::Removed && adds_started {
        panic!("Removed line found after added lines!");
      }
    }

    assert!(
      removes_done,
      "Should have processed all removes before adds"
    );
  }
}
