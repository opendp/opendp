mod checked;
pub use checked::*;

mod monotonic;
pub use monotonic::*;

mod ordered;
pub use ordered::*;

mod split;
pub use split::*;

pub trait AddIsExact {}
macro_rules! impl_addition_is_exact {
    ($($ty:ty)+) => ($(impl AddIsExact for $ty {})+)
}
impl_addition_is_exact! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }
