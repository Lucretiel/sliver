#![no_std]
#![feature(const_float_classify)]
#![feature(const_float_bits_conv)]

mod angle;
mod consts;
mod repr;
mod table;
mod trig;
mod sign;

pub use angle::Angle;
