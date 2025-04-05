use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::Number,
    transformations::{
        make_quantile_score_candidates, score_candidates_constants, traits::UnboundedMetric,
    },
};

use super::{Optimize, make_report_noisy_max_gumbel};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    generics(MI(suppress), T(suppress)),
    derived_types(T = "$get_atom(get_type(input_domain))")
)]
/// Makes a Measurement the computes the quantile of a dataset.
///
/// # Arguments
/// * `input_domain` - Uses a tighter sensitivity when the size of vectors in the input domain is known.
/// * `input_metric` - Either SymmetricDistance or InsertDeleteDistance.
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
/// * `scale` - the scale of the noise added
///
/// # Generics
/// * `MI` - Input Metric.
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_private_quantile<MI: 'static + UnboundedMetric, T: Number>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: MI,
    mut candidates: Vec<T>,
    alpha: f64,
    scale: f64,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, T, MI, MaxDivergence>>
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
    let denominator = score_candidates_constants(input_domain.size.clone(), alpha)?.1;

    let t_score =
        make_quantile_score_candidates(input_domain, input_metric, candidates.clone(), alpha)?;

    let m_rnm = make_report_noisy_max_gumbel(
        t_score.output_domain.clone(),
        t_score.output_metric.clone(),
        scale * denominator as f64 * 2.0,
        Optimize::Min,
    )?;
    let p_index = Function::new(move |idx: &usize| candidates[*idx]);

    t_score >> m_rnm >> p_index
}
