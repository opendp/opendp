use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    traits::{ExactIntCast, Number},
    transformations::{
        make_quantile_score_candidates, score_candidates_constants, traits::UnboundedMetric,
    },
};

use super::{TopKMeasure, make_noisy_max};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(output_measure(c_type = "AnyMeasure *", rust_type = b"null", hint = "Measure")),
    generics(MI(suppress), MO(suppress), T(suppress)),
    derived_types(T = "$get_atom(get_type(input_domain))")
)]
/// Makes a Measurement the computes the quantile of a dataset.
///
/// # Arguments
/// * `input_domain` - Uses a tighter sensitivity when the size of vectors in the input domain is known.
/// * `input_metric` - Either SymmetricDistance or InsertDeleteDistance.
/// * `output_measure` - Either MaxDivergence or ZeroConcentratedDivergence.
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
/// * `scale` - the scale of the noise added
///
/// # Generics
/// * `MI` - Input Metric.
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_private_quantile<MI: 'static + UnboundedMetric, MO: TopKMeasure, T: Number>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: MI,
    output_measure: MO,
    mut candidates: Vec<T>,
    alpha: f64,
    scale: f64,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, MI, MO, T>>
where
    (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
{
    if let Some(idx) = candidates.iter().position(|c| c.is_null()) {
        return fallible!(
            MakeTransformation,
            "candidates[{idx}] is NaN, which is invalid"
        );
    }
    candidates.sort_by(|a, b| a.partial_cmp(b).expect("candidates are not NaN"));

    // scale up the noise proportionally to the increase in sensitivity
    // due to the integerization of scores
    let denominator = score_candidates_constants(
        input_domain.size.map(u64::exact_int_cast).transpose()?,
        alpha,
    )?
    .1;

    let t_score =
        make_quantile_score_candidates(input_domain, input_metric, candidates.clone(), alpha)?;

    let m_rnm = make_noisy_max(
        t_score.output_domain.clone(),
        t_score.output_metric.clone(),
        output_measure,
        scale * denominator as f64,
        true,
    )?;
    let p_index = Function::new(move |idx: &usize| candidates[*idx]);

    t_score >> m_rnm >> p_index
}
