use gpui::ShapedLine;
use std::collections::{HashMap, HashSet};

/// Granular cache for shaped lines
/// Allows invalidating only modified lines instead of recalculating everything
#[derive(Default)]
pub struct LineCache {
  /// Map: line_idx → ShapedLine
  pub shaped_lines: HashMap<usize, ShapedLine>,
  pub buffer_version: usize,
  pub dirty_lines: HashSet<usize>,
}

impl LineCache {
  pub fn new() -> Self {
    Self {
      shaped_lines: HashMap::new(),
      buffer_version: 0,
      dirty_lines: HashSet::new(),
    }
  }

  /// Retrieves a line from cache, or None if not present
  pub fn get(&self, line_idx: usize) -> Option<&ShapedLine> {
    if self.dirty_lines.contains(&line_idx) {
      return None;
    }
    self.shaped_lines.get(&line_idx)
  }

  /// Inserts a line into the cache
  pub fn insert(&mut self, line_idx: usize, shaped: ShapedLine) {
    self.shaped_lines.insert(line_idx, shaped);
    self.dirty_lines.remove(&line_idx);
  }

  /// Marks a line as dirty (needs reshaping)
  pub fn mark_dirty(&mut self, line_idx: usize) {
    self.dirty_lines.insert(line_idx);
  }

  /// Marks a range of lines as dirty
  pub fn mark_dirty_range(&mut self, start: usize, end: usize) {
    for line_idx in start..=end {
      self.dirty_lines.insert(line_idx);
    }
  }

  /// Clears the entire cache (if buffer version changes drastically)
  pub fn clear(&mut self) {
    self.shaped_lines.clear();
    self.dirty_lines.clear();
  }

  /// Checks if buffer has changed and clears if necessary
  pub fn check_buffer_version(&mut self, current_version: usize) -> bool {
    if self.buffer_version != current_version {
      // Buffer has changed, clear everything
      // Note: in a more sophisticated version, we could
      // try to preserve some lines
      self.clear();
      self.buffer_version = current_version;
      true
    } else {
      false
    }
  }

  /// Returns the number of cached lines
  pub fn len(&self) -> usize {
    self.shaped_lines.len()
  }

  /// Checks if the cache is empty
  pub fn is_empty(&self) -> bool {
    self.shaped_lines.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_line_cache_new() {
    let cache = LineCache::new();
    assert_eq!(cache.buffer_version, 0);
    assert!(cache.is_empty());
    assert_eq!(cache.dirty_lines.len(), 0);
  }

  #[test]
  fn test_mark_dirty() {
    let mut cache = LineCache::new();
    cache.mark_dirty(5);
    assert!(cache.dirty_lines.contains(&5));
  }

  #[test]
  fn test_mark_dirty_range() {
    let mut cache = LineCache::new();
    cache.mark_dirty_range(10, 15);
    for i in 10..=15 {
      assert!(cache.dirty_lines.contains(&i));
    }
  }

  #[test]
  fn test_dirty_lines_block_cache_retrieval() {
    let mut cache = LineCache::new();

    cache.mark_dirty(5);

    assert!(cache.get(5).is_none());
  }

  #[test]
  fn test_buffer_version_change_clears_cache() {
    let mut cache = LineCache::new();
    cache.buffer_version = 5;

    cache.dirty_lines.insert(1);
    cache.dirty_lines.insert(2);

    let changed = cache.check_buffer_version(10);
    assert!(changed);
    assert_eq!(cache.buffer_version, 10);
    assert_eq!(cache.dirty_lines.len(), 0);
    assert!(cache.is_empty());
  }

  #[test]
  fn test_buffer_version_no_change() {
    let mut cache = LineCache::new();
    cache.buffer_version = 5;

    let changed = cache.check_buffer_version(5);
    assert!(!changed);
    assert_eq!(cache.buffer_version, 5);
  }

  #[test]
  fn test_insert_removes_dirty_flag() {
    let mut cache = LineCache::new();
    cache.mark_dirty(3);

    assert!(cache.dirty_lines.contains(&3));

    assert!(cache.get(3).is_none());
  }

  #[test]
  fn test_clear() {
    let mut cache = LineCache::new();
    cache.mark_dirty(1);
    cache.mark_dirty(2);

    cache.clear();

    assert!(cache.is_empty());
    assert_eq!(cache.dirty_lines.len(), 0);
  }

  #[test]
  fn test_len() {
    let cache = LineCache::new();
    assert_eq!(cache.len(), 0);
  }
}
