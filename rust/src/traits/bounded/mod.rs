/// Consts representing the maximum and minimum finite representable values.
pub trait FiniteBounds {
    /// # Proof Definition
    /// `Self::MAX_FINITE` is the largest permissible finite value that can be represented by `Self`.
    const MAX_FINITE: Self;
    /// # Proof Definition
    /// `Self::MIN_FINITE` is the smallest permissible finite value that can be represented by `Self`.
    const MIN_FINITE: Self;
}
macro_rules! impl_finite_bounds {
    ($($ty:ty)+) => ($(impl FiniteBounds for $ty {
        const MAX_FINITE: Self = Self::MAX;
        const MIN_FINITE: Self = Self::MIN;
    })+)
}
impl_finite_bounds!(f64 f32 i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
