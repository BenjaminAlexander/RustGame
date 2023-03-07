#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct TimerId {
    index: usize
}

impl TimerId {
    pub fn new(index: usize) -> Self {
        return Self {
            index
        };
    }
}