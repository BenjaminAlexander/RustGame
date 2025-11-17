#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TimerId {
    index: usize,
}

impl TimerId {
    pub(super) fn new(index: usize) -> Self {
        return Self { index };
    }
}
