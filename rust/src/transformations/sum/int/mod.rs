mod checked;
pub use checked::*;

mod monotonic;
pub use monotonic::*;

mod ordered;
pub use ordered::*;

mod split;
pub use split::*;

use crate::traits::Integer;

/// # Proof Definition
/// Returns true if, given data with `size` records, each of which has data bounded between `lower` and `upper`,
/// the true sum of the records exceeds the greatest value representable of type `T`.
pub(crate) fn can_int_sum_overflow<T: Integer>(size: usize, (lower, upper): (T, T)) -> bool {
    (|| {
        let size = T::exact_int_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;
        mag.inf_mul(&size).map(|_| ())
    })()
    .is_err()
}
