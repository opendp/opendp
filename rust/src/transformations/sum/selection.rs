//! Contribution selection only; no numerical accumulation or error analysis.

use crate::{error::Fallible, traits::samplers::Shuffle};

// Exact-size members already have length `limit`. For insert-delete distance
// use a prefix; for symmetric distance use a uniform sample without replacement.
// Coupling occurrence priorities across a multiset edit path changes a selected
// set by at most one addition/removal/replacement per edit. This bounds the
// change in its ideal sum by max(|L|, |U|, U-L), independently of arithmetic.
pub(super) fn select_contributions<T: Clone>(
    arg: &[T],
    limit: usize,
    ordered: bool,
) -> Fallible<Vec<T>> {
    if arg.len() <= limit || ordered || limit == 0 {
        return Ok(arg[..arg.len().min(limit)].to_vec());
    }
    let mut values = arg.to_vec();
    values.shuffle()?;
    values.truncate(limit);
    Ok(values)
}
