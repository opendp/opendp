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
                    return fallible!(
                        FailedMap,
                        "RDP budget order must be finite and greater than one"
                    );
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
            Self::ApproxDP(epsilon, delta) => {
                PrivacyGuarantee::new().with_approx_dp(epsilon, delta)
            }
            Self::ZCDP(rho, delta) => PrivacyGuarantee::new().with_zCDP(rho, delta),
            Self::RenyiDP(order, epsilon, delta) => PrivacyGuarantee::new().with_renyiDP(
                move |orders| {
                    Ok(orders
                        .iter()
                        .map(|&a| if a <= order { epsilon } else { f64::INFINITY })
                        .collect())
                },
                delta,
            ),
        }
    }

    /// Check whether a guarantee certifies this budget.
    pub fn allows(&self, guarantee: &PrivacyGuarantee) -> Fallible<bool> {
        self.validate()?;
        match *self {
            Self::ApproxDP(epsilon, delta) => {
                Ok(delta == 1.0 || guarantee.delta(epsilon)? <= delta)
            }
            Self::ZCDP(rho, delta) => Ok(delta == 1.0
                || rho.is_infinite()
                || guarantee
                    .zcdp
                    .is_some_and(|repr| repr.rho <= rho && repr.source_delta <= delta)
                || guarantee
                    .exact_zcdp_rho()?
                    .is_some_and(|bound| bound <= rho)),
            Self::RenyiDP(order, epsilon, delta) => {
                if delta == 1.0 || epsilon.is_infinite() {
                    return Ok(true);
                }
                if let Some(rho) = guarantee.exact_zcdp_rho()?.filter(|rho| rho.is_finite()) {
                    if (SInterval::point(order)? * SInterval::point(rho)?)?.upper_f64()? <= epsilon
                    {
                        return Ok(true);
                    }
                }
                if let Some((pure_epsilon, pure_delta)) = guarantee.approx_dp() {
                    if pure_delta == 0.0
                        && super::renyidp::rdp_from_pureDP(order, pure_epsilon)? <= epsilon
                    {
                        return Ok(true);
                    }
                }
                if let Some(renyi) = &guarantee.renyi {
                    if renyi.source_delta <= delta {
                        return Ok(
                            super::renyidp::evaluate_renyi(renyi.curve.as_ref(), &[order])?[0]
                                <= epsilon,
                        );
                    }
                }
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pure_dp_certifies_concentrated_budgets() -> Fallible<()> {
        let guarantee = PrivacyGuarantee::new().with_approx_dp(1.0, 0.0)?;
        assert!(PrivacyBudget::ZCDP(0.47, 0.0).allows(&guarantee)?);
        assert!(!PrivacyBudget::ZCDP(0.46, 0.0).allows(&guarantee)?);
        assert!(PrivacyBudget::RenyiDP(2.0, 0.74, 0.0).allows(&guarantee)?);
        assert!(!PrivacyBudget::RenyiDP(2.0, 0.73, 0.0).allows(&guarantee)?);
        assert!(PrivacyBudget::RenyiDP(10.0, 1.0, 0.0).allows(&guarantee)?);
        let approximate = PrivacyGuarantee::new().with_approx_dp(1.0, 0.01)?;
        assert!(!PrivacyBudget::ZCDP(0.47, 0.01).allows(&approximate)?);
        Ok(())
    }
}
