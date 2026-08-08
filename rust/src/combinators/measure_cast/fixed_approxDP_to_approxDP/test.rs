use crate::{
    domains::{AtomDomain, MapDomain},
    measurements::make_laplace_threshold,
    measures::curves::ApproxDPPoint,
    metrics::L0PInfDistance,
    traits::{DInterval, InfCast, NextFloat},
};
use dashu::rational::RBig;

use super::*;

#[derive(Debug, PartialEq)]
enum ActiveBranch {
    FirstAffine,
    SecondAffine,
    Zero,
    Crossover,
}

struct CertifiedBeta {
    lower: f64,
    upper: f64,
    active: ActiveBranch,
}

fn certified_beta(epsilon: f64, delta: f64, alpha: f64) -> Fallible<CertifiedBeta> {
    let one = DInterval::point(1.0)?;
    let zero = DInterval::point(0.0)?;
    let alpha = DInterval::point(alpha)?;
    let one_minus_delta = one.clone().sub(DInterval::point(delta)?)?;
    let exp_epsilon = DInterval::point(epsilon)?.exp()?;
    let exp_negative_epsilon = DInterval::point(epsilon)?.neg()?.exp()?;

    let first = one_minus_delta
        .clone()
        .sub(exp_epsilon.mul(alpha.clone())?)?;
    let second = exp_negative_epsilon.mul(one_minus_delta.sub(alpha)?.max(zero.clone())?)?;
    let beta = first.clone().max(second.clone())?.max(zero.clone())?;

    let first_lo = first.lower_f64()?;
    let first_hi = first.upper_f64()?;
    let second_lo = second.lower_f64()?;
    let second_hi = second.upper_f64()?;
    let active = if first_lo > second_hi && first_lo > 0.0 {
        ActiveBranch::FirstAffine
    } else if second_lo > first_hi && second_lo > 0.0 {
        ActiveBranch::SecondAffine
    } else if first_hi <= 0.0 && second_hi <= 0.0 {
        ActiveBranch::Zero
    } else {
        ActiveBranch::Crossover
    };

    Ok(CertifiedBeta {
        lower: beta.lower_f64()?,
        upper: beta.upper_f64()?,
        active,
    })
}

#[test]
fn test_fixed_approxDP_to_approxDP() -> Fallible<()> {
    let meas_fixed = make_laplace_threshold(
        MapDomain::new(AtomDomain::<String>::default(), AtomDomain::new_non_nan()),
        L0PInfDistance::default(),
        1.,
        10,
        None,
    )?;
    let meas_smooth = make_fixed_approxDP_to_approxDP(meas_fixed.clone())?;

    let (eps, del) = meas_fixed.map(&(1, 1, 1))?;
    let profile = meas_smooth.map(&(1, 1, 1))?;

    assert_eq!(profile.delta(0.)?, 1.0);
    assert_eq!(profile.delta(eps.next_down_())?, 1.0);
    assert_eq!(profile.delta(eps)?, del);
    assert_eq!(profile.delta(eps.next_up_())?, del);
    assert_eq!(profile.epsilon(del)?, eps);
    assert_eq!(profile.epsilon(del.next_down())?, f64::INFINITY);

    // The interval oracle is directed at both endpoints, so its upper bound
    // certifies the exact transcendental expression from above. The lower
    // bound checks tightness independently. Four ulps cover only the final
    // rational-to-f64 cast and are not used as the correctness oracle.
    let cases = [
        (0.5, 0.1, 0.0, ActiveBranch::FirstAffine), // alpha = 0
        (0.5, 0.1, 1.0, ActiveBranch::Zero),        // alpha = 1
        (0.5, 0.1, 0.25, ActiveBranch::FirstAffine),
        (1.0, 0.0, 0.3, ActiveBranch::SecondAffine), // delta = 0
        (0.5, 0.99, 0.2, ActiveBranch::Zero),        // delta close to one
        (1e-6, 0.0, 0.5, ActiveBranch::SecondAffine), // small epsilon
        (20.0, 0.1, 0.99, ActiveBranch::Zero),       // large epsilon
        (1.0, 0.05, 0.1, ActiveBranch::FirstAffine), // exp(epsilon) sensitive
        (1.0, 0.0, 0.5, ActiveBranch::SecondAffine), // exp(-epsilon) sensitive
        (1.0, 0.0, 0.1, ActiveBranch::FirstAffine),
        (1.0, 0.0, 0.5, ActiveBranch::SecondAffine),
        (1.0, 0.99, 0.5, ActiveBranch::Zero),
        (1.0, 0.0, 0.2689414213699951, ActiveBranch::Crossover),
    ];

    for (epsilon, delta, alpha, expected_branch) in cases {
        let certified = certified_beta(epsilon, delta, alpha)?;
        assert_eq!(certified.active, expected_branch);

        let point = ApproxDPPoint::build((epsilon, delta))?;
        let reported = f64::neg_inf_cast(point.beta(&RBig::try_from(alpha)?))?;

        assert!(
            reported <= certified.upper,
            "reported beta {reported:?} exceeds certified upper {:?} for ({epsilon}, {delta}, {alpha})",
            certified.upper
        );
        let tolerance = 4.0 * f64::EPSILON * certified.upper.abs().max(1.0);
        assert!(
            certified.lower <= reported + tolerance,
            "reported beta {reported:?} is below certified lower {:?} for ({epsilon}, {delta}, {alpha})",
            certified.lower
        );
    }

    Ok(())
}
