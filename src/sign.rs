#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Positive,
    Negative,
}

use Sign::{Negative, Positive};

impl Sign {
    #[inline]
    #[must_use]
    pub const fn from_bit(bit: bool) -> Self {
        match bit {
            false => Positive,
            true => Negative,
        }
    }
}
