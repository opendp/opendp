use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::error::ErrorVariant;

// Scalar mathematical fixtures below exercise the vector API with singleton queries.
pub(super) fn batch(
    f: impl Fn(f64) -> Fallible<f64> + Send + Sync,
) -> impl Fn(&[f64]) -> Fallible<Vec<f64>> + Send + Sync {
    move |values| values.iter().map(|&value| f(value)).collect()
}

#[test]
fn approximate_dp_profile_conversion_preserves_slack_equality_and_plateaus() -> Fallible<()> {
    for delta in [0.0, f64::from_bits(1), 0.1, 0.5, 1.0] {
        let profile = ApproxDPRepr::new(2.0, delta)?.into_profile()?;
        assert_eq!(
            profile.delta(&[0.0, 2.0_f64.next_down(), 2.0, f64::INFINITY])?,
            vec![1.0, 1.0, delta, delta]
        );
        assert_eq!(
            profile.epsilon(&[delta])?[0],
            if delta == 1.0 { 0.0 } else { 2.0 }
        );
        if delta > 0.0 {
            assert_eq!(profile.epsilon(&[delta.next_down()])?[0], f64::INFINITY);
        }
        if delta == 0.0 {
            assert_eq!(profile.pure_epsilon()?, Some(2.0));
        } else {
            assert_eq!(profile.pure_epsilon()?, None);
            assert!(profile.log_delta(&[2.0])?[0] >= delta.ln());
        }
    }
    Ok(())
}

#[test]
fn test_fixed_approx_dp_tradeoff_is_conservative() -> Fallible<()> {
    for (epsilon, delta) in [(1.0_f64, 0.1_f64), (2.0, 0.01)] {
        let guarantee = PrivacyGuarantee::new().with_approx_dp(epsilon, delta)?;
        let alpha = 0.2;
        let expected = (0.0_f64)
            .max(1.0 - delta - epsilon.exp() * alpha)
            .max((-epsilon).exp() * (1.0 - delta - alpha));
        let beta = guarantee.beta(&[alpha])?[0];
        assert!(beta <= expected);
        assert!(beta > 0.0);
        // Queries use the original certificate, never the rounded beta callback.
        assert_eq!(guarantee.delta(epsilon)?, delta);
        assert_eq!(guarantee.epsilon(delta)?, epsilon);
    }
    Ok(())
}

#[test]
fn test_ordinary_callback_conversion_is_conservative_and_lazy() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let profile = PrivacyProfile::new(batch(move |epsilon| {
        counter.fetch_add(1, Ordering::Relaxed);
        Ok((-epsilon).exp())
    }));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let target = profile.delta(&[0.25])?[0];
    assert!(target >= (-0.25f64).exp());
    let epsilon = profile.epsilon(&[target])?[0];
    assert!(epsilon <= 0.25 + 1e-12);
    assert!(epsilon >= 0.25 - 1e-12);
    for value in [0.0, f64::from_bits(1), 0.1, 0.5, 1.0] {
        let profile = PrivacyProfile::new(batch(move |_| Ok(value)));
        assert!(profile.delta(&[0.0])?[0] >= value);
        assert!(profile.log_delta(&[0.0])?[0] >= delta_to_log_lower_unchecked(value)?);
        if value > 0.0 {
            assert_eq!(profile.epsilon(&[value.next_down()])?[0], f64::INFINITY);
        }
    }
    // Log conversion may widen an ordinary plateau, never weaken its target.
    let plateau = PrivacyProfile::new(batch(|epsilon| Ok(if epsilon < 2.0 { 1.0 } else { 0.1 })));
    assert_eq!(plateau.epsilon(&[0.1])?[0], f64::INFINITY);
    assert_eq!(
        plateau.epsilon(&[plateau.delta(&[2.0])?[0].next_up()])?[0],
        2.0
    );
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_log_inversion_extremes_plateaus_and_endpoints() -> Fallible<()> {
    let profile =
        PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(|epsilon| Ok(-epsilon)))?;
    for target in [
        -f64::MAX,
        -1e300,
        -10000.0,
        -1000.0,
        -1.0,
        -f64::from_bits(1),
        -0.0,
    ] {
        assert_eq!(profile.epsilon_from_log_delta(&[target])?[0], -target);
        assert_eq!(profile.log_delta(&[-target])?[0], target);
    }
    assert_eq!(profile.epsilon(&[0.0])?[0], f64::INFINITY);
    assert_eq!(profile.log_delta(&[f64::INFINITY])?[0], f64::NEG_INFINITY);
    assert_eq!(profile.delta(&[f64::INFINITY])?[0], 0.0);
    for epsilon in [1000.0, 10000.0, f64::MAX] {
        assert_eq!(profile.delta(&[epsilon])?[0], f64::from_bits(1));
    }
    let plateau = PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(|epsilon| {
        Ok(if epsilon < 2.0 {
            0.0
        } else if epsilon < 4.0 {
            -1000.0
        } else {
            f64::NEG_INFINITY
        })
    }))?;
    assert_eq!(plateau.epsilon_from_log_delta(&[-1000.0])?[0], 2.0);
    assert_eq!(
        plateau.epsilon_from_log_delta(&[(-1000.0f64).next_down()])?[0],
        4.0
    );
    assert_eq!(plateau.epsilon(&[0.0])?[0], 4.0);
    assert_eq!(plateau.pure_epsilon()?, Some(4.0));
    assert_eq!(plateau.epsilon(&[1.0])?[0], 0.0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_source_slack_equality() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let profile = PrivacyProfile::new_log(batch(move |epsilon| {
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(if epsilon < 7.0 {
            0.0
        } else {
            f64::NEG_INFINITY
        })
    }));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    for slack in [0.0, f64::from_bits(1), 0.1, 0.5] {
        let slack_profile = profile.clone().with_delta_slack(slack)?;
        assert_eq!(slack_profile.epsilon(&[slack])?[0], 7.0);
        if slack > 0.0 {
            let before = calls.load(Ordering::Relaxed);
            assert_eq!(
                slack_profile.epsilon(&[slack.next_down()])?[0],
                f64::INFINITY
            );
            assert_eq!(calls.load(Ordering::Relaxed), before);
            assert_eq!(slack_profile.pure_epsilon()?, None);
        }
    }
    assert_eq!(profile.with_delta_slack(1.0)?.epsilon(&[1.0])?[0], 0.0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_slack_rounding_and_log_budgets() -> Fallible<()> {
    let profile = PrivacyProfile::new(batch(|_| Ok(1.0)))
        .with_log_profile(batch(|epsilon| Ok(-epsilon)))?
        .with_delta_slack(0.1)?;
    for epsilon in [0.0, 1.0, 10.0, 1000.0, f64::MAX, f64::INFINITY] {
        let core = if epsilon == f64::INFINITY {
            SInterval::point(0.0)?
        } else {
            SInterval::point((-epsilon).max(-1024.0))?.exp()?
        };
        let total = (SInterval::point(0.1)? + core)?.min(SInterval::point(1.0)?);
        assert!(profile.delta(&[epsilon])?[0] >= total.lower_f64()?);
        assert!(profile.log_delta(&[epsilon])?[0] >= total.ln()?.lower_f64()?);
    }
    assert_eq!(profile.delta(&[f64::INFINITY])?[0], 0.1);
    assert_eq!(profile.epsilon(&[0.1])?[0], f64::INFINITY);
    assert_eq!(profile.epsilon(&[0.1f64.next_down()])?[0], f64::INFINITY);
    assert_eq!(
        profile.epsilon_from_log_delta(&[-10000.0])?[0],
        f64::INFINITY
    );
    for target in [0.1f64.next_up(), 0.2, 0.5, 1.0] {
        let epsilon = profile.epsilon(&[target])?[0];
        let remaining = (SInterval::point(target)? - SInterval::point(0.1)?)?;
        if target != 1.0 {
            assert!(epsilon >= -remaining.ln()?.upper_f64()?);
        }
    }
    for target in [-2.0, -1.0, 0.0] {
        let epsilon = profile.epsilon_from_log_delta(&[target])?[0];
        if target < 0.0 {
            let total = (SInterval::point(0.1)? + SInterval::point(-epsilon)?.exp()?)?;
            assert!(total.upper() <= SInterval::point(target)?.exp()?.lower());
        }
    }
    let tiny_core = PrivacyProfile::new(batch(|_| Ok(1.0)))
        .with_log_profile(batch(|_| Ok(-f64::MAX)))?
        .with_delta_slack(0.5)?;
    assert_eq!(tiny_core.delta(&[0.0])?[0], 0.5_f64.next_up());
    let zero_core = tiny_core.with_log_profile(batch(|_| Ok(f64::NEG_INFINITY)))?;
    assert_eq!(zero_core.delta(&[0.0])?[0], 0.5);
    // Both zeros survive exact source-slack subtraction; positive core underflow does not.
    let zero = PrivacyProfile::new(batch(|_| Ok(0.0))).with_delta_slack(f64::from_bits(1))?;
    assert_eq!(zero.epsilon(&[f64::from_bits(1)])?[0], 0.0);
    assert_eq!(zero.delta(&[0.0])?[0], f64::from_bits(1));
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_directed_rounding_against_512_bit_reference() -> Fallible<()> {
    use rug::{Float, float::Round, ops::AddAssignRound};
    for slack in [0.0, f64::from_bits(1), 0.1, 1.0_f64.next_down()] {
        let profile = PrivacyProfile::new(batch(|_| Ok(1.0)))
            .with_log_profile(batch(|epsilon| Ok(-epsilon)))?
            .with_delta_slack(slack)?;
        for epsilon in [0.0, 0.1, 1.0, 40.0, 740.0, 1000.0] {
            let mut reference = Float::with_val(512, -epsilon);
            reference.exp_round(Round::Up);
            reference.add_assign_round(slack, Round::Up);
            if reference > 1.0 {
                reference = Float::with_val(512, 1);
            }
            assert!(Float::with_val(512, profile.delta(&[epsilon])?[0]) >= reference);
            if slack == 0.0 {
                assert_eq!(profile.log_delta(&[epsilon])?[0], -epsilon);
            } else {
                reference.ln_round(Round::Up);
                assert!(
                    Float::with_val(512, profile.log_delta(&[epsilon])?[0]) >= reference,
                    "slack={slack}, epsilon={epsilon}"
                );
            }
        }
        for target in [0.1_f64.next_up(), 0.5, 1.0_f64.next_down()] {
            if target <= slack {
                continue;
            }
            let mut reference = Float::with_val(512, target);
            reference.add_assign_round(-slack, Round::Down);
            reference.ln_round(Round::Down);
            reference = -reference;
            assert!(Float::with_val(512, profile.epsilon(&[target])?[0]) >= reference);
        }
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_log_target_rounding_is_never_weakened() -> Fallible<()> {
    let target = 0.1_f64;
    let upper = delta_to_log_upper_unchecked(target)?;
    let lower = delta_to_log_lower_unchecked(target)?;
    assert!(SInterval::point(upper)?.exp()?.lower() > SInterval::point(target)?.upper());
    let profile =
        PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(move |epsilon| {
            Ok(if epsilon < 2.0 {
                0.0
            } else if epsilon < 4.0 {
                upper
            } else if epsilon < 6.0 {
                lower
            } else {
                f64::NEG_INFINITY
            })
        }))?;
    assert_eq!(profile.epsilon(&[target])?[0], 4.0);
    assert_eq!(profile.epsilon_from_log_delta(&[upper])?[0], 2.0);
    assert_eq!(profile.epsilon(&[0.0])?[0], 6.0);
    assert!(profile.delta(&[4.0])?[0] <= target);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_setters_retain_profile_slack() -> Fallible<()> {
    let profile = PrivacyProfile::new(batch(|_| Ok(1.0))).with_delta_slack(0.25)?;
    for profile in [
        profile.clone().with_profile(batch(|_| Ok(0.0)))?,
        profile.with_log_profile(batch(|_| Ok(f64::NEG_INFINITY)))?,
    ] {
        assert_eq!(profile.delta(&[0.0])?[0], 0.25);
        assert_eq!(profile.epsilon(&[0.25])?[0], 0.0);
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_intermediate_overflow_does_not_establish_comparison() -> Fallible<()> {
    use crate::traits::CInterval;
    for sign in [-1.0, 1.0] {
        let profile =
            PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(move |epsilon| {
                if epsilon < 2.0 {
                    return Ok(0.0);
                }
                let intermediate = (CInterval::point(sign * f64::MAX)? * CInterval::point(2.0)?)?;
                (-intermediate).upper_f64()
            }))?;
        for target in [0.1, 0.0] {
            assert_eq!(
                profile.epsilon(&[target]).unwrap_err().variant,
                ErrorVariant::Overflow
            );
        }
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_errors_and_invalid_values_propagate() -> Fallible<()> {
    for variant in ["Overflow", "NumericError", "FailedFunction", "Search"] {
        for fail_at_zero in [true, false] {
            let profile = PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(
                move |epsilon| {
                    if fail_at_zero || epsilon >= 2.0 {
                        return Err(crate::error::Error {
                            variant: variant.parse().unwrap(),
                            ..err!(FailedMap, "unchanged failure")
                        });
                    }
                    Ok(0.0)
                },
            ))?;
            let error = profile.epsilon(&[0.1]).unwrap_err();
            assert_eq!(error.variant.to_string(), variant);
            assert_eq!(error.message.as_deref(), Some("unchanged failure"));
        }
    }
    let bisection_error =
        PrivacyProfile::new(batch(|_| Ok(1.0))).with_log_profile(batch(|epsilon| {
            if epsilon == 1.5 {
                return fallible!(Search, "bisection callback failure");
            }
            Ok(if epsilon < 1.5 { 0.0 } else { -1.0 })
        }))?;
    let error = bisection_error.epsilon_from_log_delta(&[-1.0]).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Search);
    assert_eq!(error.message.as_deref(), Some("bisection callback failure"));
    for invalid in [f64::NAN, -1.0, f64::INFINITY, 1.1] {
        assert!(
            PrivacyProfile::new(batch(|_| Ok(0.0)))
                .with_delta_slack(invalid)
                .is_err()
        );
        assert!(
            PrivacyProfile::new(batch(|_| Ok(0.0)))
                .epsilon(&[invalid])
                .is_err()
        );
        assert!(
            PrivacyProfile::new(batch(move |_| Ok(invalid)))
                .delta(&[0.0])
                .is_err()
        );
    }
    for invalid in [f64::NAN, 1.0, f64::INFINITY] {
        let profile = PrivacyProfile::new(batch(|_| Ok(1.0)))
            .with_log_profile(batch(move |_| Ok(invalid)))?;
        assert!(profile.delta(&[0.0]).is_err());
        assert!(profile.epsilon(&[0.1]).is_err());
        assert!(profile.epsilon_from_log_delta(&[invalid]).is_err());
    }
    for invalid in [f64::NAN, -1.0, f64::NEG_INFINITY] {
        let profile = PrivacyProfile::new_log(batch(|_| Ok(0.0)));
        assert!(profile.delta(&[invalid]).is_err());
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_vector_callbacks_receive_whole_batches_once() -> Fallible<()> {
    for is_log in [false, true] {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let callback = move |epsilon: &[f64]| {
            counter.fetch_add(1, Ordering::Relaxed);
            assert_eq!(epsilon, &[0.0, 1.0, 1.0, 10000.0]);
            Ok(epsilon
                .iter()
                .map(|&x| if is_log { -x } else { (-x).exp() })
                .collect())
        };
        let profile = if is_log {
            PrivacyProfile::new(|x| Ok(vec![1.0; x.len()])).with_log_profile(callback)?
        } else {
            PrivacyProfile::new(callback)
        };
        let inputs = &[0.0, 1.0, 1.0, 10000.0];
        let values = profile.delta(inputs)?;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(values.len(), inputs.len());
        assert_eq!(values[1], values[2]);
        assert_eq!(profile.log_delta(inputs)?.len(), inputs.len());
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
    Ok(())
}

#[test]
fn test_scalar_and_vector_queries_agree() -> Fallible<()> {
    let profile = PrivacyProfile::new_log(batch(|epsilon| Ok(-epsilon)));
    for (query, inputs) in [
        (
            PrivacyProfile::delta as fn(&PrivacyProfile, &[f64]) -> _,
            vec![0.0, 1.0, 1.0, 10000.0, f64::INFINITY],
        ),
        (
            PrivacyProfile::log_delta,
            vec![0.0, 1.0, 1.0, 10000.0, f64::INFINITY],
        ),
        (PrivacyProfile::epsilon, vec![0.0, 0.1, 0.1, 1.0]),
        (
            PrivacyProfile::epsilon_from_log_delta,
            vec![f64::NEG_INFINITY, -10000.0, -1.0, -1.0, 0.0],
        ),
    ] {
        let scalars = inputs
            .iter()
            .map(|&value| Ok(query(&profile, &[value])?[0]))
            .collect::<Fallible<Vec<_>>>()?;
        assert_eq!(query(&profile, &inputs)?, scalars);
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_vector_validation_and_callback_lengths() -> Fallible<()> {
    let profile =
        PrivacyProfile::new(|_| panic!("unsorted input must be rejected before callback"));
    for query in [
        PrivacyProfile::delta,
        PrivacyProfile::log_delta,
        PrivacyProfile::epsilon,
        PrivacyProfile::epsilon_from_log_delta,
    ] {
        assert!(query(&profile, &[0.5, 0.1]).is_err());
    }
    assert!(profile.epsilon_from_log_delta(&[-1.0, -2.0]).is_err());
    for length in [0, 1, 3] {
        let profile = PrivacyProfile::new(move |_| Ok(vec![0.0; length]));
        assert!(
            profile
                .delta(&[0.0, 1.0])
                .unwrap_err()
                .message
                .unwrap()
                .contains("output length")
        );
        let profile = profile.with_log_profile(move |_| Ok(vec![0.0; length]))?;
        assert!(profile.log_delta(&[0.0, 1.0]).is_err());
        if length != 1 {
            // Generic inversion evaluates the forward callback with singletons.
            assert!(profile.epsilon(&[0.1, 0.2]).is_err());
            assert!(profile.epsilon_from_log_delta(&[-2.0, -1.0]).is_err());
        }
    }
    let profile = PrivacyProfile::new(|inputs| {
        assert!(inputs.len() <= 1);
        Ok(inputs
            .iter()
            .map(|&x| if x < 2.0 { 1.0 } else { 0.0 })
            .collect())
    });
    assert_eq!(
        profile.epsilon(&[0.0, 0.0, 0.5, 1.0])?,
        vec![2.0, 2.0, 2.0, 0.0]
    );
    assert!(profile.delta(&[])?.is_empty());
    assert!(profile.log_delta(&[])?.is_empty());
    assert!(profile.epsilon(&[])?.is_empty());
    assert!(profile.epsilon_from_log_delta(&[])?.is_empty());
    Ok(())
}

#[test]
fn test_alpha_inverse_uses_strict_lower_witnesses() -> Fallible<()> {
    use crate::traits::InfCast;
    use dashu::rational::RBig;

    for slope in [1.0, 2.0, 3.0, 7.0, f64::MAX] {
        let lower = move |alpha: f64| {
            let value = (RBig::from(1) - RBig::try_from(slope)? * RBig::try_from(alpha)?)
                .max(RBig::from(0));
            f64::neg_inf_cast(value)
        };
        for target in [0.0, 0.125, 0.5, 1.0] {
            let reference = (RBig::from(1) - RBig::try_from(target)?) / RBig::try_from(slope)?;
            let alpha = invert_beta_callback(lower, target)?;
            assert!(
                RBig::try_from(alpha)? <= reference,
                "alpha={alpha} exceeds crossing {reference}, slope={slope}, target={target}"
            );
            if alpha > 0.0 {
                assert!(lower(alpha)? > target);
            }
            if target == 0.0 {
                assert!(RBig::try_from(alpha.next_up())? >= reference);
            }
        }
    }
    Ok(())
}

#[test]
fn test_alpha_inverse_endpoints_plateaus_and_domain() -> Fallible<()> {
    let plateau = |alpha: f64| {
        assert!((0.0..=1.0).contains(&alpha));
        Ok(if alpha < 0.25 {
            1.0
        } else if alpha < 0.75 {
            0.5
        } else {
            0.0
        })
    };
    // Equality must not certify a point deep within a plateau as its left edge.
    assert_eq!(invert_beta_callback(plateau, 0.5)?, 0.25_f64.next_down());
    assert_eq!(invert_beta_callback(plateau, 0.0)?, 0.75_f64.next_down());
    assert_eq!(invert_beta_callback(plateau, 1.0)?, 0.0);
    assert_eq!(invert_beta_callback(|_| Ok(0.0), 0.0)?, 0.0);
    assert_eq!(invert_beta_callback(|_| Ok(0.75), 0.5)?, 1.0);

    for edge in [f64::from_bits(1), f64::MIN_POSITIVE, 0.5, 1.0] {
        let alpha =
            invert_beta_callback(move |alpha| Ok(if alpha < edge { 1.0 } else { 0.0 }), 0.0)?;
        assert_eq!(alpha, edge.next_down().max(0.0));
    }
    // Epsilon inversion must keep its opposite, passing endpoint convention.
    assert_eq!(invert_decreasing_callback(plateau, 0.5)?, 0.25);
    Ok(())
}

#[test]
fn test_alpha_inverse_preserves_errors_and_validates_values() {
    for variant in ["Overflow", "NumericError", "FailedFunction"] {
        for fail_at_zero in [false, true] {
            let error = invert_beta_callback(
                |alpha| {
                    if fail_at_zero || (0.25..0.75).contains(&alpha) {
                        return Err(crate::error::Error {
                            variant: variant.parse().unwrap(),
                            ..err!(FailedMap, "alpha callback failure")
                        });
                    }
                    Ok(1.0 - alpha)
                },
                0.5,
            )
            .unwrap_err();
            assert_eq!(error.variant.to_string(), variant);
            assert_eq!(error.message.as_deref(), Some("alpha callback failure"));
        }
    }
    for invalid in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert_eq!(
            invert_beta_callback(move |_| Ok(invalid), 0.0)
                .unwrap_err()
                .variant,
            ErrorVariant::FailedMap
        );
    }
}

#[test]
fn test_rounded_derived_bounds_do_not_enter_convex_optimizer() -> Fallible<()> {
    use dashu::rational::RBig;
    let guarantee = PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?;
    let alphas = [0.5_f64.next_down(), 0.5, 0.5_f64.next_up()];
    let values = guarantee.beta(&alphas)?;
    let [a, b, c] = alphas.map(|a| RBig::try_from(a).unwrap());
    let [fa, fb, fc] = [values[0], values[1], values[2]].map(|v| RBig::try_from(v).unwrap());
    // These rounded samples are not convex. Delta and epsilon must instead
    // query the source certificate, without constructing another representation.
    assert!((&fb - &fa) / (&b - &a) > (&fc - &fb) / (&c - &b));
    assert_eq!(guarantee.delta(0.0)?, 0.0);
    assert_eq!(guarantee.epsilon(0.0)?, 0.0);
    Ok(())
}

#[test]
fn test_reporting_vector_validation_and_shapes() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let profile = PrivacyProfile::new_log(move |epsilon| {
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(vec![f64::NEG_INFINITY; epsilon.len()])
    });
    let profile = PrivacyGuarantee::new().with_profile(profile);
    let certificate = PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?;
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    for guarantee in [&profile, &certificate] {
        let beta = guarantee.beta(&[0.0, 0.25, 0.25, 1.0])?;
        assert_eq!(beta.len(), 4);
        assert_eq!(beta[1], beta[2]);
        for (actual, expected) in beta.into_iter().zip([1.0, 0.75, 0.75, 0.0]) {
            assert!((actual - expected).abs() < 1e-14);
        }
        let alpha = guarantee.alpha(&[0.25, 0.5, 0.5, 1.0])?;
        assert_eq!(alpha.len(), 4);
        assert_eq!(alpha[1], alpha[2]);
        for (actual, expected) in alpha.into_iter().zip([0.75, 0.5, 0.5, 0.0]) {
            assert!((actual - expected).abs() < 1e-14);
        }
        assert!(guarantee.beta(&[])?.is_empty());
        assert!(guarantee.alpha(&[])?.is_empty());
        let before = calls.load(Ordering::Relaxed);
        for inputs in [
            vec![0.5, 0.25],
            vec![f64::NAN],
            vec![-0.1],
            vec![1.1],
            vec![f64::INFINITY],
        ] {
            assert!(guarantee.beta(&inputs).is_err());
            assert!(guarantee.alpha(&inputs).is_err());
        }
        assert_eq!(calls.load(Ordering::Relaxed), before);
    }
    // Reporting must propagate malformed source-profile results.
    for length in [0, 2] {
        let guarantee = PrivacyGuarantee::new()
            .with_profile(PrivacyProfile::new_log(move |_| Ok(vec![0.0; length])));
        assert!(guarantee.beta(&[0.5]).is_err());
        assert!(guarantee.alpha(&[0.5]).is_err());
    }
    for value in [f64::NAN, f64::INFINITY, 0.1] {
        let guarantee =
            PrivacyGuarantee::new().with_profile(PrivacyProfile::new_log(move |epsilon| {
                Ok(vec![value; epsilon.len()])
            }));
        assert!(guarantee.beta(&[0.5]).is_err());
        assert!(guarantee.alpha(&[0.5]).is_err());
    }
    Ok(())
}

#[test]
fn test_pure_dp_profile_tradeoff_oracle() -> Fallible<()> {
    let epsilon_0 = 2.0_f64.ln();
    let profile = PrivacyProfile::new(batch(move |epsilon| {
        Ok((1.0 - (epsilon - epsilon_0).exp()).max(0.0))
    }));
    let guarantee = PrivacyGuarantee::new().with_profile(profile.clone());
    assert!((guarantee.beta(&[0.25])?[0] - 0.5).abs() < 1e-10);
    assert!((guarantee.beta(&[0.75])?[0] - 0.125).abs() < 1e-10);
    assert!(profile.delta(&[epsilon_0])?[0] <= 1e-12);
    // Fixed certificates stay in their source representation; no rounded
    // derived callback is handed to the convex optimizer.
    let certificate = PrivacyGuarantee::new().with_approx_dp(epsilon_0, 0.0)?;
    assert_eq!(certificate.delta(epsilon_0)?, 0.0);
    assert!((certificate.beta(&[0.25])?[0] - 0.5).abs() < 1e-10);
    Ok(())
}

#[test]
fn test_alpha_lower_endpoint_in_source_representations() -> Fallible<()> {
    use dashu::rational::RBig;
    let certificate = PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?;
    let profile = PrivacyGuarantee::new()
        .with_profile(PrivacyProfile::new_log(batch(|_| Ok(f64::NEG_INFINITY))));
    for guarantee in [certificate, profile] {
        for target in [0.0, 0.125, 0.5, 1.0] {
            let alpha = guarantee.alpha(&[target])?[0];
            let reference = RBig::from(1) - RBig::try_from(target)?;
            assert!(RBig::try_from(alpha)? <= reference);
            assert!(reference - RBig::try_from(alpha)? < RBig::try_from(1e-14)?);
        }
    }
    Ok(())
}

#[test]
fn test_aggregate_reporting_is_conjunctive() -> Fallible<()> {
    let profile = PrivacyProfile::new_log(batch(|_| Ok(-1.0)));
    let profile_only = PrivacyGuarantee::new().with_profile(profile.clone());
    let certificate_only = PrivacyGuarantee::new().with_approx_dp(1.0, 0.1)?;
    let combined = PrivacyGuarantee::new()
        .with_profile(profile)
        .with_approx_dp(1.0, 0.1)?;
    for source in [profile_only, certificate_only] {
        assert!(combined.delta(1.0)? <= source.delta(1.0)?);
        assert!(combined.epsilon(0.5)? <= source.epsilon(0.5)?);
        assert!(combined.beta(&[0.25])?[0] >= source.beta(&[0.25])?[0]);
        assert!(combined.alpha(&[0.0])?[0] >= source.alpha(&[0.0])?[0]);
    }
    Ok(())
}

#[test]
fn test_aggregate_queries_preserve_numerical_failures() -> Fallible<()> {
    for variant in ["NumericError", "Overflow", "FailedFunction"] {
        let profile = PrivacyProfile::new(move |_| {
            Err(crate::error::Error {
                variant: variant.parse().unwrap(),
                ..err!(FailedMap, "profile failed")
            })
        });
        let guarantee = PrivacyGuarantee::new()
            .with_approx_dp(1.0, 0.1)?
            .with_profile(profile);
        for result in [
            guarantee.delta(1.0),
            guarantee.epsilon(0.5),
            guarantee.beta(&[0.5]).map(|v| v[0]),
            guarantee.alpha(&[0.5]).map(|v| v[0]),
        ] {
            assert_eq!(result.unwrap_err().variant.to_string(), variant);
        }
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_representation_queries_and_aggregation() -> Fallible<()> {
    let exact = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.0)?;
    let approximate = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.1)?;

    assert_eq!(exact.delta(f64::INFINITY)?, 0.0);
    assert_eq!(approximate.delta(f64::INFINITY)?, 0.1);
    assert_eq!(exact.beta(&[0.5])?[0], 0.5);
    let alpha = exact.alpha(&[0.5])?[0];
    assert!(alpha < 0.5 && alpha > 0.5 - 1e-14);
    assert!(exact.beta(&[alpha])?[0] > 0.5);

    let profile = ApproxDPRepr::new(1.0, 0.2)?.into_profile()?;
    let profile_only = PrivacyGuarantee::new().with_profile(profile.clone());
    let rdp_only = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.0)?;
    let combined = PrivacyGuarantee::new()
        .with_profile(profile.clone())
        .with_renyiDP(|_| Ok(0.0), 0.0)?;

    assert!(combined.delta(1.0)? <= profile_only.delta(1.0)?);
    assert!(combined.delta(1.0)? <= rdp_only.delta(1.0)?);
    assert!(combined.epsilon(0.2)? <= profile_only.epsilon(0.2)?);
    assert!(combined.epsilon(0.2)? <= rdp_only.epsilon(0.2)?);
    assert!(combined.beta(&[0.5])?[0] >= profile_only.beta(&[0.5])?[0]);
    assert!(combined.beta(&[0.5])?[0] >= rdp_only.beta(&[0.5])?[0]);
    assert!(combined.alpha(&[0.5])?[0] >= profile_only.alpha(&[0.5])?[0]);
    assert!(combined.alpha(&[0.5])?[0] >= rdp_only.alpha(&[0.5])?[0]);

    let profile_only = PrivacyGuarantee::new().with_profile(profile);
    let failing_rdp =
        profile_only.with_renyiDP(|_| fallible!(FailedFunction, "RDP failed"), 0.0)?;
    assert_eq!(
        failing_rdp.delta(1.0).unwrap_err().variant,
        ErrorVariant::FailedFunction
    );
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_high_order_delta_search() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::new().with_renyiDP(|_| Ok(1.0), 0.0)?;
    let delta = guarantee.delta(1.0)?;

    // The order-1024 result is around 1e-3; high-order probing should find
    // the substantially tighter backend-supported bound.
    assert!(delta < 1e-12, "delta={delta}");
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_aggregate_queries_preserve_renyi_numerical_failure() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::new()
        .with_profile(PrivacyProfile::new(batch(|_| Ok(0.0))))
        .with_renyiDP(|_| fallible!(NumericError, "RDP callback failed"), 0.0)?;
    for result in [
        guarantee.delta(1.0),
        guarantee.epsilon(0.5),
        guarantee.beta(&[0.5]).map(|v| v[0]),
        guarantee.alpha(&[0.5]).map(|v| v[0]),
    ] {
        assert_eq!(result.unwrap_err().variant, ErrorVariant::NumericError);
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_delta_search_propagates_unevaluable_order() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = calls.clone();
    let guarantee = PrivacyGuarantee::new().with_renyiDP(
        move |order| {
            callback_calls.fetch_add(1, Ordering::Relaxed);
            if order > 1_000.0 {
                fallible!(FailedFunction, "RDP order is unevaluable")
            } else {
                Ok(1.0)
            }
        },
        0.0,
    )?;

    let error = guarantee.delta(1.0).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::FailedFunction);
    assert_eq!(error.message.as_deref(), Some("RDP order is unevaluable"));
    assert!(calls.load(Ordering::Relaxed) > 0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_nonzero_curve_exercises_all_queries() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = calls.clone();
    let guarantee = PrivacyGuarantee::new().with_renyiDP(
        move |order| {
            callback_calls.fetch_add(1, Ordering::Relaxed);
            Ok(0.01 + 0.05 * order)
        },
        0.0,
    )?;

    let delta = guarantee.delta(1.0)?;
    let epsilon = guarantee.epsilon(1.0)?;
    let beta = guarantee.beta(&[1.0])?[0];
    let alpha = guarantee.alpha(&[1.0])?[0];
    for value in [delta, epsilon, beta, alpha] {
        assert!(!value.is_nan());
        assert!(value >= 0.0);
    }
    assert!(calls.load(Ordering::Relaxed) > 0);
    Ok(())
}
