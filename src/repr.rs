use core::num::FpCategory;

use bitvec::{
    field::BitField,
    prelude::{BitArray, Lsb0, Msb0},
    view::BitView,
};

use crate::sign::Sign;

/// The value of the exponent bits equivalent to `2^0`.
const FLOAT_ZERO_EXP: i32 = 0x03_FF;

/// Fixed precision value. Represents a value in the range 0..2^O. Usually O
/// is 0 and this represents a value from 0 to 1, but we also use it to store
/// TAU (which has an exponent of 3).
///
/// This type probably has float conversion bugs when `O` is too high
/// or too low (outside of the representable range of exponent in an f64),
/// but it's not exported and we never use it that way, so it's fine.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BaseRepr<const O: i32>(pub u64);

impl<const O: i32> BaseRepr<O> {
    #[inline]
    #[must_use]
    pub const fn new(repr: u64) -> Self {
        Self(repr)
    }

    /// Create a new Repr value from a float. Returns None if the float isn't
    /// a normal finite number. Performs a modular truncation if the float is
    /// out of range (1.5 -> 0.5, -.25 => +.75).
    #[must_use]
    pub fn from_float(value: f64) -> Option<Self> {
        let value = match value.classify() {
            FpCategory::Zero => return Some(Self(0)),
            FpCategory::Normal => value,
            FpCategory::Nan | FpCategory::Infinite | FpCategory::Subnormal => return None,
        };

        let float_repr = value.to_bits();
        let float_repr = float_repr.view_bits::<Lsb0>();

        // The "true" mantissa of the float, including the omitted 1 bit
        // stored in the least significant 53
        let mantissa = {
            let mut mantissa: u64 = float_repr[..52].load();
            mantissa.view_bits_mut::<Lsb0>().set(52, true);
            mantissa
        };

        // The shift distance, based on the exponent in the float
        let shift_distance = {
            let raw_exponent = float_repr[52..63].load::<u32>() as i32;
            let exponent = raw_exponent - FLOAT_ZERO_EXP;
            exponent + 12 - O
        };

        // Perform the shift
        let fixed_point_repr = if shift_distance.is_negative() {
            mantissa >> shift_distance.abs()
        } else {
            mantissa << shift_distance
        };

        let sign = Sign::from_bit(float_repr[63]);

        // If the value is negative, perform a negation then 2's complement
        // cast. This turns out to do the right thing with regard to modular
        // arithmetic
        let sign_adjusted_repr = if matches!(sign, Sign::Positive) {
            fixed_point_repr
        } else {
            let repr = fixed_point_repr as i64;
            let repr = repr.wrapping_neg();
            repr as u64
        };

        Some(Self(sign_adjusted_repr))
    }

    /// Multiply a pair of `BaseRepr` values. One of them must be
    /// `BaseRepr<0>`.
    #[inline]
    #[must_use]
    pub const fn mul(self, other: Repr) -> Self {
        Self((((self.0 as u128) * (other.0 as u128)) >> 64) as u64)
    }

    /// Multiply a pair of `BaseRepr` values, one of which must be
    /// `BaseRepr<0>`, and return the result as a `BaseRepr<0>`.
    #[must_use]
    #[inline]
    pub const fn mul0(self, other: Repr) -> Repr {
        Repr::new((((self.0 as u128) * (other.0 as u128)) >> (64 - O)) as u64)
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Convert this `Repr` value to an `f64`, retaining as much precision as
    /// possible.
    #[must_use]
    pub fn as_float(self) -> f64 {
        let repr = self.0;
        let view = repr.view_bits::<Msb0>();

        let Some(one_idx) = view.first_one() else { return 0.0 };
        let mantissa = &view[one_idx + 1..];
        let mantissa = mantissa.get(..52).unwrap_or(mantissa);

        // Safety: one_idx in in 0..64, so it surely fits in an i32
        let exponent = O - 1 - (one_idx as i32);
        let biased_exponent = FLOAT_ZERO_EXP + exponent;

        f64::from_bits({
            let mut float_repr: BitArray<u64, Lsb0> = BitArray::ZERO;
            float_repr[..52].store(mantissa.load::<u64>());
            float_repr[52..63].store(biased_exponent);
            float_repr.load()
        })
    }
}

pub type Repr = BaseRepr<0>;

#[cfg(test)]
mod build_repr_tests {
    use super::{BaseRepr, Repr};

    #[test]
    fn half() {
        let repr = Repr::from_float(0.5).unwrap();
        assert_eq!(repr.0, 0x80_00_00_00_00_00_00_00)
    }

    #[test]
    fn three_quarters() {
        let repr = Repr::from_float(0.75).unwrap();
        assert_eq!(repr.0, 0xC0_00_00_00_00_00_00_00)
    }

    #[test]
    fn one_overflow() {
        let repr = Repr::from_float(1.0).unwrap();
        assert_eq!(repr.0, 0)
    }

    #[test]
    fn negative_overflow() {
        let repr = Repr::from_float(-1.5).unwrap();
        assert_eq!(repr.0, 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn negative_modular_overflow() {
        let repr = Repr::from_float(-1.75).unwrap();
        assert_eq!(repr.0, 0x40_00_00_00_00_00_00_00);
    }

    #[test]
    fn one_point_five_overflow() {
        let repr = Repr::from_float(1.5).unwrap();
        assert_eq!(repr.0, 0x80_00_00_00_00_00_00_00)
    }

    #[test]
    fn shifted() {
        let repr = BaseRepr::<1>::from_float(1.5).unwrap();
        assert_eq!(repr.0, 0xC0_00_00_00_00_00_00_00)
    }

    #[test]
    fn shifted_2() {
        let repr = BaseRepr::<2>::from_float(2.0).unwrap();
        assert_eq!(repr.0, 0x80_00_00_00_00_00_00_00)
    }
}

#[cfg(test)]
mod build_float_tests {
    use super::{BaseRepr, Repr};

    #[test]
    fn half() {
        let value = Repr::new(0x80_00_00_00_00_00_00_00);
        let float = value.as_float();
        assert_eq!(float, 0.5);
    }

    #[test]
    fn three_fourths() {
        let value = Repr::new(0xC0_00_00_00_00_00_00_00);
        let float = value.as_float();
        assert_eq!(float, 0.75);
    }

    #[test]
    fn quarter() {
        let value = Repr::new(0x40_00_00_00_00_00_00_00);
        let float = value.as_float();
        assert_eq!(float, 0.25);
    }

    #[test]
    fn two() {
        let value = BaseRepr::<2>::new(0x80_00_00_00_00_00_00_00);
        let float = value.as_float();
        assert_eq!(float, 2.0);
    }
}
