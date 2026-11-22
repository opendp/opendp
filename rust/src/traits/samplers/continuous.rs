use std::{cmp::Ordering, marker::PhantomData};

use dashu::{integer::UBig, rational::RBig, rbig};

use crate::{
    error::Fallible,
    traits::{
        Float,
        samplers::{
            sample_geometric_exp_fast, sample_rounded_gaussian_rational,
            sample_standard_bernoulli,
        },
    },
};

pub trait RoundedContinuousVectorSampler<T: Float> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>>;
}

trait LazyRational {
    fn lower(&self) -> RBig;
    fn upper(&self) -> RBig;
    fn refine(&mut self) -> Fallible<()>;

    fn refine_until_disjoint_from<L: LazyRational>(&mut self, other: &mut L) -> Fallible<Ordering> {
        loop {
            let a_lo = self.lower();
            let a_hi = self.upper();
            let b_lo = other.lower();
            let b_hi = other.upper();

            if a_hi < b_lo {
                return Ok(Ordering::Less);
            }
            if b_hi < a_lo {
                return Ok(Ordering::Greater);
            }

            self.refine()?;
            other.refine()?;
        }
    }

    fn greater_than<L: LazyRational>(&mut self, other: &mut L) -> Fallible<bool> {
        Ok(self.refine_until_disjoint_from(other)? == Ordering::Greater)
    }
}

#[derive(Clone, Debug)]
struct LazyDyadicUniform01 {
    prefix: UBig,
    precision: usize,
}

impl LazyDyadicUniform01 {
    fn sample() -> Fallible<Self> {
        Ok(Self {
            prefix: UBig::ZERO,
            precision: 0,
        })
    }

    fn den(&self) -> UBig {
        UBig::ONE << self.precision
    }
}

impl LazyRational for LazyDyadicUniform01 {
    fn lower(&self) -> RBig {
        if self.precision == 0 {
            RBig::ZERO
        } else {
            RBig::from_parts(self.prefix.as_ibig().clone(), self.den())
        }
    }

    fn upper(&self) -> RBig {
        if self.precision == 0 {
            RBig::ONE
        } else {
            RBig::from_parts(
                (self.prefix.clone() + UBig::ONE).as_ibig().clone(),
                self.den(),
            )
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        let bit = if sample_standard_bernoulli()? {
            UBig::ONE
        } else {
            UBig::ZERO
        };
        self.prefix = (&self.prefix << 1usize) + bit;
        self.precision += 1;
        Ok(())
    }
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

    fn refine(&mut self) -> Fallible<()> {
        self.inner.refine()
    }
}


fn midpoint(lhs: RBig, rhs: RBig) -> RBig {
    (lhs + rhs) / RBig::from(2)
}

fn half_gap(lhs: RBig, rhs: RBig) -> RBig {
    (rhs - lhs) / RBig::from(2)
}

type RoundingCell = (Option<RBig>, Option<RBig>);

fn rounding_cell<T: Float>(y: T) -> Fallible<RoundingCell> {
    if y == T::infinity() {
        let max = T::max_value();
        let max_r = max.into_rational()?;
        let prev_r = max.next_down_().into_rational()?;
        return Ok((Some(max_r.clone() + half_gap(prev_r, max_r)), None));
    }

    if y == T::neg_infinity() {
        let min = T::min_value();
        let min_r = min.into_rational()?;
        let next_r = min.next_up_().into_rational()?;
        return Ok((None, Some(min_r.clone() - half_gap(min_r, next_r))));
    }

    if y == T::zero() {
        let lower = midpoint(y.next_down_().into_rational()?, RBig::ZERO);
        let upper = midpoint(RBig::ZERO, y.next_up_().into_rational()?);
        return Ok((Some(lower), Some(upper)));
    }

    let y_r = y.into_rational()?;
    let lower = midpoint(y.next_down_().into_rational()?, y_r.clone());
    let upper = midpoint(y_r.clone(), y.next_up_().into_rational()?);
    Ok((Some(lower), Some(upper)))
}

fn interval_inside_cell(interval: &(RBig, RBig), cell: &RoundingCell) -> bool {
    let lower_in = cell.0.as_ref().is_none_or(|lo| &interval.0 >= lo);
    let upper_in = cell.1.as_ref().is_none_or(|hi| &interval.1 <= hi);
    lower_in && upper_in
}

fn finalize_lazy_rational_to_float<T: Float, L: LazyRational>(mut x: L) -> Fallible<T> {
    loop {
        let interval = (x.lower(), x.upper());
        let mid = midpoint(interval.0.clone(), interval.1.clone());
        let candidate = T::from_rational(mid);
        let cell = rounding_cell(candidate)?;
        if interval_inside_cell(&interval, &cell) {
            return Ok(candidate);
        }
        x.refine()?;
    }
}

#[derive(Clone, Debug)]
struct LazyStdLaplace {
    negative: bool,
    e: LazyStdExponential,
}

impl LazyRational for LazyStdLaplace {
    fn lower(&self) -> RBig {
        if self.negative {
            -self.e.upper()
        } else {
            self.e.lower()
        }
    }

    fn upper(&self) -> RBig {
        if self.negative {
            -self.e.lower()
        } else {
            self.e.upper()
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        self.e.refine()
    }
}

#[derive(Clone, Debug)]
struct LazyStdExponential {
    k: UBig,
    x: LazyDyadicUniform01,
}

impl LazyRational for LazyStdExponential {
    fn lower(&self) -> RBig {
        RBig::from(self.k.clone()) + self.x.lower()
    }

    fn upper(&self) -> RBig {
        RBig::from(self.k.clone()) + self.x.upper()
    }

    fn refine(&mut self) -> Fallible<()> {
        self.x.refine()
    }
}

fn sample_lazy_standard_laplace() -> Fallible<LazyStdLaplace> {
    Ok(LazyStdLaplace {
        negative: sample_standard_bernoulli()?,
        e: sample_lazy_standard_exponential()?,
    })
}

fn sample_lazy_standard_exponential() -> Fallible<LazyStdExponential> {
    loop {
        let k = sample_geometric_exp_fast(rbig!(1))?;
        let mut x = LazyDyadicUniform01::sample()?;
        if sample_bernoulli_exp_unit_lazy(&mut x)? {
            return Ok(LazyStdExponential { k, x });
        }
    }
}

fn sample_bernoulli_exp_unit_lazy(x: &mut LazyDyadicUniform01) -> Fallible<bool> {
    let mut y = LazyDyadicUniform01::sample()?;

    if !x.greater_than(&mut y)? {
        return Ok(true);
    }

    let mut accept_on_failure = false;
    loop {
        let mut u = LazyDyadicUniform01::sample()?;

        if !y.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
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
        input
            .iter()
            .copied()
            .map(|x| {
                let x = x.clamp(T::MIN_FINITE, T::MAX_FINITE);
                finalize_lazy_rational_to_float::<T, _>(LazyAffine::new(
                    self.scale.clone(),
                    x.into_rational()?,
                    sample_lazy_standard_laplace()?,
                ))
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
        input
            .iter()
            .copied()
            .map(|x| {
                sample_rounded_gaussian_rational(
                    x.clamp(T::MIN_FINITE, T::MAX_FINITE),
                    self.scale.clone(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
