use core::f64::consts as f64_consts;

use bitvec::{bitarr, field::BitField as _, order::Msb0};

use crate::{consts, repr::Repr, trig};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Angle(Repr);

impl Angle {
    /// Create a new float from the representational format. The representation
    /// here is a fractional value in the range [0, 1), filling the full space
    /// of a u64.

    #[inline]
    #[must_use]
    pub const fn from_repr(repr: u64) -> Self {
        Self(Repr::new(repr))
    }

    /// Create a new float from a fractional number of rotations.

    #[inline]
    #[must_use]
    pub fn from_rotations(rotations: f64) -> Option<Self> {
        Repr::from_float(rotations).map(Self)
    }

    #[inline]
    #[must_use]
    pub fn from_radians(radians: f64) -> Option<Self> {
        // TODO: find some way with integer division. Maybe a shift-left into
        // an i128, then divide.
        //
        // We have repr::div, but we need a reliable way to convert radians
        // to float without truncation
        Self::from_rotations(radians / f64_consts::TAU)
    }

    #[inline]
    #[must_use]
    pub fn from_degrees(degrees: f64) -> Option<Self> {
        Self::from_rotations(degrees / 360.0)
    }

    /// Get a lossless representation of this angle as an unsigned integer.

    #[inline]
    #[must_use]
    pub fn repr(self) -> u64 {
        self.0 .0
    }

    #[inline]
    #[must_use]
    pub fn as_rotations(self) -> f64 {
        self.0.as_float()
    }

    #[inline]
    #[must_use]
    pub fn as_radians(self) -> f64 {
        consts::TAU.mul(self.0).as_float()
    }

    #[inline]
    #[must_use]
    pub fn as_degrees(self) -> f64 {
        consts::DEGREES.mul(self.0).as_float()
    }

    #[inline]
    #[must_use]
    pub fn sin(self) -> f64 {
        trig::sin(self.repr()).as_float()
    }

    #[inline]
    #[must_use]
    pub fn cos(self) -> f64 {
        let quarter: u64 = bitarr!(u64, Msb0; 0, 1, 0, 0).load();
        trig::sin(self.repr().wrapping_add(quarter)).as_float()
    }

    #[inline]
    #[must_use]
    pub fn tan(self) -> f64 {
        self.sin() / self.cos()
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
