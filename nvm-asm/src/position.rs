// nvm-core/src/position.rs
//
//! Позиция токена в исходном коде.
use std::ops::Range;

/// Хранит плоские байтовые смещения от начала файла `[start..end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub start: u32,
    pub end: u32,
}

impl Position {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn as_range(&self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }

    pub fn to(&self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

// Позволяет писать:
// ```
// let range: Range<usize> = pos.into();
// ```
impl From<Position> for Range<usize> {
    fn from(pos: Position) -> Self {
        pos.as_range()
    }
}
