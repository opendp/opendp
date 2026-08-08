use std::sync::Arc;

use super::{
    ApproxDPPoint, PrivacyGuarantee, RenyiDP, RenyiFn, ZCDP, check_mu, logspace::check_delta,
};
use crate::{
    error::Fallible,
    traits::{SInterval, backend::Dashu},
};

type DInterval = SInterval<Dashu>;

impl PrivacyGuarantee {
    /// Compose privacy guarantees while retaining every supported representation.
    ///
    /// Each supported representation is composed independently. A representation
    /// is attached to the result only when every input supplies it natively or via
    /// an explicitly supported analytic embedding.
    pub(crate) fn compose(curves: Vec<Self>) -> Fallible<Self> {
        if curves.is_empty() {
            return PrivacyGuarantee::new().with_approxDP(vec![(0.0, 0.0)]);
        }

        let mut out = PrivacyGuarantee::new();

        out.gaussian_dp = compose_gaussianDP(&curves)?;
        out.renyi_dp = compose_renyiDP(&curves)?;
        out.zcdp = compose_zCDP(&curves)?;
        out.approx_dp = compose_singleton_approxDP(&curves)?;

        if !out.has_representation() {
            return fallible!(
                FailedFunction,
                "PrivacyGuarantee composition has no supported common representation"
            );
        }

        Ok(out)
    }

    fn has_representation(&self) -> bool {
        self.approx_dp.is_some()
            || self.gaussian_dp.is_some()
            || self.profile.is_some()
            || self.tradeoff.is_some()
            || self.renyi_dp.is_some()
            || self.zcdp.is_some()
    }
}

/// Gaussian-DP composition is analytic: mu_total² = sum_i mu_i².
#[allow(non_snake_case)]
fn compose_gaussianDP(curves: &[PrivacyGuarantee]) -> Fallible<Option<f64>> {
    if curves.iter().any(|curve| curve.gaussian_dp.is_none()) {
        return Ok(None);
    }

    let mut sum_mu2 = DInterval::point(0.0)?;
    for curve in curves {
        let mu = curve.gaussian_dp.unwrap();
        check_mu(mu)?;
        let mu = DInterval::point(mu)?;
        sum_mu2 = (sum_mu2 + (mu.clone() * mu)?)?;
    }

    let mu = sum_mu2.sqrt()?.upper_f64()?;
    if !mu.is_finite() {
        return fallible!(Overflow, "composed Gaussian DP parameter is not finite");
    }
    check_mu(mu)?;
    Ok(Some(mu))
}

/// Native approximate-zCDP representations compose by adding rho and source delta.
///
/// This is the same theorem used by `CompositionMeasure for Approximate<zCDP>`.
/// No RDP-to-zCDP conversion is attempted.
#[allow(non_snake_case)]
fn compose_zCDP(curves: &[PrivacyGuarantee]) -> Fallible<Option<ZCDP>> {
    if curves.iter().any(|curve| curve.zcdp.is_none()) {
        return Ok(None);
    }

    let mut rho_sum = DInterval::point(0.0)?;
    let mut delta_sum = DInterval::point(0.0)?;
    let mut rho_is_infinite = false;

    for curve in curves {
        let ZCDP { rho, delta } = curve.zcdp.unwrap();
        check_rho(rho)?;
        check_delta(delta)?;

        if rho.is_infinite() {
            rho_is_infinite = true;
        } else {
            rho_sum = (rho_sum + DInterval::point(rho)?)?;
        }
        delta_sum = (delta_sum + DInterval::point(delta)?)?;
    }

    Ok(Some(ZCDP {
        rho: if rho_is_infinite {
            f64::INFINITY
        } else {
            rho_sum.upper_f64()?
        },
        delta: delta_sum.upper_f64()?.min(1.0),
    }))
}

#[derive(Clone)]
enum RdpComponent {
    Native(Arc<RenyiFn>),
    ZCDP(f64),
    Minimum { native: Arc<RenyiFn>, zcdp_rho: f64 },
}

/// Compose the tightest cheaply available RDP representation from each input.
///
/// Exact zCDP embeds into RDP as `epsilon(alpha) = rho * alpha`. Approximate
/// zCDP with nonzero source delta is deliberately not embedded: OpenDP has not
/// established that its representation-specific delta can be reinterpreted as
/// the source delta of the approximate-RDP representation. A native RDP
/// representation on the same input remains independently usable.
#[allow(non_snake_case)]
fn compose_renyiDP(curves: &[PrivacyGuarantee]) -> Fallible<Option<RenyiDP>> {
    let mut components = Vec::with_capacity(curves.len());
    let mut delta_sum = DInterval::point(0.0)?;

    for guarantee in curves {
        let component = match (&guarantee.renyi_dp, guarantee.zcdp) {
            (Some(RenyiDP { curve, delta }), Some(ZCDP { rho, delta: 0.0 })) => {
                // The exact zCDP guarantee remains valid after relaxing to
                // the native RDP representation's source delta, so both epsilon
                // bounds can be intersected at that delta.
                check_delta(*delta)?;
                check_rho(rho)?;
                delta_sum = (delta_sum + DInterval::point(*delta)?)?;
                RdpComponent::Minimum {
                    native: curve.clone(),
                    zcdp_rho: rho,
                }
            }
            (Some(RenyiDP { curve, delta }), _) => {
                check_delta(*delta)?;
                delta_sum = (delta_sum + DInterval::point(*delta)?)?;
                RdpComponent::Native(curve.clone())
            }
            (None, Some(ZCDP { rho, delta: 0.0 })) => {
                check_rho(rho)?;
                RdpComponent::ZCDP(rho)
            }
            // A nonzero approximate-zCDP delta is local to zCDP and is not
            // silently repurposed as an approximate-RDP delta.
            (None, Some(ZCDP { .. })) | (None, None) => return Ok(None),
        };
        components.push(component);
    }

    let curve = Arc::new(move |alpha: f64| -> Fallible<f64> {
        check_renyi_order(alpha)?;

        let mut sum = DInterval::point(0.0)?;
        for component in components.iter() {
            let epsilon = match component {
                RdpComponent::Native(native) => eval_native_rdp(native.as_ref(), alpha)?,
                RdpComponent::ZCDP(rho) => eval_zcdp_rdp(*rho, alpha)?,
                RdpComponent::Minimum { native, zcdp_rho } => {
                    eval_native_rdp(native.as_ref(), alpha)?.min(eval_zcdp_rdp(*zcdp_rho, alpha)?)
                }
            };

            if epsilon.is_infinite() {
                return Ok(f64::INFINITY);
            }
            sum = (sum + DInterval::point(epsilon)?)?;
        }
        sum.upper_f64()
    });

    Ok(Some(RenyiDP {
        curve,
        delta: delta_sum.upper_f64()?.min(1.0),
    }))
}

fn eval_native_rdp(curve: &RenyiFn, alpha: f64) -> Fallible<f64> {
    let epsilon = curve(alpha)?;
    check_rdp_epsilon(epsilon)?;
    Ok(epsilon)
}

fn eval_zcdp_rdp(rho: f64, alpha: f64) -> Fallible<f64> {
    if rho.is_infinite() {
        return Ok(f64::INFINITY);
    }
    (DInterval::point(alpha)? * DInterval::point(rho)?)?.upper_f64()
}

/// Basic composition for singleton approximate-DP representations.
#[allow(non_snake_case)]
fn compose_singleton_approxDP(
    curves: &[PrivacyGuarantee],
) -> Fallible<Option<Arc<[ApproxDPPoint]>>> {
    if curves
        .iter()
        .any(|curve| !matches!(curve.approx_dp.as_deref(), Some([_])))
    {
        return Ok(None);
    }

    let mut epsilon_sum = DInterval::point(0.0)?;
    let mut delta_sum = DInterval::point(0.0)?;

    for curve in curves {
        let [point] = curve.approx_dp.as_deref().unwrap() else {
            unreachable!()
        };
        epsilon_sum = (epsilon_sum + DInterval::point(point.epsilon)?)?;
        delta_sum = (delta_sum + DInterval::point(point.delta)?)?;
    }

    let epsilon_sum = epsilon_sum.upper_f64()?;
    if !epsilon_sum.is_finite() {
        return fallible!(Overflow, "composed epsilon is not finite");
    }

    Ok(Some(Arc::from(
        vec![ApproxDPPoint::build((
            epsilon_sum,
            delta_sum.upper_f64()?.min(1.0),
        ))?]
        .into_boxed_slice(),
    )))
}

fn check_rho(rho: f64) -> Fallible<()> {
    if rho.is_nan() {
        return fallible!(FailedMap, "rho must not be NaN");
    }
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({rho}) must be non-negative");
    }
    Ok(())
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

fn check_rdp_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() || epsilon.is_sign_negative() {
        return fallible!(
            FailedMap,
            "RDP epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }
    Ok(())
}

#[cfg(test)]
mod test;
