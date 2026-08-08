use std::sync::Arc;

use super::{PrivacyGuarantee, RenyiFn, RenyiRepresentation, ZCDPRepresentation, check_rho};
use crate::{
    error::Fallible,
    traits::{SInterval, backend::Dashu},
};

impl PrivacyGuarantee {
    /// Compose privacy guarantees while retaining every supported runtime
    /// representation. Static capability selection is performed by
    /// `CompositionMeasure::compose_measure`; this method only evaluates
    /// representations that happen to be present at runtime.
    pub(crate) fn compose(curves: Vec<Self>) -> Fallible<Self> {
        if curves.is_empty() {
            let mut identity = PrivacyGuarantee::new();
            identity.renyi = Some(RenyiRepresentation {
                curve: Arc::new(|_| Ok(0.0)),
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

        #[cfg(feature = "idealized-numerics")]
        match compose_gaussianDP(&curves) {
            Ok(gaussian) => out.gaussian = gaussian,
            Err(error) => path_errors.push(error),
        }
        match compose_renyiDP(&curves) {
            Ok(renyi) => out.renyi = renyi,
            Err(error) => path_errors.push(error),
        }
        match compose_zCDP(&curves) {
            Ok(zcdp) => out.zcdp = zcdp,
            Err(error) => path_errors.push(error),
        }

        let has_representation = out.renyi.is_some() || out.zcdp.is_some() || {
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
            return fallible!(
                FailedFunction,
                "PrivacyGuarantee composition has no supported common representation"
            );
        }

        Ok(out)
    }
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

    let mut sum_mu2 = SInterval::<Dashu>::point(0.0)?;
    for curve in curves {
        let mu = curve.gaussian.unwrap().mu;
        let mu = SInterval::<Dashu>::point(mu)?;
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
fn compose_zCDP(curves: &[PrivacyGuarantee]) -> Fallible<Option<ZCDPRepresentation>> {
    if curves.iter().any(|curve| curve.zcdp.is_none()) {
        return Ok(None);
    }

    let mut rho_sum = SInterval::<Dashu>::point(0.0)?;
    let mut delta_sum = SInterval::<Dashu>::point(0.0)?;
    let mut rho_is_infinite = false;

    for curve in curves {
        let ZCDPRepresentation { rho, source_delta } = curve.zcdp.unwrap();
        check_rho(rho)?;
        super::check_delta(source_delta)?;

        if rho.is_infinite() {
            rho_is_infinite = true;
        } else {
            rho_sum = (rho_sum + SInterval::<Dashu>::point(rho)?)?;
        }
        delta_sum = (delta_sum + SInterval::<Dashu>::point(source_delta)?)?;
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

/// Compose RDP using native pure RDP and the approved pure-zCDP embedding.
/// Approximate zCDP is never embedded into exact RDP.
#[allow(non_snake_case)]
fn compose_renyiDP(curves: &[PrivacyGuarantee]) -> Fallible<Option<RenyiRepresentation>> {
    let mut components = Vec::with_capacity(curves.len());
    let mut source_delta = 0.0;

    // If an all-exact path exists, use it independently of any approximate
    // native RDP representation. This prevents an invalid pointwise minimum
    // between approximate RDP and exact zCDP.
    let has_exact_path = curves.iter().all(|guarantee| {
        matches!(
            guarantee.renyi,
            Some(RenyiRepresentation {
                source_delta: 0.0,
                ..
            })
        ) || matches!(
            guarantee.zcdp,
            Some(ZCDPRepresentation {
                source_delta: 0.0,
                ..
            })
        )
    });

    for guarantee in curves {
        let component = if has_exact_path {
            match (&guarantee.renyi, guarantee.zcdp) {
                (
                    Some(RenyiRepresentation {
                        curve,
                        source_delta: 0.0,
                    }),
                    Some(ZCDPRepresentation {
                        rho,
                        source_delta: 0.0,
                    }),
                ) => {
                    check_rho(rho)?;
                    RdpComponent::Minimum {
                        native: curve.clone(),
                        zcdp_rho: rho,
                    }
                }
                (
                    Some(RenyiRepresentation {
                        curve,
                        source_delta: 0.0,
                    }),
                    _,
                ) => RdpComponent::Native(curve.clone()),
                (
                    _,
                    Some(ZCDPRepresentation {
                        rho,
                        source_delta: 0.0,
                    }),
                ) => {
                    check_rho(rho)?;
                    RdpComponent::ZCDP(rho)
                }
                _ => unreachable!("exact RDP path was checked above"),
            }
        } else {
            match (&guarantee.renyi, guarantee.zcdp) {
                (
                    Some(RenyiRepresentation {
                        curve,
                        source_delta,
                    }),
                    _,
                ) if *source_delta == 0.0 => RdpComponent::Native(curve.clone()),
                (
                    Some(RenyiRepresentation {
                        curve,
                        source_delta: native_delta,
                    }),
                    Some(ZCDPRepresentation {
                        source_delta: 0.0, ..
                    }),
                ) => {
                    // Approximate native RDP cannot be intersected with exact
                    // zCDP. It is retained only as an approximate path when
                    // no all-exact composition path exists.
                    super::check_delta(*native_delta)?;
                    source_delta = *native_delta;
                    RdpComponent::Native(curve.clone())
                }
                (Some(RenyiRepresentation { .. }), _) => return Ok(None),
                (
                    None,
                    Some(ZCDPRepresentation {
                        rho,
                        source_delta: 0.0,
                    }),
                ) => {
                    check_rho(rho)?;
                    RdpComponent::ZCDP(rho)
                }
                // An approximate zCDP source delta is local to zCDP and is
                // not silently reinterpreted as approximate-RDP source delta.
                (None, Some(ZCDPRepresentation { .. })) | (None, None) => return Ok(None),
            }
        };
        components.push(component);
    }

    if source_delta != 0.0 && curves.len() != 1 {
        return Ok(None);
    }

    let curve = Arc::new(move |alpha: f64| -> Fallible<f64> {
        check_renyi_order(alpha)?;
        let mut sum = SInterval::<Dashu>::point(0.0)?;
        for component in &components {
            let epsilon = match component {
                RdpComponent::Native(native) => eval_native_rdp(native.as_ref(), alpha)?,
                RdpComponent::ZCDP(rho) => eval_zcdp_rdp(*rho, alpha)?,
                RdpComponent::Minimum { native, zcdp_rho } => {
                    let native = eval_native_rdp(native.as_ref(), alpha);
                    let zcdp = eval_zcdp_rdp(*zcdp_rho, alpha);
                    match (native, zcdp) {
                        (Ok(native), Ok(zcdp)) => native.min(zcdp),
                        (Ok(native), Err(_)) => native,
                        (Err(_), Ok(zcdp)) => zcdp,
                        (Err(error), Err(_)) => return Err(error),
                    }
                }
            };
            if epsilon.is_infinite() {
                return Ok(f64::INFINITY);
            }
            sum = (sum + SInterval::<Dashu>::point(epsilon)?)?;
        }
        sum.upper_f64()
    });

    Ok(Some(RenyiRepresentation {
        curve,
        source_delta,
    }))
}

fn eval_native_rdp(curve: &RenyiFn, alpha: f64) -> Fallible<f64> {
    let epsilon = curve(alpha)?;
    if epsilon.is_nan() || epsilon.is_sign_negative() {
        return fallible!(
            FailedMap,
            "RDP epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }
    Ok(epsilon)
}

fn eval_zcdp_rdp(rho: f64, alpha: f64) -> Fallible<f64> {
    if rho.is_infinite() {
        return Ok(f64::INFINITY);
    }
    (SInterval::<Dashu>::point(alpha)? * SInterval::<Dashu>::point(rho)?)?.upper_f64()
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
