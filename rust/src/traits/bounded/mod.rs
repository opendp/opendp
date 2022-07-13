/// Consts representing the maximum and minimum finite representable values.
pub trait FiniteBounds {
    const MAX_FINITE: Self;
    const MIN_FINITE: Self;
}
macro_rules! impl_finite_bounds {
    ($($ty:ty)+) => ($(impl FiniteBounds for $ty {
        const MAX_FINITE: Self = Self::MAX;
        const MIN_FINITE: Self = Self::MIN;
    })+)
}
impl_finite_bounds!(f64 f32 i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);

