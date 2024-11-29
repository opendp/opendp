use crate::{
    error::Fallible,
    traits::samplers::{InverseCDF, PartialSample},
};
use dashu::float::FBig;
use num::Zero;
use std::fmt::Display;

#[cfg(feature = "contrib")]
mod gumbel;
#[cfg(feature = "contrib")]
pub use gumbel::*;

#[cfg(feature = "contrib")]
mod exponential;
#[cfg(feature = "contrib")]
pub use exponential::*;

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "polars", serde(rename_all = "lowercase"))]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Optimize::Min => f.write_str("min"),
            Optimize::Max => f.write_str("max"),
        }
    }
}

impl TryFrom<&str> for Optimize {
    type Error = crate::error::Error;
    fn try_from(s: &str) -> Fallible<Self> {
        Ok(match s {
            "min" => Optimize::Min,
            "max" => Optimize::Max,
            _ => return fallible!(FailedCast, "optimize must be \"min\" or \"max\""),
        })
    }
}

pub(crate) trait ArgmaxRV: InverseCDF + Sized {
    fn new(shift: FBig, scale: FBig) -> Fallible<Self>;
}

pub(crate) fn select_score<TIA, RV: ArgmaxRV>(
    iter: impl Iterator<Item = TIA>,
    optimize: Optimize,
    scale: FBig,
) -> Fallible<usize>
where
    TIA: PartialOrd,
    FBig: TryFrom<TIA>,
{
    if scale.is_zero() {
        let cmp = |l: &TIA, r: &TIA| match optimize {
            Optimize::Max => l > r,
            Optimize::Min => l < r,
        };
        return Ok(iter
            .enumerate()
            .reduce(|l, r| if cmp(&l.1, &r.1) { l } else { r })
            .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
            .0);
    }

    (iter.enumerate())
        // skip NaN scores. These should not be in the input domain, but this results in graceful failure
        .filter_map(|(i, v)| Some((i, FBig::try_from(v).ok()?)))
        .map(|(i, mut shift)| {
            // normalize sign
            if optimize == Optimize::Min {
                shift = -shift;
            }

            // create a partial sample
            Ok((i, PartialSample::new(RV::new(shift, scale.clone())?)))
        })
        .reduce(|l, r| {
            let (mut l, mut r) = (l?, r?);
            Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
        })
        .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
        .map(|v| v.0)
}
