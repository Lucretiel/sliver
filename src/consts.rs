use crate::repr::BaseRepr;

/// The value of TAU, shifted left to fill a u64. The first 3 bits are the
/// integer component (not that it really matters in practice).
///
/// For math purposes, this is tau / 8, or pi / 4.
#[allow(clippy::unusual_byte_groupings)]
pub const TAU: BaseRepr<3> = BaseRepr::new(
    0b110_0_1001_0000_1111_1101_1010_1010_0010_0010_0001_0110_1000_1100_0010_0011_0100,
);

/// The value of the exponent bits equivelent to `2^0`. Modify this based on
/// the computed exponent, then shift this to the left 52 bits.
pub const FLOAT_ZERO_EXP: i32 = 0x03_FF;

/// Mask for the 52 bits of the mantissa of a float
pub const MANTISSA_MASK: u64 = (1u64 << 52) - 1;

/// Mask for the 11 bits of the exponent of the float (after they've been shifted left)
pub const EXP_MASK: u64 = (1u64 << 11) - 1;

/// The 1 bit that floats chop off
pub const EXTRA_FLOAT_BIT: u64 = 1u64 << 52;

/// The most significant byte in a u64
pub const HIGH_BYTE: u64 = 0xFF00_0000_0000_0000;

/// The bit offset of the high byte in a u64
pub const HIGH_OFFSET: u64 = HIGH_BYTE.trailing_zeros() as u64;

/// The 7 low bytes in a u64
pub const LOW_BYTES: u64 = !HIGH_BYTE;

/// The most significant bit in a u64
pub const HIGH_BIT: u64 = 1 << 63;
