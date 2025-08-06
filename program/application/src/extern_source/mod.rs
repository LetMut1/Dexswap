// All code here anf in this modules copied from source and slightly refactored in context of Error type.
pub mod meteora_v1;
pub mod raydium_v4;
pub trait CheckedCeilDiv: Sized {
    fn checked_ceil_div(&self, rhs: Self) -> Option<(Self, Self)>;
}
