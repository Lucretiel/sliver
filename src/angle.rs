use core::{
    num::FpCategory,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use crate::{
    consts,
    repr::{BaseRepr, Repr},
};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Angle(Repr);

impl Angle {
    /// Create a new float from the representational format. The representation
    /// here is a fractional value in the range [0, 1), filling the full space
    /// of a u64.
    pub const fn from_repr(repr: u64) -> Self {
        Self(Repr::new(repr))
    }

    /// Create a new float from a fractional number of rotations. This will
    /// be `const` when the relevant float methods are stabilized.
    pub const fn from_rotations(rotations: f64) -> Option<Self> {
        match Repr::from_float(rotations) {
            Some(repr) => Some(Self(repr)),
            None => None,
        }
    }

    pub const fn from_radians(radians: f64) -> Self {
        todo!()
    }

    pub const fn from_degrees(degrees: f64) -> Self {
        todo!()
    }

    pub const fn as_rotations(self) -> f64 {
        self.0.as_float()
    }

    pub const fn as_radians(self) -> f64 {
        consts::TAU.mul(self.0).as_float()
    }

    pub const fn as_degrees(self) -> f64 {
        // TODO: We don't need to use a u128 if there are 9 or more leading 0
        // bits. We assume for now that the branch isn't worth it, especially
        // since typical use cases won't be such tiny fractions of a degree.

        let widened = self.0.as_repr() as u128;
        let degrees = widened * 360;
        let shifted = degrees >> 9;
        debug_assert!(shifted.leading_zeros() >= 64, "overflowed a u64 somehow");
        BaseRepr::<9>::new(shifted as u64).as_float()
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::Angle;

    #[test]
    fn test_half_rotations() {
        let angle = Angle::from_repr(0x80_00_00_00_00_00_00_00);
        assert_eq!(angle.as_rotations(), 0.5)
    }

    #[test]
    fn test_half_degrees() {
        let angle = Angle::from_repr(0x80_00_00_00_00_00_00_00);
        assert_eq!(angle.as_degrees(), 180.0)
    }

    #[test]
    fn test_half_radians() {
        let angle = Angle::from_repr(0x80_00_00_00_00_00_00_00);
        assert_eq!(angle.as_radians(), core::f64::consts::PI)
    }
}
