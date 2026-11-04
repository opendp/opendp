use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::error::ErrorVariant;

// Scalar mathematical fixtures below exercise the vector API with singleton queries.
fn batch(
    f: impl Fn(f64) -> Fallible<f64> + Send + Sync,
) -> impl Fn(&[f64]) -> Fallible<Vec<f64>> + Send + Sync {
    move |values| values.iter().map(|&value| f(value)).collect()
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
