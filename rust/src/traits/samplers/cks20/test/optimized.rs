use crate::{
    error::Fallible,
    traits::samplers::{
        EntropySource,
        cks20::*,
        entropy::test::{TestBuffered, TestStream},
    },
};
use dashu::{
    base::Abs,
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};

fn words(words: &[u64]) -> Vec<u8> {
    words.iter().flat_map(|x| x.to_ne_bytes()).collect()
}

#[test]
fn gaussian_integer_exponent_matches_original_rational_expression() -> Fallible<()> {
    for scale in [
        rbig!(1 / 10),
        rbig!(1 / 3),
        rbig!(3 / 2),
        rbig!(10),
        RBig::from(UBig::ONE << 1074),
    ] {
        let envelope = GaussianEnvelope::new(scale.clone())?;
        let t = RBig::from(scale.clone().floor() + 1i8);
        let sigma2 = scale.pow(2);
        let large = t.clone().floor();
        for candidate in [
            IBig::ZERO,
            1.into(),
            (-1).into(),
            7.into(),
            (-7).into(),
            large.clone(),
            -large,
        ] {
            let expected = (candidate.clone().abs() - &sigma2 / &t).pow(2) / (&sigma2 * rbig!(2));
            let actual = RBig::from_parts(
                envelope.bias_numerator(&candidate).into(),
                envelope.denominator.clone(),
            );
            assert_eq!(actual, expected);
        }
    }
    Ok(())
}

#[test]
fn laplace_rejects_negative_zero_and_preserves_nonzero_signs() -> Fallible<()> {
    let mut script = vec![0]; // negative sign, then magnitude zero
    script.extend(words(&[1])); // exp(-1) fails, geometric integer part is zero
    script.push(1); // positive sign on a fresh proposal
    script.extend(words(&[1]));
    let mut source = TestStream::from_bytes(script);
    assert_eq!(
        sample_discrete_laplace_with(RBig::ONE, &mut source)?,
        IBig::ZERO
    );
    assert_eq!(source.position, 18); // must redraw magnitude, not just sign

    for positive in [false, true] {
        let mut script = vec![u8::from(positive)];
        // exp(-1) succeeds once (1/2 succeeds, 1/3 fails), then fails.
        script.extend(words(&[0, 1, 1]));
        let mut source = TestStream::from_bytes(script);
        let out = sample_discrete_laplace_with(RBig::ONE, &mut source)?;
        assert_eq!(out, IBig::from(if positive { 1 } else { -1 }));
        assert_eq!(source.position, 25);
    }
    Ok(())
}

#[test]
fn gaussian_rejection_restarts_and_late_entropy_failure_propagates() -> Fallible<()> {
    // sigma=1, t=2: propose zero; its acceptance exponent is exactly 1/8.
    let mut first = vec![1, 0]; // positive sign, u=0 for geometric(rate=1/2)
    first.extend(words(&[1, 0, 1])); // v=0, then exp(-1/8) fails
    let mut script = first.clone();
    script.extend([1, 0]);
    script.extend(words(&[1, 1])); // propose zero again, accept this time
    let mut source = TestStream::from_bytes(script);
    assert_eq!(
        sample_discrete_gaussian_with(RBig::ONE, &mut source)?,
        IBig::ZERO
    );
    assert_eq!(source.position, 44);
    assert!(sample_discrete_gaussian_with(RBig::ONE, &mut TestStream::from_bytes(first)).is_err());
    Ok(())
}

#[test]
fn zero_scales_do_not_request_entropy_and_invalid_scales_error() -> Fallible<()> {
    let mut source = TestStream::from_bytes(Vec::new());
    assert_eq!(
        sample_discrete_laplace_with(RBig::ZERO, &mut source)?,
        IBig::ZERO
    );
    assert_eq!(
        sample_discrete_gaussian_with(RBig::ZERO, &mut source)?,
        IBig::ZERO
    );
    assert_eq!(
        sample_geometric_exp_fast_with(RBig::ZERO, &mut source)?,
        UBig::ZERO
    );
    assert!(sample_discrete_laplace_with(-RBig::ONE, &mut source).is_err());
    assert!(sample_discrete_gaussian_with(-RBig::ONE, &mut source).is_err());
    assert_eq!(source.position, 0);
    Ok(())
}

#[test]
fn buffered_and_direct_discrete_traces_are_identical() -> Fallible<()> {
    fn check<const N: usize>() -> Fallible<()> {
        let mut direct = TestStream::new(17);
        let mut buffered = TestBuffered::<N>::new(17);
        for i in 0..1000 {
            let scale = [
                rbig!(1 / 10),
                RBig::ONE,
                rbig!(10 / 3),
                RBig::from(UBig::ONE << 1074),
            ][i % 4]
                .clone();
            let a = sample_discrete_laplace_with(scale.clone(), &mut direct)?;
            let b = sample_discrete_laplace_with(scale.clone(), &mut buffered)?;
            assert_eq!(a, b);
            assert_eq!(direct.position, buffered.consumed());
            let a = sample_discrete_gaussian_with(scale.clone(), &mut direct)?;
            let b = sample_discrete_gaussian_with(scale, &mut buffered)?;
            assert_eq!(a, b);
            assert_eq!(direct.position, buffered.consumed());
        }
        Ok(())
    }
    check::<7>()?;
    check::<256>()?;
    Ok(())
}

#[test]
fn each_entropy_failure_in_complete_traces_is_returned_immediately() -> Fallible<()> {
    struct FailAt {
        stream: TestStream,
        calls: usize,
        fail: usize,
    }
    impl EntropySource for FailAt {
        fn fill(&mut self, out: &mut [u8]) -> Fallible<()> {
            self.calls += 1;
            if self.calls == self.fail {
                out.fill(0xee);
                return fallible!(FailedFunction, "synthetic partial entropy failure");
            }
            self.stream.fill(out)
        }
    }
    for gaussian in [false, true] {
        for seed in 0..3 {
            let sample = |source: &mut FailAt| {
                if gaussian {
                    sample_discrete_gaussian_with(rbig!(3 / 2), source)
                } else {
                    sample_discrete_laplace_with(rbig!(3 / 2), source)
                }
            };
            let mut baseline = FailAt {
                stream: TestStream::new(seed),
                calls: 0,
                fail: usize::MAX,
            };
            sample(&mut baseline)?;
            for fail in 1..=baseline.calls {
                let mut source = FailAt {
                    stream: TestStream::new(seed),
                    calls: 0,
                    fail,
                };
                assert!(sample(&mut source).is_err());
                assert_eq!(source.calls, fail, "error must not become rejection/retry");
            }
        }
    }
    Ok(())
}

#[test]
fn gaussian_fractional_scales_match_discrete_moments_and_zero_mass() -> Fallible<()> {
    let n = 30_000;
    for scale in [rbig!(1 / 10), rbig!(1 / 3), rbig!(3 / 2)] {
        let sigma = scale.to_f64().value();
        let weights: Vec<_> = (-50i32..=50)
            .map(|k| (k as f64, (-(k * k) as f64 / (2.0 * sigma * sigma)).exp()))
            .collect();
        let z: f64 = weights.iter().map(|(_, w)| w).sum();
        let expected_zero = 1.0 / z;
        let m2: f64 = weights.iter().map(|(k, w)| k * k * w).sum::<f64>() / z;
        let m4: f64 = weights.iter().map(|(k, w)| k.powi(4) * w).sum::<f64>() / z;
        let mut zeros = 0;
        let mut second = 0.0;
        for _ in 0..n {
            let x = sample_discrete_gaussian(scale.clone())?;
            zeros += usize::from(x == IBig::ZERO);
            let x = f64::try_from(x).unwrap();
            second += x * x;
        }
        let zero_hat = zeros as f64 / n as f64;
        let second = second / n as f64;
        let zero_slack =
            8.0 * (expected_zero * (1.0 - expected_zero) / n as f64).sqrt() + 3.0 / n as f64;
        let moment_slack = 8.0 * ((m4 - m2 * m2).max(0.0) / n as f64).sqrt() + 3.0 / n as f64;
        assert!(
            (zero_hat - expected_zero).abs() < zero_slack,
            "scale={sigma}, p0={zero_hat}, expected={expected_zero}"
        );
        assert!(
            (second - m2).abs() < moment_slack,
            "scale={sigma}, second={second}, expected={m2}"
        );
    }
    Ok(())
}
