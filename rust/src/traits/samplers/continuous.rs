use std::marker::PhantomData;

use dashu::{integer::UBig, rational::RBig};

use crate::{
    error::Fallible,
    traits::{
        Float,
        samplers::{
            EntropySource, PartialUniform01, SamplerEntropy, Uniform01RV,
            sample_rounded_gaussian_rational,
        },
    },
};

pub trait RoundedContinuousVectorSampler<T: Float> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>>;
}

trait LazyRational {
    fn lower(&self) -> RBig;
    fn upper(&self) -> RBig;
    fn refine(&mut self, entropy: &mut impl EntropySource) -> Fallible<()>;
}

#[derive(Clone, Debug)]
struct LazyAffine<L> {
    scale: RBig,
    offset: RBig,
    inner: L,
}

impl<L> LazyAffine<L> {
    fn new(scale: RBig, offset: RBig, inner: L) -> Self {
        Self {
            scale,
            offset,
            inner,
        }
    }
}

impl<L: LazyRational> LazyRational for LazyAffine<L> {
    fn lower(&self) -> RBig {
        if self.scale >= RBig::ZERO {
            &self.offset + &self.scale * self.inner.lower()
        } else {
            &self.offset + &self.scale * self.inner.upper()
        }
    }

    fn upper(&self) -> RBig {
        if self.scale >= RBig::ZERO {
            &self.offset + &self.scale * self.inner.upper()
        } else {
            &self.offset + &self.scale * self.inner.lower()
        }
    }

    fn refine(&mut self, entropy: &mut impl EntropySource) -> Fallible<()> {
        self.inner.refine(entropy)
    }
}

// Share the boundary handling for subnormals, finite extrema and infinities.
use super::rounded_gaussian::{interval_inside_cell, rounding_cell};

fn finalize_lazy_rational_to_float<T: Float, L: LazyRational>(
    mut x: L,
    entropy: &mut impl EntropySource,
) -> Fallible<T> {
    loop {
        let interval = (x.lower(), x.upper());
        // Any endpoint is a valid candidate guess: containment below is the
        // certificate. Avoid constructing an additional rational midpoint.
        let candidate = T::from_rational(interval.0.clone());
        let cell = rounding_cell(candidate)?;
        if interval_inside_cell(&interval, &cell) {
            return Ok(candidate);
        }
        x.refine(entropy)?;
    }
}

impl LazyRational for PartialUniform01 {
    fn lower(&self) -> RBig {
        PartialUniform01::lower(self).expect("Uniform01 inverse CDF is identity")
    }

    fn upper(&self) -> RBig {
        PartialUniform01::upper(self).expect("Uniform01 inverse CDF is identity")
    }

    fn refine(&mut self, entropy: &mut impl EntropySource) -> Fallible<()> {
        self.refine_with(entropy)
    }
}

struct LazyStdExponential {
    k: UBig,
    x: PartialUniform01,
}

fn sample_lazy_standard_exponential(
    entropy: &mut impl EntropySource,
) -> Fallible<LazyStdExponential> {
    // Each attempt accepts with probability integral_0^1 exp(-x) dx = 1-e^-1.
    // Thus P[k=m, X in dx] = e^-m exp(-x) dx: the number of rejected
    // attempts supplies the integer part of Exp(1), without a separate draw.
    let mut k = UBig::ZERO;
    loop {
        let mut x = PartialUniform01::new(Uniform01RV);
        if sample_bernoulli_exp_unit_lazy(&mut x, entropy)? {
            return Ok(LazyStdExponential { k, x });
        }
        k += 1u8;
    }
}

fn sample_bernoulli_exp_unit_lazy(
    x: &mut PartialUniform01,
    entropy: &mut impl EntropySource,
) -> Fallible<bool> {
    let mut y = PartialUniform01::new(Uniform01RV);

    if !y.less_than_scaled_uniform(x, 0, entropy)? {
        return Ok(true);
    }

    let mut accept_on_failure = false;
    loop {
        let mut u = PartialUniform01::new(Uniform01RV);

        if !u.less_than_scaled_uniform(&mut y, 0, entropy)? {
            return Ok(accept_on_failure);
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

fn sample_rounded_laplace_rational<T: Float>(
    mu: T,
    scale: &RBig,
    entropy: &mut impl EntropySource,
) -> Fallible<T> {
    let mu = mu.into_rational()?;
    let negative = entropy.coin()?;
    let LazyStdExponential { k, x } = sample_lazy_standard_exponential(entropy)?;
    let scale = if negative { -scale } else { scale.clone() };
    // Fold the integer part into the offset once, rather than reconstructing
    // and scaling k+x separately at both bounds.
    let offset = mu + &scale * RBig::from(k);
    finalize_lazy_rational_to_float::<T, _>(LazyAffine::new(scale, offset, x), entropy)
}

#[derive(Clone, Debug)]
pub struct IidContinuousLaplace<T: Float> {
    scale: RBig,
    _marker: PhantomData<T>,
}

impl<T: Float> IidContinuousLaplace<T> {
    pub fn new(scale: RBig) -> Fallible<Self> {
        if scale < RBig::ZERO {
            return fallible!(FailedFunction, "scale must be nonnegative");
        }
        Ok(Self {
            scale,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for IidContinuousLaplace<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        if self.scale.is_zero() {
            return Ok(input
                .iter()
                .map(|x| x.clamp(T::MIN_FINITE, T::MAX_FINITE))
                .collect());
        }
        let mut entropy = SamplerEntropy::new();
        input
            .iter()
            .copied()
            .map(|x| {
                sample_rounded_laplace_rational(
                    x.clamp(T::MIN_FINITE, T::MAX_FINITE),
                    &self.scale,
                    &mut entropy,
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct IidContinuousGaussian<T: Float> {
    scale: RBig,
    _marker: PhantomData<T>,
}

impl<T: Float> IidContinuousGaussian<T> {
    pub fn new(scale: RBig) -> Fallible<Self> {
        if scale < RBig::ZERO {
            return fallible!(FailedFunction, "scale must be nonnegative");
        }
        Ok(Self {
            scale,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for IidContinuousGaussian<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        let mut entropy = SamplerEntropy::new();
        input
            .iter()
            .copied()
            .map(|x| {
                sample_rounded_gaussian_rational(
                    x.clamp(T::MIN_FINITE, T::MAX_FINITE),
                    self.scale.clone(),
                    &mut entropy,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::CastInternalRational;

    fn finalize_lazy_rational_to_float<T: Float, L: LazyRational>(x: L) -> Fallible<T> {
        super::finalize_lazy_rational_to_float(x, &mut SamplerEntropy::new())
    }

    // A certificate that must finalize without asking for more entropy.
    struct FixedInterval(RBig, RBig);

    impl LazyRational for FixedInterval {
        fn lower(&self) -> RBig {
            self.0.clone()
        }
        fn upper(&self) -> RBig {
            self.1.clone()
        }
        fn refine(&mut self, _: &mut impl EntropySource) -> Fallible<()> {
            panic!("the interval should already fit in one rounding cell")
        }
    }

    #[test]
    fn endpoint_candidate_on_a_tie_refines_instead_of_misrounding() -> Fallible<()> {
        struct BoundaryInterval(RBig, RBig);
        impl LazyRational for BoundaryInterval {
            fn lower(&self) -> RBig {
                self.0.clone()
            }
            fn upper(&self) -> RBig {
                self.1.clone()
            }
            fn refine(&mut self, _: &mut impl EntropySource) -> Fallible<()> {
                self.0 = (&self.0 + &self.1) / RBig::from(2);
                Ok(())
            }
        }
        // At the initial lower endpoint, ties-to-even selects 1.0, but all
        // interior real values round to its upper neighbor. Containment must
        // reject that candidate and refine rather than treating it as final.
        let boundary = RBig::ONE + RBig::from_parts(1.into(), UBig::ONE << 53);
        let hi = &boundary + RBig::from_parts(1.into(), UBig::ONE << 55);
        assert_eq!(
            finalize_lazy_rational_to_float::<f64, _>(BoundaryInterval(boundary, hi))?,
            f64::from_bits(1f64.to_bits() + 1)
        );
        Ok(())
    }

    #[test]
    fn folded_affine_matches_unfolded_bounds() -> Fallible<()> {
        let lo = RBig::from_parts(1.into(), 8u8.into());
        let hi = RBig::from_parts(3.into(), 8u8.into());
        for mu in [0.0f64, 1.0, f64::from_bits(1), f64::MAX] {
            for scale in [0.1f64, 1.0, f64::from_bits(1), f64::MAX] {
                for negative in [false, true] {
                    for k in [0u8, 1, 3] {
                        let mu = mu.into_rational()?;
                        let signed_scale =
                            scale.into_rational()? * RBig::from(if negative { -1 } else { 1 });
                        let k = RBig::from(k);
                        let affine = LazyAffine::new(
                            signed_scale.clone(),
                            &mu + &signed_scale * &k,
                            FixedInterval(lo.clone(), hi.clone()),
                        );
                        let a = &mu + &signed_scale * (&k + &lo);
                        let b = &mu + &signed_scale * (&k + &hi);
                        assert_eq!(affine.lower(), a.clone().min(b.clone()));
                        assert_eq!(affine.upper(), a.max(b));
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn exact_rounding_at_extremes_and_subnormals() -> Fallible<()> {
        for y in [
            f64::MIN,
            -1.0,
            -f64::from_bits(1),
            0.0,
            f64::from_bits(1),
            1.0,
            f64::MAX,
        ] {
            let center = y.into_rational()?;
            assert_eq!(
                finalize_lazy_rational_to_float::<f64, _>(FixedInterval(center.clone(), center))?,
                y
            );
        }
        for y in [
            f32::MIN,
            -1.0,
            -f32::from_bits(1),
            0.0,
            f32::from_bits(1),
            1.0,
            f32::MAX,
        ] {
            let center = y.into_rational()?;
            assert_eq!(
                finalize_lazy_rational_to_float::<f32, _>(FixedInterval(center.clone(), center))?,
                y
            );
        }
        // Exact real values safely beyond the overflow threshold round to infinity.
        let large = RBig::from(UBig::ONE << 1024);
        assert_eq!(
            finalize_lazy_rational_to_float::<f64, _>(FixedInterval(
                large.clone(),
                &large + RBig::ONE
            ))?,
            f64::INFINITY
        );
        assert_eq!(
            finalize_lazy_rational_to_float::<f64, _>(FixedInterval(-&large - RBig::ONE, -large))?,
            f64::NEG_INFINITY
        );
        Ok(())
    }

    #[test]
    fn affine_rounding_handles_cancellation_and_tiny_scales() -> Fallible<()> {
        // Exact rational addition must preserve a tiny residual after cancellation.
        let tiny = f64::from_bits(1).into_rational()?;
        let x = LazyAffine::new(
            RBig::ONE,
            -RBig::ONE,
            FixedInterval(RBig::ONE + &tiny, RBig::ONE + &tiny),
        );
        assert_eq!(
            finalize_lazy_rational_to_float::<f64, _>(x)?,
            f64::from_bits(1)
        );
        let x = LazyAffine::new(
            tiny,
            RBig::ZERO,
            FixedInterval(
                RBig::from_parts(1.into(), 8u8.into()),
                RBig::from_parts(1.into(), 4u8.into()),
            ),
        );
        assert_eq!(finalize_lazy_rational_to_float::<f64, _>(x)?, 0.0);
        Ok(())
    }

    #[test]
    fn clamped_extreme_inputs_do_not_error() -> Fallible<()> {
        for input in [f64::MIN, f64::MAX, f64::NEG_INFINITY, f64::INFINITY] {
            let expected = input.clamp(f64::MIN, f64::MAX);
            assert_eq!(
                IidContinuousLaplace::<f64>::new(RBig::ONE)?.sample_around(&[input])?,
                [expected]
            );
            assert_eq!(
                IidContinuousGaussian::<f64>::new(RBig::ONE)?.sample_around(&[input])?,
                [expected]
            );
        }
        Ok(())
    }

    #[test]
    fn laplace_distribution_smoke() -> Fallible<()> {
        // Loose (>7 standard errors) bounds: this checks sign, rejection-count
        // integer part, fractional part, and scale without fragile tight tests.
        let n = 10_000;
        let samples =
            IidContinuousLaplace::<f64>::new(RBig::from(2))?.sample_around(&vec![1.0; n])?;
        let mean = samples.iter().map(|x| (x - 1.0) / 2.0).sum::<f64>() / n as f64;
        let abs_mean = samples.iter().map(|x| ((x - 1.0) / 2.0).abs()).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.12, "mean={mean}");
        assert!((abs_mean - 1.0).abs() < 0.1, "abs_mean={abs_mean}");
        for t in [0.5, 1.0, 2.0] {
            let tail = samples
                .iter()
                .filter(|x| ((**x - 1.0) / 2.0).abs() > t)
                .count() as f64
                / n as f64;
            assert!((tail - (-t).exp()).abs() < 0.04, "t={t}, tail={tail}");
        }
        Ok(())
    }

    #[test]
    fn buffering_preserves_complete_laplace_traces() -> Fallible<()> {
        use crate::traits::samplers::entropy::test::{TestBuffered, TestStream};
        fn check<const N: usize>() -> Fallible<()> {
            let mut direct = TestStream::new(123);
            let mut buffered = TestBuffered::<N>::new(123);
            for i in 0..1000 {
                let mu = [0.0f64, 1.0, -1e6, f64::MAX][i % 4];
                let scale = [1.0f64, 1e-6, 1e6, f64::from_bits(1)][(i / 4) % 4].into_rational()?;
                let a = sample_rounded_laplace_rational(mu, &scale, &mut direct)?;
                let b = sample_rounded_laplace_rational(mu, &scale, &mut buffered)?;
                assert_eq!(a.to_bits(), b.to_bits());
                assert_eq!(direct.position, buffered.consumed());
            }
            Ok(())
        }
        check::<7>()?; // force unaligned words across refills
        check::<256>()?;
        Ok(())
    }

    #[test]
    fn laplace_entropy_failure_is_not_a_rejection() {
        struct FailedEntropy;
        impl EntropySource for FailedEntropy {
            fn fill(&mut self, _: &mut [u8]) -> Fallible<()> {
                fallible!(FailedFunction, "synthetic entropy failure")
            }
        }
        assert!(sample_rounded_laplace_rational(0.0f64, &RBig::ONE, &mut FailedEntropy).is_err());
    }

    #[test]
    fn zero_scale_returns_input() -> Fallible<()> {
        let input = [-2.5f64, 0.0, 3.25];
        assert_eq!(
            IidContinuousLaplace::<f64>::new(RBig::ZERO)?.sample_around(&input)?,
            input
        );
        assert_eq!(
            IidContinuousGaussian::<f64>::new(RBig::ZERO)?.sample_around(&input)?,
            input
        );
        Ok(())
    }

    #[test]
    fn negative_scale_rejected() {
        assert!(IidContinuousLaplace::<f32>::new(-RBig::ONE).is_err());
        assert!(IidContinuousGaussian::<f32>::new(-RBig::ONE).is_err());
    }

    #[test]
    fn vector_samples_are_finite_for_ordinary_inputs() -> Fallible<()> {
        let input = [0.0f32, 0.5, -1.5];
        assert!(
            IidContinuousLaplace::<f32>::new(RBig::ONE)?
                .sample_around(&input)?
                .iter()
                .all(|x| x.is_finite())
        );
        assert!(
            IidContinuousGaussian::<f32>::new(RBig::ONE)?
                .sample_around(&input)?
                .iter()
                .all(|x| x.is_finite())
        );
        Ok(())
    }
}
