use super::{PrivacyGuarantee, check_delta, check_epsilon, check_rho};
use crate::{error::Fallible, traits::SInterval};

/// A fixed privacy bound. Unlike a PrivacyGuarantee, a budget does not
/// contain an arbitrary callback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrivacyBudget {
    /// (epsilon, delta)-DP; delta zero gives pure DP.
    ApproxDP(f64, f64),
    /// (rho, delta)-approximate zCDP.
    ZCDP(f64, f64),
    /// (order, epsilon, delta)-approximate RDP at one fixed order.
    RenyiDP(f64, f64, f64),
}

impl PrivacyBudget {
    pub(crate) fn validate(&self) -> Fallible<()> {
        match *self {
            Self::ApproxDP(epsilon, delta) => {
                check_epsilon(epsilon)?;
                if !epsilon.is_finite() {
                    return fallible!(FailedMap, "budget epsilon must be finite");
                }
                check_delta(delta)
            }
            Self::ZCDP(rho, delta) => {
                check_rho(rho)?;
                check_delta(delta)
            }
            Self::RenyiDP(order, epsilon, delta) => {
                if !order.is_finite() || order <= 1.0 {
                    return fallible!(FailedMap, "RDP budget order must be finite and greater than one");
                }
                check_epsilon(epsilon)?;
                check_delta(delta)
            }
        }
    }

    /// Interpret this budget as a guarantee.
    pub fn to_guarantee(&self) -> Fallible<PrivacyGuarantee> {
        self.validate()?;
        match *self {
            Self::ApproxDP(epsilon, delta) => PrivacyGuarantee::new().with_approx_dp(epsilon, delta),
            Self::ZCDP(rho, delta) => PrivacyGuarantee::new().with_zCDP(rho, delta),
            Self::RenyiDP(order, epsilon, delta) => PrivacyGuarantee::new().with_renyiDP(
                move |orders| Ok(orders.iter().map(|&a| {
                    if a <= order { epsilon } else { f64::INFINITY }
                }).collect()),
                delta,
            ),
        }
    }

    /// Check whether a guarantee certifies this budget.
    pub fn allows(&self, guarantee: &PrivacyGuarantee) -> Fallible<bool> {
        self.validate()?;
        match *self {
            Self::ApproxDP(epsilon, delta) => Ok(delta == 1.0 || guarantee.delta(epsilon)? <= delta),
            Self::ZCDP(rho, delta) => Ok(delta == 1.0 || rho.is_infinite()
                || guarantee.zcdp.is_some_and(|repr| repr.rho <= rho && repr.source_delta <= delta)),
            Self::RenyiDP(order, epsilon, delta) => {
                if delta == 1.0 || epsilon.is_infinite() {
                    return Ok(true);
                }
                if let Some(zcdp) = guarantee.zcdp {
                    if zcdp.source_delta == 0.0 && zcdp.rho.is_finite()
                        && (SInterval::point(order)? * SInterval::point(zcdp.rho)?)?
                            .upper_f64()? <= epsilon
                    {
                        return Ok(true);
                    }
                }
                if let Some(renyi) = &guarantee.renyi {
                    if renyi.source_delta <= delta {
                        return Ok(super::renyidp::evaluate_renyi(renyi.curve.as_ref(), &[order])?[0] <= epsilon);
                    }
                }
                Ok(false)
            }
        }
    }
}
