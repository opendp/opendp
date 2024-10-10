use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance, SymmetricDistance},
    traits::{
        samplers::Shuffle, AlertingAbs, ExactIntCast, InfAdd, InfCast, InfMul, InfSub, ProductOrd,
    },
};

use dashu::integer::IBig;
use num::{One, Zero};
use opendp_derive::bootstrap;

use super::{Float, Pairwise, Sequential, SumRelaxation};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)")),
    generics(S(default = "Pairwise<T>", generics = "T")),
    returns(c_type = "FfiResult<AnyTransformation *>"),
    derived_types(T = "$get_atom_or_infer(S, get_first(bounds))")
)]
/// Make a Transformation that computes the sum of bounded data with known dataset size.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
/// Use `make_clamp` to bound data and `make_resize` to establish dataset size.
///
/// | S (summation algorithm) | input type     |
/// | ----------------------- | -------------- |
/// | `Sequential<S::Item>`   | `Vec<S::Item>` |
/// | `Pairwise<S::Item>`     | `Vec<S::Item>` |
///
/// `S::Item` is the type of all of the following:
/// each bound, each element in the input data, the output data, and the output sensitivity.
///
/// For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
/// set `S` to `Pairwise<f32>`.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `size_limit` - Upper bound on number of records to keep in the input data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `S` - Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
pub fn make_bounded_float_checked_sum<S>(
    size_limit: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<S::Item>>,
        AtomDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float,
{
    if S::can_float_sum_overflow(size_limit, bounds)? {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function. You could resolve this by choosing tighter clipping bounds."
        );
    }

    let (lower, upper) = bounds;
    let ideal_sensitivity = upper
        .inf_sub(&lower)?
        .total_max(lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = S::relaxation(size_limit, lower, upper)?;

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?),
        AtomDomain::default(),
        Function::new_fallible(move |arg: &Vec<S::Item>| {
            let mut data = arg.clone();
            if arg.len() > size_limit {
                data.shuffle()?
            }
            Ok(S::unchecked_sum(&data[..size_limit.min(data.len())]))
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * ideal_sens + 2 * error
            //       =  d_in * ideal_sens + relaxation
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)")),
    generics(S(default = "Pairwise<T>", generics = "T")),
    returns(c_type = "FfiResult<AnyTransformation *>"),
    derived_types(T = "$get_atom_or_infer(S, get_first(bounds))")
)]
/// Make a Transformation that computes the sum of bounded floats with known dataset size.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
///
/// | S (summation algorithm) | input type     |
/// | ----------------------- | -------------- |
/// | `Sequential<S::Item>`   | `Vec<S::Item>` |
/// | `Pairwise<S::Item>`     | `Vec<S::Item>` |
///
/// `S::Item` is the type of all of the following:
/// each bound, each element in the input data, the output data, and the output sensitivity.
///
/// For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
/// set `S` to `Pairwise<f32>`.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `S` - Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
pub fn make_sized_bounded_float_checked_sum<S>(
    size: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<S::Item>>,
        AtomDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float,
{
    if S::can_float_sum_overflow(size, bounds)? {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function. You could resolve this by choosing tighter clipping bounds."
        );
    }

    let (lower, upper) = bounds;
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    let relaxation = S::relaxation(size, lower, upper)?;

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size),
        AtomDomain::default(),
        // Under the assumption that the input data is in input domain, then an unchecked sum is safe.
        Function::new(move |arg: &Vec<S::Item>| S::unchecked_sum(arg)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in / 2 * (U - L) + 2 * error
            //       =  d_in / 2 * (U - L) + relaxation
            S::Item::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    )
}

#[doc(hidden)]
pub trait UncheckedSum: SumRelaxation + CanFloatSumOverflow {
    fn unchecked_sum(arg: &[Self::Item]) -> Self::Item;
}
impl<T: Float> UncheckedSum for Sequential<T> {
    fn unchecked_sum(arg: &[T]) -> T {
        arg.iter().cloned().sum()
    }
}

impl<T: Float> UncheckedSum for Pairwise<T> {
    fn unchecked_sum(arg: &[T]) -> T {
        match arg.len() {
            0 => T::zero(),
            1 => arg[0],
            n => {
                let m = n / 2;
                Self::unchecked_sum(&arg[..m]) + Self::unchecked_sum(&arg[m..])
            }
        }
    }
}

#[doc(hidden)]
pub trait CanFloatSumOverflow: SumRelaxation {
    fn can_float_sum_overflow(size: usize, bounds: (Self::Item, Self::Item)) -> Fallible<bool>;
}

impl<T: Float> CanFloatSumOverflow for Sequential<T> {
    fn can_float_sum_overflow(size: usize, (lower, upper): (T, T)) -> Fallible<bool> {
        let _2 = T::one() + T::one();
        let size_ = T::inf_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;

        // CHECK 1
        // If bound magnitude < ulp(T::MAX) / 2,
        //     then each addition to the accumulator will be unable to reach inf,
        //     because summations that reach the last band of floats will underflow/saturate.

        // ulp(T::MAX) / 2 = 2^(max_exponent - num_mantissa_bits) / 2
        // max_unbiased_exponent is always the same as the exponent bias
        let mag_limit = _2.powf(T::exact_int_cast(
            T::EXPONENT_BIAS - T::MANTISSA_BITS - T::Bits::one(),
        )?);
        if mag < mag_limit {
            // we can't overflow, because high magnitude additions will underflow
            return Ok(false);
        }

        // CHECK 2
        // The round up will never be by more than the next magnitude of 2
        // 2^ceil(log2(max(|L|, U))) * N is finite
        Ok(round_up_to_nearest_power_of_two(mag)?
            .inf_mul(&size_)
            .is_err())
    }
}

impl<T: Float> CanFloatSumOverflow for Pairwise<T> {
    fn can_float_sum_overflow(size: usize, (lower, upper): (T, T)) -> Fallible<bool> {
        let _2 = T::one() + T::one();
        let size_ = T::inf_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;

        // CHECK 1
        // If mag * N / 2 < ulp(T::MAX) / 2,
        //     then the final branch of the pairwise sum will underflow
        //     if the sum reaches the coarsest band of floats.
        // Therefore we want mag < ulp(T::MAX) / N

        // mag_limit = ulp(T::MAX) / N = 2^(max_unbiased_exponent - num_mantissa_bits) / N
        // max_unbiased_exponent is always the same as the exponent bias
        let max_ulp = _2.powf(T::exact_int_cast(T::EXPONENT_BIAS - T::MANTISSA_BITS)?);
        let mag_limit = max_ulp.neg_inf_div(&size_)?;
        if mag < mag_limit {
            // we can't overflow, because the largest possible addition will underflow
            return Ok(false);
        }

        // CHECK 2
        // The round up will never be by more than the next magnitude of 2
        // 2^ceil(log2(max(|L|, U))) * N is finite
        Ok(round_up_to_nearest_power_of_two(mag)?
            .inf_mul(&size_)
            .is_err())
    }
}

fn round_up_to_nearest_power_of_two<T>(x: T) -> Fallible<T>
where
    T: ExactIntCast<T::Bits> + Float,
{
    if x.is_sign_negative() {
        return fallible!(
            FailedFunction,
            "get_smallest_greater_or_equal_power_of_two must have a positive argument"
        );
    }

    let exponent_bias: IBig = T::EXPONENT_BIAS.into();
    let exponent: IBig = x.raw_exponent().into();
    // this subtraction is on small whole integers, so is exact
    let exponent_unbiased = exponent - exponent_bias;

    let pow = exponent_unbiased
        + if x.mantissa().is_zero() {
            IBig::ZERO
        } else {
            IBig::ONE
        };

    let _2 = T::one() + T::one();
    _2.inf_powi(pow)
}

#[cfg(test)]
mod test;
