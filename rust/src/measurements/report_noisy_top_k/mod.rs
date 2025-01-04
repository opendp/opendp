use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{BoundedRange, MaxDivergence},
    metrics::LInfDistance,
    traits::{
        samplers::{ExponentialRV, GumbelRV, InverseCDF, PartialSample},
        InfCast, InfDiv, Number,
    },
};
use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;
use std::{cmp::Ordering, fmt::Display};
use Ordering::*;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        optimize(c_type = "char *", rust_type = "String"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `output_measure` - One of `MaxDivergence` or `BoundedRange`.
/// * `k` - Number of indices to select.
/// * `scale` - Scale for the noise distribution.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_report_noisy_top_k<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    k: usize,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, MO>>
where
    TIA: Number,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    use crate::traits::InfMul;

    if input_domain.element_domain.nullable() {
        return fallible!(
            MakeMeasurement,
            "elements in the input vector domain must be non-null"
        );
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }

    let f_scale = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            Function::new_fallible(move |arg: &Vec<TIA>| {
                let iter = arg.iter().enumerate();
                let cmp = match optimize {
                    Optimize::Max => |l: &mut (usize, &TIA), r: &mut (usize, &TIA)| Ok(l.1 > r.1),
                    Optimize::Min => |l: &mut (usize, &TIA), r: &mut (usize, &TIA)| Ok(l.1 < r.1),
                };

                Ok(top(iter, k, cmp)?.into_iter().map(|(i, _)| i).collect())
            })
        } else {
            Function::new_fallible(move |arg: &Vec<TIA>| {
                let iter = arg.iter().enumerate();
                let iter = iter
                    // skip NaN scores. These should not be in the input domain, but this results in graceful failure
                    .filter_map(|(i, v)| Some((i, FBig::try_from(*v).ok()?)))
                    .map(|(i, mut shift)| {
                        // normalize sign
                        if optimize == Optimize::Min {
                            shift = -shift;
                        }

                        // create a partial sample
                        (i, PartialSample::new(MO::RV::new(shift, f_scale.clone())))
                    });

                Ok(top(iter, k, |l, r| l.1.greater_than(&mut r.1))?
                    .into_iter()
                    .map(|(i, _)| i)
                    .collect())
            })
        },
        input_metric.clone(),
        output_measure,
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(d_in.clone())?;

            // convert data type to QO
            let d_in = f64::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(
                    InvalidDistance,
                    "sensitivity ({}) must be non-negative",
                    d_in
                );
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_out >= d_in / scale
            d_in.inf_div(&scale)?
                .inf_mul(&<f64 as InfCast<usize>>::inf_cast(k)?)
        }),
    )
}

/// Returns the top k elements from the iterator, using a heap to track the top k elements.
/// Optimized for the case where k is small compared to the number of elements in the iterator.
fn top<T>(
    mut iter: impl Iterator<Item = T>,
    k: usize,
    greater_than: impl Fn(&mut T, &mut T) -> Fallible<bool>,
) -> Fallible<Vec<T>> {
    let mut heap = Vec::with_capacity(k);
    for mut value in iter.by_ref().take(k) {
        let index = partition_point_mut(&mut heap, |x| greater_than(x, &mut value))?;
        heap.insert(index, value);
    }

    if heap.is_empty() {
        return Ok(heap);
    }

    for mut value in iter {
        if greater_than(heap.last_mut().unwrap(), &mut value)? {
            continue;
        }
        heap.pop();
        let index = partition_point_mut(&mut heap, |x| greater_than(x, &mut value))?;
        heap.insert(index, value);
    }

    Ok(heap)
}

pub fn partition_point_mut<T, P>(data: &mut Vec<T>, mut pred: P) -> Fallible<usize>
where
    P: FnMut(&mut T) -> Fallible<bool>,
{
    binary_search_by_mut(data, |x| Ok(if pred(x)? { Less } else { Greater }))
}

pub fn binary_search_by_mut<T, F>(data: &mut Vec<T>, mut f: F) -> Fallible<usize>
where
    F: FnMut(&mut T) -> Fallible<Ordering>,
{
    let mut size = data.len();
    if size == 0 {
        return Ok(0);
    }
    let mut base = 0usize;

    while size > 1 {
        let half = size / 2;
        let mid = base + half;

        let cmp = f(&mut data[mid])?;
        base = if let Greater = cmp { base } else { mid };

        size -= half;
    }

    let cmp = f(&mut data[base])?;
    Ok(base + (cmp == Less) as usize)
}

pub trait SelectionMeasure: 'static + Measure<Distance = f64> {
    type RV: ShiftScaleRV;
}

impl SelectionMeasure for BoundedRange {
    type RV = GumbelRV;
}

impl SelectionMeasure for MaxDivergence {
    type RV = ExponentialRV;
}

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

pub trait ShiftScaleRV: InverseCDF + Sized {
    fn new(shift: FBig, scale: FBig) -> Self;
}

impl ShiftScaleRV for GumbelRV {
    fn new(shift: FBig, scale: FBig) -> Self {
        GumbelRV { shift, scale }
    }
}

impl ShiftScaleRV for ExponentialRV {
    fn new(shift: FBig, scale: FBig) -> Self {
        ExponentialRV { shift, scale }
    }
}
