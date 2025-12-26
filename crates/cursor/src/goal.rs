#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum CursorGoal {
    #[default]
    None,
    /// The column position we want to maintain when moving up/down
    Column(usize),
}
