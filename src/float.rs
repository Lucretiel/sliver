use bitvec::{
    field::BitField,
    order::Lsb0,
    prelude::{BitArray, BitOrder, Msb0},
    slice::BitSlice,
    view::{BitView, BitViewSized},
    BitArr,
};

pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FloatBits {
    repr: BitArray<u64, Lsb0>,
}

impl FloatBits {
    pub fn new(value: f64) -> Self {
        Self {
            repr: value.to_bits().into_bitarray(),
        }
    }

    pub fn to_float(&self) -> f64 {
        f64::from_bits(self.repr.load())
    }

    pub fn raw_mantissa(&self) -> &BitSlice<u64, Lsb0> {
        &self.repr[..52]
    }

    pub fn set_raw_mantissa(&mut self, bits: &BitSlice<u64, O>) {
        self.repr[..52].copy_from_bitslice(bits)
    }

    pub fn raw_exponent(&self) -> &BitSlice<u64, Lsb0> {
        &self.repr[52..63]
    }

    pub fn sign(&self) -> Sign {
        match self.repr[63] {
            false => Sign::Negative,
            true => Sign::Positive,
        }
    }

    /// Get the mantissa, and add the omitted leading 1 if the exponent is not
    /// zero. Note that this will produce senseless results in a NaN / Inf
    /// state.
    pub fn mantissa(&self) -> u64 {
        let mut raw: BitArray<u64, Lsb0> = self.raw_mantissa().try_into().unwrap();
        raw.set(52, true);
        raw.load()
    }

    pub fn exponent(&self) -> Option<i64> {
        let raw = self.raw_exponent();

        if raw.all() || raw.not_any() {
            None
        } else {
            let int: u64 = raw.load();
            Some((int as i64) - 1023)
        }
    }
}
