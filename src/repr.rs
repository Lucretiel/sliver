use core::num::FpCategory;

use crate::consts;

// TODO: We keep going back and forth on whether this type is needed. It is
// nice to be able to keep track of the range of a repr with `const O`, but in
// practice it somehow keeps *interfering* with the math we want to do.
/// A BaseRepr is essentially fixed-precision value in the range [0..2^O).
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
    /// a normal finite number.
    #[must_use]
    pub const fn from_float(value: f64) -> Option<Self> {
        let value = match value.classify() {
            FpCategory::Zero => return Some(Self(0)),
            FpCategory::Normal => value,
            FpCategory::Nan | FpCategory::Infinite | FpCategory::Subnormal => return None,
        };

        let float_repr = value.to_bits();
        let value_preshift = (float_repr & consts::MANTISSA_MASK) | consts::EXTRA_FLOAT_BIT;
        let exp = (float_repr >> 52) & consts::EXP_MASK;
        let exp_difference = (exp as i32) - consts::FLOAT_ZERO_EXP + 12 - O;

        Some(Self(if exp_difference.is_negative() {
            value_preshift >> exp_difference.abs()
        } else {
            value_preshift << exp_difference
        }))
    }

    #[inline]
    #[must_use]
    pub const fn as_repr(self) -> u64 {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn mul(self, other: Repr) -> Self {
        Self((((self.0 as u128) * (other.0 as u128)) >> 64) as u64)
    }

    pub const fn mul0(self, other: Repr) -> Repr {
        Repr::new((((self.0 as u128) * (other.0 as u128)) >> (64 - O)) as u64)
    }

    #[inline]
    #[must_use]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    #[inline]
    #[must_use]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    #[must_use]
    pub const fn as_float(self) -> f64 {
        let repr = self.0;

        if repr == 0 {
            return 0.0;
        }

        let zeroes = repr.leading_zeros() as i32;
        let precision_adjustment = zeroes + 1;

        let computed_exp = consts::FLOAT_ZERO_EXP - precision_adjustment + O;

        let mantissa = repr >> (12i32 - precision_adjustment);
        let mantissa = mantissa & consts::MANTISSA_MASK;
        let shifted_exp = (computed_exp as u64) << 52;

        let float_repr = mantissa | shifted_exp;

        f64::from_bits(float_repr)
    }

    #[must_use]
    #[inline]
    pub const fn as_repr0(self) -> Repr {
        Repr::new(if O.is_negative() {
            self.0 >> O.abs()
        } else {
            self.0 << O
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
    fn one_point_five_overflow() {
        let repr = Repr::from_float(1.5).unwrap();
        assert_eq!(repr.0, 0x80_00_00_00_00_00_00_00)
    }

    #[test]
    fn shifted() {
        let repr = BaseRepr::<1>::from_float(1.5).unwrap();
        assert_eq!(repr.0, 0xC0_00_00_00_00_00_00_00)
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
