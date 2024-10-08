use super::*;

/// Conduct a Kolmogorov-Smirnov (KS) test.
///
/// Since the critical values are difficult to compute,
/// this function hardcodes the critical value corresponding to a p-value of 1e-6 when 1000 samples are taken.
///
/// Assuming the samples are draws from the distribution specified by the cdf,
/// then the p-value is the false discovery rate,
/// or chance of this test failing even when the data is a sample from the distribution.
pub fn kolmogorov_smirnov(mut samples: [f64; 1000], cdf: impl Fn(f64) -> f64) -> Fallible<()> {
    // first, compute the test statistic. For a one-sample KS test,
    // this is the greatest distance between the empirical CDF of the samples and the expected CDF.
    samples.sort_by(|a, b| a.total_cmp(b));

    let n = samples.len() as f64;
    let statistic = samples
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let empirical_cdf = i as f64 / n;
            let idealized_cdf = cdf(s);
            (empirical_cdf - idealized_cdf).abs()
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();

    // The KS-test is nonparametric,
    // so the critical value only changes in response to the number of samples (hardcoded at 1000),
    // not the distribution.
    //
    // The p-value corresponds to the mass of the tail of the KS distribution beyond the critical value.
    // The mass of the tail is the complement of the cumulative distribution function,
    // which is also called the survival function.
    // The inverse of the survival function `isf` tells us the critical value corresponding to a given mass of the tail.
    //
    // We therefore derive the critical value via the inverse survival function of the two-sided, one-sample KS distribution:
    // ```python
    // from scipy.stats import kstwo
    // CRIT_VALUE = kstwo(n=1000).isf(1e-6)
    // ```
    static CRIT_VALUE: f64 = 0.08494641956324511;
    if statistic > CRIT_VALUE {
        return fallible!(FailedFunction, "Statistic ({statistic}) exceeds critical value ({CRIT_VALUE})! This indicates that the data is not sampled from the same distribution specified by the cdf. There is a 1e-6 probability of this being a false positive.");
    }

    Ok(())
}

pub fn assert_ordered_progression<D: InverseCDF>(
    sampler: &mut PartialSample<D>,
    min_refinements: usize,
) -> (D::Edge, D::Edge)
where
    D::Edge: PartialOrd,
{
    loop {
        sampler.refine().unwrap();
        let Some((l, r)) = sampler.lower().zip(sampler.upper()) else {
            continue;
        };
        assert!(l <= r);

        if sampler.refinements >= min_refinements {
            return (l, r);
        }
    }
}

struct UniformRV;

impl InverseCDF for UniformRV {
    type Edge = RBig;

    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<Self::Edge> {
        Some(uniform)
    }
}

#[test]
fn test_value() -> Fallible<()> {
    let mut psrn = PartialSample::new(UniformRV);
    // sampled value will always be in [0, 1]
    assert!((0f64..1f64).contains(&psrn.value()?));

    Ok(())
}

#[test]
fn test_greater_than() -> Fallible<()> {
    let (mut l, mut r) = (PartialSample::new(UniformRV), PartialSample::new(UniformRV));

    if l.greater_than(&mut r)? {
        assert!(l.value::<f64>()? > r.value::<f64>()?);
    } else {
        assert!(l.value::<f64>()? < r.value::<f64>()?);
    }

    Ok(())
}
