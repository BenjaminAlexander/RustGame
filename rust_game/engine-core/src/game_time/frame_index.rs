use serde::{
    Deserialize,
    Serialize,
};
use std::ops::{
    Add,
    Sub,
};

/// A FrameIndex is the index of a frame.  The game engine uses frames with
/// fixed amounts of time between them to compute the progression of the game's
///  state over time.  A FrameIndex is the index of one of these frames in the
/// sequence.
#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameIndex(usize);

impl FrameIndex {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn next(&self) -> FrameIndex {
        Self(self.0 + 1)
    }

    pub const fn previous(&self) -> FrameIndex {
        Self(self.0 + 1)
    }

    pub const fn usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for FrameIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for FrameIndex {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for &FrameIndex {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<f64> for FrameIndex {
    fn into(self) -> f64 {
        self.0 as f64
    }
}

impl Into<f64> for &FrameIndex {
    fn into(self) -> f64 {
        self.0 as f64
    }
}

impl Add<usize> for &FrameIndex {
    type Output = FrameIndex;

    fn add(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 + rhs)
    }
}

impl Add<usize> for FrameIndex {
    type Output = FrameIndex;

    fn add(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 + rhs)
    }
}

impl Sub<usize> for &FrameIndex {
    type Output = FrameIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 - rhs)
    }
}

impl Sub<usize> for FrameIndex {
    type Output = FrameIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 - rhs)
    }
}
