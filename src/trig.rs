use core::cmp::Ordering;
use core::slice::RChunks;
use Ordering::*;

use crate::consts::{HIGH_BYTE, HIGH_OFFSET, LOW_BYTES, TAU};
use crate::table::CURVE;

#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub enum Output {
    One,
    Fraction(u64),
}

/*
Assuming that repr represents a value from 0 to 1/4 turns, use it to compute
the sin.
*/

#[inline]
#[must_use]
const fn sin_exact(repr: u8) -> u64 {
    CURVE[repr as usize]
}

const fn repr_mul(lhs: u64, rhs: u64, shift: i32) -> u64 {
    (((lhs as u128) * (rhs as u128)) >> (64i32 - shift)) as u64
}

/*
Assuming that repr represents a value in the range [0, 0.25) rotations, return
the sin of that value.

We assume the identity:

sin(A + b) == sin(A)cos(b) + cos(A)sin(b)

Where b is the epsilon, and therefore very close to 0, we assume:
sin(b) ~= b (after converting to radians)
cos(b) ~= 1

Therefore:

sin(A + b) ~= sin(A) + b * cos(A)

Additionally:

cos(A) = sin(A + 1/4 rot)

Therefore:

sin(A + b) ~= sin(A) + b * sin(A + 1/4 rot)

We do have to make some accomodations for the fact that our repr can't handle 1
(it's asymptotic towards 1), but this ends up being a straightforward degenerate
case (if A == 0, then sin(A) + b * cos(A) = 0 + b * 1 = b)
*/
#[must_use]
const fn quarter_sin(repr: u64) -> u64 {
    let high_part = (repr >> 54) & 0xFF;
    let low_part = repr & 0x003F_FFFF_FFFF_FFFF;

    // For the most part, we're interested in the radians repr of the low_part,
    // which will be used as a product. Note that, while theoretically this
    // can be anywhere from 0..tau, in practice it'll always be well under that
    // (because this the epsilon). It's therefore only shifted 61 bits and is
    // in its correct 0..1 repr for future multiplies.
    let low_part_radians = repr_mul(low_part, TAU.as_repr(), 3);

    // If the high_part is 0, then sin(A) == 0 and cos(A) == 1. This ends up
    // being a degenerate fallback to result repr (as radians).
    //
    // It would be nice for this to be branchless, and we sort of hope the
    // compiler can work out a set of operations to make it that way
    if high_part == 0 {
        low_part_radians
    } else {
        let sin_a = sin_exact(high_part as u8);

        let cos_a = sin_exact((0x100u64 - high_part) as u8);
        let b_cos_a = repr_mul(cos_a, low_part_radians, 0);

        sin_a.saturating_add(b_cos_a)
    }
}

// Assuming that repr represents a value in the range [0, 0.5) rotations, return
// the sin of that value. Returns `None` if the sin is precisely 1. This is
// computed by reflecting angles in the range (0.25, 0.5) to use quarter_sin.
#[inline]
#[must_use]
const fn half_sin(repr: u64) -> Output {
    let repr = repr & 0x7FFF_FFFF_FFFF_FFFFu64;

    const ONE_QUARTER: u64 = 0x4000_0000_0000_0000;
    const ONE_HALF: u64 = 0x8000_0000_0000_0000;

    let adjusted_repr = if repr < ONE_QUARTER {
        repr
    } else if repr == ONE_QUARTER {
        return Output::One;
    } else {
        ONE_HALF - repr
    };

    Output::Fraction(quarter_sin(adjusted_repr))
}

// Assuming that repr represents a value in the range [0, 1) rotations, return
// the sin of that value. Returns `None` if the sin is precisely 1. This is
// computed by reflecting angles in the range (0.5, 1) to be the negated version
// of the half sin.
#[inline]
#[must_use]
pub const fn sin(repr: u64) -> (Sign, Output) {
    const MAX_BIT: u64 = 1 << 63;

    let sign = if repr < MAX_BIT {
        Sign::Positive
    } else {
        Sign::Negative
    };

    (sign, half_sin(repr))
}

#[inline]
pub const fn cos(repr: u64) -> (Sign, Output) {
    sin(repr.wrapping_add(0x4000_0000_0000_0000))
}
