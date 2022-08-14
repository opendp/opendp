mod via_postprocess;
pub use via_postprocess::*;

#[cfg(feature = "floating-point")]
mod via_lipschitz;
#[cfg(feature = "floating-point")]
pub use via_lipschitz::*;

use crate::error::Fallible;

fn check_parameters(strat_sizes: &Vec<usize>, sample_sizes: &Vec<usize>) -> Fallible<()> {
    if strat_sizes.len() != sample_sizes.len() {
        return fallible!(
            MakeTransformation,
            "strat sizes and sample sizes must share the same length"
        );
    }

    if strat_sizes.len() == 0 {
        return fallible!(MakeTransformation, "must have at least one partition");
    }

    if (strat_sizes.iter())
        .zip(sample_sizes.iter())
        .any(|(a, b)| a < b)
    {
        return fallible!(
            MakeTransformation,
            "strat sizes may not be smaller than sample sizes"
        );
    }

    if sample_sizes.iter().any(|&s| s == 0) {
        return fallible!(MakeTransformation, "partitions must be non-empty");
    }

    Ok(())
}
