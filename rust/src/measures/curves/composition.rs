use std::sync::Arc;

use super::{
    ApproxDPRepr, PrivacyGuarantee, RenyiFn, RenyiRepresentation, ZCDPRepresentation, check_rho,
};
use crate::{combinators::Composability, error::Fallible, measures::PrivacyProfile, traits::SInterval};

impl PrivacyGuarantee {
    /// Compose independently certified runtime privacy facts under the
    /// interaction model enforced by the compositor.
    pub(crate) fn compose(
        curves: Vec<Self>,
        composability: Composability,
    ) -> Fallible<Self> {
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
        let mut path_errors = Vec::new();

        out.approx_dp = compose_approx_dp(&curves)?;
        #[cfg(feature = "idealized-numerics")]
        match compose_gaussianDP(&curves) {
            Ok(gaussian) => out.gaussian = gaussian,
            Err(error) => path_errors.push(error),
        }
        match compose_renyiDP(&curves, sequential) {
            Ok(renyi) => out.renyi = renyi,
            Err(error) => path_errors.push(error),
        }
        match compose_zCDP(&curves, sequential) {
            Ok(zcdp) => out.zcdp = zcdp,
            Err(error) => path_errors.push(error),
        }

        let has_representation =
            out.approx_dp.is_some() || out.renyi.is_some() || out.zcdp.is_some() || {
                #[cfg(feature = "idealized-numerics")]
                {
                    out.gaussian.is_some()
                }
                #[cfg(not(feature = "idealized-numerics"))]
                {
                    false
                }
            };
        if !has_representation {
            if let Some(error) = path_errors.into_iter().next() {
                return Err(error);
            }
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
    PrivacyProfile::new(move |epsilons| {
        epsilons.iter().map(|&epsilon| {
            let allocation = (SInterval::point(epsilon)? / SInterval::point(count)?)?.lower_f64()?;
            let mut delta = SInterval::point(0.0)?;
            for curve in &curves {
                delta = (delta + SInterval::point(curve.delta(allocation)?)?)?;
            }
            delta.min(SInterval::point(1.0)?).upper_f64()
        }).collect()
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

/// Native approximate-zCDP representations compose by adding rho and source
/// delta. No RDP-to-zCDP conversion is attempted.
#[allow(non_snake_case)]
fn compose_zCDP(
    curves: &[PrivacyGuarantee],
    sequential: bool,
) -> Fallible<Option<ZCDPRepresentation>> {
    if curves.iter().any(|curve| {
        curve.zcdp.is_none_or(|repr| !sequential && repr.source_delta != 0.0)
    }) {
        return Ok(None);
    }

    let mut rho_sum = SInterval::point(0.0)?;
    let mut delta_sum = SInterval::point(0.0)?;
    let mut rho_is_infinite = false;

    for curve in curves {
        let ZCDPRepresentation { rho, source_delta } = curve.zcdp.unwrap();
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

#[derive(Clone)]
enum RdpComponent {
    Native(Arc<RenyiFn>),
    ZCDP(f64),
    Minimum { native: Arc<RenyiFn>, zcdp_rho: f64 },
}

/// Compose native RDP, embedding pure zCDP when available. Positive
/// approximate-RDP slack is used only when children cannot be interleaved.
#[allow(non_snake_case)]
fn compose_renyiDP(
    curves: &[PrivacyGuarantee],
    sequential: bool,
) -> Fallible<Option<RenyiRepresentation>> {
    let mut components = Vec::with_capacity(curves.len());
    let mut source_delta = SInterval::point(0.0)?;

    for guarantee in curves {
        let exact_native = guarantee.renyi.as_ref().filter(|r| r.source_delta == 0.0);
        let exact_zcdp = guarantee.zcdp.filter(|r| r.source_delta == 0.0);
        let component = match (exact_native, exact_zcdp) {
            (Some(native), Some(zcdp)) => RdpComponent::Minimum {
                native: native.curve.clone(),
                zcdp_rho: zcdp.rho,
            },
            (Some(native), None) => RdpComponent::Native(native.curve.clone()),
            (None, Some(zcdp)) => RdpComponent::ZCDP(zcdp.rho),
            (None, None) if sequential || curves.len() == 1 => {
                let Some(native) = &guarantee.renyi else { return Ok(None); };
                source_delta = (source_delta + SInterval::point(native.source_delta)?)?;
                RdpComponent::Native(native.curve.clone())
            }
            _ => return Ok(None),
        };
        components.push(component);
    }
    let source_delta = source_delta.min(SInterval::point(1.0)?).upper_f64()?;

    let curve = Arc::new(move |orders: &[f64]| -> Fallible<Vec<f64>> {
        for &alpha in orders {
            check_renyi_order(alpha)?;
        }
        let mut sums = vec![SInterval::point(0.0)?; orders.len()];
        for component in &components {
            let evaluate_native = |native: &Arc<RenyiFn>| {
                super::renyidp::evaluate_renyi(native.as_ref(), orders)
            };
            let evaluate_zcdp = |rho: f64| {
                orders.iter().map(|&alpha| eval_zcdp_rdp(rho, alpha)).collect::<Fallible<Vec<_>>>()
            };
            let values = match component {
                RdpComponent::Native(native) => evaluate_native(native)?,
                RdpComponent::ZCDP(rho) => evaluate_zcdp(*rho)?,
                RdpComponent::Minimum { native, zcdp_rho } => {
                    match (evaluate_native(native), evaluate_zcdp(*zcdp_rho)) {
                        (Ok(native), Ok(zcdp)) => native.into_iter().zip(zcdp)
                            .map(|(a,b)| a.min(b)).collect(),
                        (Ok(values), Err(_)) | (Err(_), Ok(values)) => values,
                        (Err(error), Err(_)) => return Err(error),
                    }
                }
            };
            for (sum, epsilon) in sums.iter_mut().zip(values) {
                *sum = (sum.clone() + SInterval::point(epsilon)?)?;
            }
        }
        sums.into_iter().map(|sum| sum.upper_f64()).collect()
    });

    Ok(Some(RenyiRepresentation { curve, source_delta }))
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
