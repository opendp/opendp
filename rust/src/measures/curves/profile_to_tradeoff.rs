use crate::{
    error::Fallible,
    measures::curves::{
        PrivacyProfile, check_alpha,
        maximize::{Candidate, refine_maxima},
    },
    traits::SInterval,
};

const EPS_TRUE_MIN: f64 = 744.4400719213812;
type Cert = SInterval;

/// Convert a conservative profile into tradeoff lower bounds. Each sampled
/// epsilon provides two supporting lines for every type-I error. Share the
/// coarse grid and batch the local refinement probes across all queries.
pub(super) fn beta_via_profile_batch(
    profile: &PrivacyProfile,
    alphas: &[f64],
) -> Fallible<Vec<f64>> {
    super::check_inputs(alphas, check_alpha)?;
    let start = alphas.partition_point(|&alpha| alpha == 0.0);
    let end = alphas.partition_point(|&alpha| alpha < 1.0);
    let mut out = vec![0.0; alphas.len()];
    if start > 0 {
        // At alpha=0 the infinity endpoint gives the strongest bound.
        out[..start].fill(one_minus_delta(profile.log_delta(&[f64::INFINITY])?[0])?.lower_f64()?);
    }
    if start == end {
        return Ok(out);
    }
    let alphas = &alphas[start..end];

    // A geometric grid resolves the neighborhood of zero while reaching
    // every scale at which a positive f64 lower bound is possible.
    let epsilons = std::iter::once(0.0)
        .chain((-104..=19).map(|i| 2.0_f64.powf(f64::from(i) / 2.0)))
        .chain([EPS_TRUE_MIN])
        .collect::<Vec<_>>();
    let bounds = profile.log_delta(&epsilons)?;
    let score = |query: usize, epsilon: f64, log_delta: f64| {
        let alpha = alphas[query / 2];
        let complement = -log_delta.exp_m1();
        Ok(if query % 2 == 0 {
            complement - (epsilon + alpha.ln()).exp()
        } else {
            // Log-space preserves tiny positive values in the second branch.
            -epsilon + (complement - alpha).max(0.0).ln()
        })
    };

    let mut best = Vec::with_capacity(2 * alphas.len());
    let mut brackets = Vec::with_capacity(2 * alphas.len());
    for query in 0..2 * alphas.len() {
        let mut index = 0;
        let mut candidate = Candidate {
            arg: epsilons[0],
            bound: bounds[0],
            score: score(query, epsilons[0], bounds[0])?,
        };
        for (i, (&arg, &bound)) in epsilons.iter().zip(&bounds).enumerate().skip(1) {
            let value = score(query, arg, bound)?;
            if value > candidate.score {
                index = i;
                candidate = Candidate {
                    arg,
                    bound,
                    score: value,
                };
            }
        }
        best.push(candidate);
        brackets.push((
            query,
            epsilons[index.saturating_sub(1)],
            epsilons[(index + 1).min(epsilons.len() - 1)],
        ));
    }

    refine_maxima(brackets, &mut best, |eps| profile.log_delta(eps), score)?;
    for (i, (&alpha, candidates)) in alphas.iter().zip(best.chunks_exact(2)).enumerate() {
        let first = candidates[0];
        let second = candidates[1];
        out[start + i] = beta_profile_candidate_t1_lower(alpha, first.arg, first.bound)?.max(
            beta_profile_candidate_t2_lower(alpha, second.arg, second.bound)?,
        );
    }
    // A lower bound at a larger alpha also holds at all smaller alphas.
    let mut best = 0.0_f64;
    for value in out.iter_mut().rev() {
        best = best.max(*value);
        *value = best;
    }
    Ok(out)
}

/// Conservative enclosure of the complement of an upper log-delta bound.
fn one_minus_delta(log_delta: f64) -> Fallible<Cert> {
    if log_delta == f64::NEG_INFINITY {
        return Cert::point(1.0);
    }
    // Clamping the log upward can only decrease the complement.
    Ok(-Cert::point(log_delta.max(-1024.0))?.exp_m1()?)
}

#[inline]
fn positive_part(x: Cert) -> Fallible<Cert> {
    x.max(Cert::point(0.0)?).clamp01()
}

// Certified lower evaluation of branch 1:
//
//     1 - delta(eps) - alpha * exp(eps).
#[inline]
fn beta_profile_candidate_t1_lower(alpha: f64, epsilon: f64, log_delta: f64) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta(log_delta)?;

    // Compute alpha * exp(epsilon) as exp(epsilon + ln(alpha)).
    // This avoids overflowing exp(epsilon) when alpha is tiny.
    let z = (Cert::point(epsilon)? + Cert::point(alpha)?.ln()?)?;

    // If the upper endpoint is nonnegative, then alpha * exp(epsilon) may be
    // at least one, and returning zero is a conservative lower bound.
    if z.upper_f64()? >= 0.0 {
        return Ok(0.0);
    }

    let alpha_exp_eps = z.exp()?;

    positive_part((one_minus_delta - alpha_exp_eps)?)?.lower_f64()
}

// Certified lower evaluation of branch 2:
//
//     exp(-eps) * max(1 - delta(eps) - alpha, 0).
#[inline]
fn beta_profile_candidate_t2_lower(alpha: f64, epsilon: f64, log_delta: f64) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta(log_delta)?;

    let base = positive_part((one_minus_delta - Cert::point(alpha)?)?)?;

    if base.upper_f64()? == 0.0 {
        return Ok(0.0);
    }

    ((Cert::point(-epsilon)?.exp()?) * base)?
        .clamp01()?
        .lower_f64()
}
