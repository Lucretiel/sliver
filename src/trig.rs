use bitvec::{bitarr, field::BitField, prelude::Msb0, view::BitView};
use core::cmp::Ordering;

// TODO: Find a way to use the Repr type in this module. The basic problem is
// that we keep bouncing around between bit fiddling, CPU arithmetic, and
// "real" math (multiplications and stuff). For now we stick wit u64 and assume
// conversions at call boundaries.

use crate::consts::TAU;
use crate::repr::Repr;
use crate::sign::Sign;
use crate::table::CURVE;

#[derive(Debug, Clone, Copy)]
pub enum Output {
    One,
    Fraction(Repr),
}

// 24 bytes for ~9 bytes of information makes me cry :(
#[derive(Debug, Clone, Copy)]
pub struct SignedOutput {
    sign: Sign,
    value: Output,
}

impl SignedOutput {
    pub fn as_float(&self) -> f64 {
        let unsigned = match self.value {
            Output::One => 1.0,
            Output::Fraction(repr) => repr.as_float(),
        };

        match self.sign {
            Sign::Positive => unsigned,
            Sign::Negative => -unsigned,
        }
    }
}

/// Look up a sin value in the table, where repr represents a value in the range
/// [0..1/4) turns. Note that this can never return 1, because the max input
/// is *just* short of 1/4.
#[inline]
#[must_use]
const fn sin_exact(repr: u8) -> Repr {
    Repr::new(CURVE[repr as usize])
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

We do have to make some accommodations for the fact that our repr can't handle 1
(it's asymptotic towards 1), but this ends up being a straightforward degenerate
case (if A == 0, then sin(A) + b * cos(A) = 0 + b * 1 = b)
*/
#[must_use]
fn quarter_sin(repr: u64) -> Repr {
    let view = repr.view_bits::<Msb0>();

    let zone: u16 = view[2..10].load();
    let epsilon = Repr::new(view[10..].load());

    // For the most part, we're interested in the radians repr of the low_part,
    // which will be used as a product. Note that, while theoretically this
    // can be anywhere from 0..tau, in practice it'll always be well under that
    // (because this the epsilon). It's therefore only shifted 61 bits and is
    // in its correct 0..1 repr for future multiplies.
    let epsilon_radians = TAU.mul0(epsilon);

    // If the high_part is 0, then sin(A) == 0 and cos(A) == 1. This ends up
    // being a degenerate fallback to result repr (as radians).
    //
    // It would be nice for this to be branchless, and we sort of hope the
    // compiler can work out a set of operations to make it that way
    if zone == 0 {
        epsilon_radians
    } else {
        let sin_a = sin_exact(zone as u8);

        let cos_a = sin_exact((0x100u16 - zone) as u8);
        let b_cos_a = cos_a.mul(epsilon_radians);

        sin_a.saturating_add(b_cos_a)
    }
}

// TODO: find a way to use bit slices for basically all of this sin
// implementation. The problem mainly is the subtraction in step 2, which
// precludes the sort of easy bit slicing that we could otherwise use everywhere
// here.

// Assuming that repr represents a value in the range [0, 0.5) rotations, return
// the sin of that value. Returns `None` if the sin is precisely 1. This is
// computed by reflecting angles in the range (0.25, 0.5) to use quarter_sin.
#[inline]
#[must_use]
fn half_sin(repr: u64) -> Output {
    let repr: u64 = repr.view_bits::<Msb0>()[1..].load();

    let half: u64 = bitarr!(u64, Msb0; 1, 0, 0, 0).load();
    let quarter: u64 = bitarr!(u64, Msb0; 0, 1, 0, 0).load();

    let repr = match Ord::cmp(&repr, &quarter) {
        Ordering::Less => repr,
        Ordering::Equal => return Output::One,
        Ordering::Greater => half - repr,
    };

    Output::Fraction(quarter_sin(repr))
}

/// Assuming that repr represents a value in the range [0, 1) rotations, return
/// the sin of that value. Returns `None` if the sin is precisely 1. This is
/// computed by reflecting angles in the range (0.5, 1) to be the negated version
/// of the half sin.
#[inline]
#[must_use]
pub fn sin(repr: u64) -> SignedOutput {
    let view = repr.view_bits::<Msb0>();

    let sign = match view[0] {
        false => Sign::Positive,
        true => Sign::Negative,
    };

    SignedOutput {
        sign,
        value: half_sin(repr),
    }
}
