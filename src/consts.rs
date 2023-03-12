use crate::repr::BaseRepr;

/// The value of TAU in BaseRpr form
#[allow(clippy::unusual_byte_groupings)]
pub const TAU: BaseRepr<3> = BaseRepr::new(0xC90F_DAA2_2168_C234);

/// 360 in `BaseRepr` form
pub const DEGREES: BaseRepr<9> = BaseRepr::new(0xB400_0000_0000_0000);
