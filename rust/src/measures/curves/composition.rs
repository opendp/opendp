use std::sync::Arc;

use super::{
    ApproxDPRepr, PrivacyGuarantee, RenyiFn, RenyiRepresentation, ZCDPRepresentation, check_rho,
    logspace::logaddexp_upper,
};
use crate::{
    combinators::Composability, error::Fallible, measures::PrivacyProfile, traits::SInterval,
};

impl PrivacyGuarantee {
    /// Compose independently certified runtime privacy facts under the
    /// interaction model enforced by the compositor.
    pub(crate) fn compose(curves: Vec<Self>, composability: Composability) -> Fallible<Self> {
        if curves.len() == 1 {
            return Ok(curves.into_iter().next().unwrap());
        }
        let sequential = composability == Composability::Sequential;
        if curves.is_empty() {
            let mut identity = PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?;
            identity.renyi = Some(RenyiRepresentation {
                curve: Arc::new(|orders| Ok(vec![0.0; orders.len()])),
                source_delta: 0.0,
            });
            identity.zcdp = Some(ZCDPRepresentation {
                rho: 0.0,
                source_delta: 0.0,
            });
            #[cfg(feature = "idealized-numerics")]
            {
                identity.gaussian = Some(super::GaussianDPRepresentation { mu: 0.0 });
            }
            return Ok(identity);
        }

        let mut out = PrivacyGuarantee::new();

        out.approx_dp = compose_approx_dp(&curves)?;
        #[cfg(feature = "idealized-numerics")]
        {
            out.gaussian = compose_gaussianDP(&curves)?;
        }
        out.renyi = compose_renyiDP(&curves, sequential)?;
        out.zcdp = compose_zCDP(&curves, sequential)?;

        let has_representation = out.approx_dp.as_ref().is_some_and(|r| r.delta < 1.0)
            || out.renyi.as_ref().is_some_and(|r| r.source_delta < 1.0)
            || out
                .zcdp
                .is_some_and(|r| r.rho.is_finite() && r.source_delta < 1.0)
            || {
                #[cfg(feature = "idealized-numerics")]
                {
                    out.gaussian.is_some()
                }
                #[cfg(not(feature = "idealized-numerics"))]
                {
                    false
                }
            };
        if !has_representation || curves.iter().any(|curve| curve.profile.is_some()) {
            out.profile = Some(compose_profiles(curves));
        }

        Ok(out)
    }
}

/// A fixed equal allocation of epsilon is valid for every collection of
/// approximate-DP bounds, regardless of how those bounds were derived.
fn compose_profiles(curves: Vec<PrivacyGuarantee>) -> PrivacyProfile {
    let count = curves.len() as f64;
    // On platforms where usize can exceed the exact f64 integer range,
    // round the count upward before dividing.
    let count = if curves.len() as u128 > (1u128 << 53) {
        count.next_up()
    } else {
        count
    };
    PrivacyProfile::new_log(move |epsilons| {
        let allocations = epsilons
            .iter()
            .map(|&epsilon| {
                if epsilon.is_infinite() {
                    return Ok(epsilon);
                }
                (SInterval::point(epsilon)? / SInterval::point(count)?)?.lower_f64()
            })
            .collect::<Fallible<Vec<_>>>()?;
        let mut sums = vec![f64::NEG_INFINITY; epsilons.len()];
        for curve in &curves {
            for (sum, log_delta) in sums.iter_mut().zip(curve.log_delta_batch(&allocations)?) {
                *sum = logaddexp_upper(*sum, log_delta)?;
            }
        }
        Ok(sums)
    })
}

/// Basic composition of independently certified approximate-DP bounds.
/// This does not derive certificates from analytical privacy profiles.
fn compose_approx_dp(curves: &[PrivacyGuarantee]) -> Fallible<Option<ApproxDPRepr>> {
    if curves.iter().any(|curve| curve.approx_dp.is_none()) {
        return Ok(None);
    }
    let mut epsilon_sum = SInterval::point(0.0)?;
    let mut delta_sum = SInterval::point(0.0)?;
    for curve in curves {
        let repr = curve.approx_dp.as_ref().unwrap();
        epsilon_sum = (epsilon_sum + SInterval::point(repr.epsilon)?)?;
        delta_sum = (delta_sum + SInterval::point(repr.delta)?)?;
    }
    let epsilon = epsilon_sum.upper_f64()?;
    if !epsilon.is_finite() {
        return fallible!(Overflow, "composed approximate-DP epsilon is not finite");
    }
    ApproxDPRepr::new(epsilon, delta_sum.upper_f64()?.min(1.0)).map(Some)
}

/// Gaussian-DP composition is analytic: mu_total² = sum_i mu_i².
#[cfg(feature = "idealized-numerics")]
#[allow(non_snake_case)]
fn compose_gaussianDP(
    curves: &[PrivacyGuarantee],
) -> Fallible<Option<super::GaussianDPRepresentation>> {
    if curves.iter().any(|curve| curve.gaussian.is_none()) {
        return Ok(None);
    }

    let mut sum_mu2 = SInterval::point(0.0)?;
    for curve in curves {
        let mu = curve.gaussian.unwrap().mu;
        let mu = SInterval::point(mu)?;
        sum_mu2 = (sum_mu2 + (mu.clone() * mu)?)?;
    }

    let mu = sum_mu2.sqrt()?.upper_f64()?;
    if !mu.is_finite() {
        return fallible!(Overflow, "composed Gaussian DP parameter is not finite");
    }
    Ok(Some(super::GaussianDPRepresentation { mu }))
}

/// Compose zCDP bounds, including those implied by pure DP and GDP.
/// Positive slack is admissible only for sequential composition.
#[allow(non_snake_case)]
fn compose_zCDP(
    curves: &[PrivacyGuarantee],
    sequential: bool,
) -> Fallible<Option<ZCDPRepresentation>> {
    let mut rho_sum = SInterval::point(0.0)?;
    let mut delta_sum = SInterval::point(0.0)?;
    let mut rho_is_infinite = false;

    for curve in curves {
        let exact = curve.exact_zcdp_rho()?;
        let approximate = curve.zcdp.filter(|r| sequential && r.source_delta != 0.0);
        let repr = match (exact, approximate) {
            (Some(rho), Some(r)) if r.rho < rho => r,
            (Some(rho), _) => ZCDPRepresentation {
                rho,
                source_delta: 0.0,
            },
            (None, Some(r)) => r,
            (None, None) => return Ok(None),
        };
        let ZCDPRepresentation { rho, source_delta } = repr;
        check_rho(rho)?;
        super::check_delta(source_delta)?;

        if rho.is_infinite() {
            rho_is_infinite = true;
        } else {
            rho_sum = (rho_sum + SInterval::point(rho)?)?;
        }
        delta_sum = (delta_sum + SInterval::point(source_delta)?)?;
    }

    Ok(Some(ZCDPRepresentation {
        rho: if rho_is_infinite {
            f64::INFINITY
        } else {
            rho_sum.upper_f64()?
        },
        source_delta: delta_sum.upper_f64()?.min(1.0),
    }))
}

struct RdpComponent {
    native: Option<Arc<RenyiFn>>,
    pure_epsilon: Option<f64>,
    zcdp_rho: Option<f64>,
}

/// Compose native RDP and pure-DP bounds, embedding exact zCDP when available. Positive
/// approximate-RDP slack is used only when children cannot be interleaved.
#[allow(non_snake_case)]
fn compose_renyiDP(
    curves: &[PrivacyGuarantee],
    sequential: bool,
) -> Fallible<Option<RenyiRepresentation>> {
    if curves.iter().all(|curve| {
        curve.renyi.is_none()
            && !curve
                .approx_dp()
                .is_some_and(|(epsilon, delta)| epsilon > 0.0 && delta == 0.0)
    }) {
        return Ok(None);
    }
    let mut components = Vec::with_capacity(curves.len());
    let mut source_delta = SInterval::point(0.0)?;
    let mut rho_sum = SInterval::point(0.0)?;

    for guarantee in curves {
        let exact_native = guarantee.renyi.as_ref().filter(|r| r.source_delta == 0.0);
        let exact_zcdp = guarantee.exact_zcdp_rho()?.filter(|rho| rho.is_finite());
        let pure_epsilon = guarantee
            .approx_dp()
            .filter(|&(_, delta)| delta == 0.0)
            .map(|(epsilon, _)| epsilon);
        let component = if exact_native.is_some() || pure_epsilon.is_some() {
            RdpComponent {
                native: exact_native.map(|native| native.curve.clone()),
                pure_epsilon,
                zcdp_rho: exact_zcdp,
            }
        } else if let Some(rho) = exact_zcdp {
            rho_sum = (rho_sum + SInterval::point(rho)?)?;
            continue;
        } else if sequential {
            let Some(native) = &guarantee.renyi else {
                return Ok(None);
            };
            source_delta = (source_delta + SInterval::point(native.source_delta)?)?;
            RdpComponent {
                native: Some(native.curve.clone()),
                pure_epsilon: None,
                zcdp_rho: None,
            }
        } else {
            return Ok(None);
        };
        components.push(component);
    }
    if components.is_empty() {
        return Ok(None);
    }
    let source_delta = source_delta.min(SInterval::point(1.0)?).upper_f64()?;

    let curve = Arc::new(move |orders: &[f64]| -> Fallible<Vec<f64>> {
        for &alpha in orders {
            check_renyi_order(alpha)?;
        }
        let mut sums = orders
            .iter()
            .map(|&alpha| SInterval::point(alpha)? * rho_sum.clone())
            .collect::<Fallible<Vec<_>>>()?;
        for component in &components {
            let values = if let Some(native) = &component.native {
                super::renyidp::evaluate_renyi(native.as_ref(), orders)?
            } else {
                vec![f64::INFINITY; orders.len()]
            };
            for ((sum, &alpha), mut epsilon) in sums.iter_mut().zip(orders).zip(values) {
                if let Some(pure) = component.pure_epsilon {
                    epsilon = epsilon.min(super::renyidp::rdp_from_pureDP(alpha, pure)?);
                }
                if let Some(rho) = component.zcdp_rho {
                    epsilon = epsilon.min(eval_zcdp_rdp(rho, alpha)?);
                }
                *sum = (sum.clone() + SInterval::point(epsilon)?)?;
            }
        }
        sums.into_iter().map(|sum| sum.upper_f64()).collect()
    });

    Ok(Some(RenyiRepresentation {
        curve,
        source_delta,
    }))
}

fn eval_zcdp_rdp(rho: f64, alpha: f64) -> Fallible<f64> {
    if rho.is_infinite() {
        return Ok(f64::INFINITY);
    }
    (SInterval::point(alpha)? * SInterval::point(rho)?)?.upper_f64()
}

fn check_renyi_order(alpha: f64) -> Fallible<()> {
    if !alpha.is_finite() || alpha <= 1.0 {
        return fallible!(
            FailedMap,
            "Rényi order alpha ({alpha}) must be finite and greater than one"
        );
    }
    Ok(())
}

#[cfg(test)]
mod test;
