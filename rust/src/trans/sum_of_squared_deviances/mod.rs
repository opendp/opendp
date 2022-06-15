#[cfg(feature = "ffi")]
mod ffi;

use std::iter::Sum;
use std::ops::{Add, Div, Sub};

use num::{Float, One, Zero};

use crate::core::{Function, StabilityMap, Transformation};
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{
    AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfDiv, InfMul,
    InfPow, InfSub,
};

pub fn make_sized_bounded_sum_of_squared_deviances<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance>
        + Float
        + One
        + Sum<T>
        + ExactIntCast<usize>
        + ExactIntCast<T::Bits>
        + InfMul
        + InfSub
        + InfAdd
        + InfDiv
        + CheckNull
        + InfPow
        + FloatBits
        + for<'a> Sum<&'a T>
        + AlertingAbs,
    for<'a> &'a T: Sub<Output = T> + Add<&'a T, Output = T>,
{
    let _size = T::exact_int_cast(size)?;
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let (lower, upper) = bounds.clone();
    let _1 = T::one();
    let _2 = T::exact_int_cast(2)?;

    // DERIVE RELAXATION TERM
    // Let x_bar_approx = x_bar + e, the approximate mean on finite data types
    // Let e = (n^2/2^k) / n, the mean error
    let mean_error = _size.inf_div(&_2.inf_pow(&mantissa_bits)?)?;

    // Let d_i = x_i - x_bar, the deviation
    // Then sum_i (x_i - x_bar_approx)^2
    //      = sum_i (x_i - (x_bar + e))^2
    //      = sum_i (d_i - e)^2
    //      = sum_i (d_i^2 - 2 e d_i + e^2)
    // (1)  = sum_i (d_i^2 + pert_i)  Let pert_i = e^2 - 2 e d_i

    // In the worst case, each deviance may differ by an additional pert_max
    // pert_max = max_i|pert_i|
    //    = max_i|e^2 - 2 e d_i|
    //    = e max_i|e - 2 d_i|
    //    <= e max(|e - 2L|, |e - 2U|)
    //    <= e (e + 2 M) where M = max(|L|, U) is the maximum magnitude of a bound
    let bound_mag_max = lower.alerting_abs()?.total_max(upper)?;
    let pert_max = mean_error.inf_mul(&mean_error.inf_add(&_2.inf_mul(&bound_mag_max)?)?)?;

    // Continuing the analysis of SSD from (1):
    // (1)  <= sum_i (d_i^2 + pert_max)
    //      = sum_i d_i^1 + n pert_max

    // (1)  >= sum_i (d_i^2 - pert_max)
    //      = sum_i d_i^1 - n pert_max

    // Let e_var = n^2 / 2^k, the error from computing the sum of deviances
    let var_error = _size
        .inf_mul(&_size)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?;

    // Now d_sum = sum_i d_i^1 +- (e_var + n pert_max)
    let var_relaxation = var_error.inf_add(&_size.inf_mul(&pert_max)?)?;

    // DERIVE IDEAL SENSITIVITY TERM
    // Let range = U - L
    let range = upper.inf_sub(&lower)?;

    // Let ideal_sensitivity = range^2 * (n - 1) / n
    let ideal_sensitivity = range
        .inf_mul(&range)?
        .inf_mul(&_size.neg_inf_sub(&_1)?)?
        .inf_div(&_size)?;

    // OVERFLOW CHECKS
    // Bound the magnitude of the sum when computing the mean
    lower.inf_mul(&_size)?;
    upper.inf_mul(&_size)?;
    // The squared difference from the mean is bounded above by range^2
    range.inf_mul(&range)?.inf_mul(&_size)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            let mean = arg.iter().sum::<T>() / _size;
            arg.iter().map(|v| (v - &mean).powi(2)).sum::<T>()
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // d_in / 2 * sensitivity + relaxation
        StabilityMap::new_fallible(move |d_in| {
            T::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&var_relaxation)
        }),
    ))
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<BoundedDomain<(T, T)>>>;

pub fn make_sized_bounded_sum_of_product_deviances<T>(
    size: usize,
    bounds_0: (T, T),
    bounds_1: (T, T),
) -> Fallible<
    Transformation<CovarianceDomain<T>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>,
>
where
    T: ExactIntCast<usize>
        + CheckNull
        + DistanceConstant<IntDistance>
        + ExactIntCast<T::Bits>
        + Sum<T>
        + Zero
        + Float
        + InfAdd
        + InfSub
        + InfDiv
        + InfPow
        + FloatBits
        + AlertingAbs,
    for<'a> T: Div<&'a T, Output = T> + Add<&'a T, Output = T>,
    for<'a> &'a T: Sub<Output = T>,
{
    let _size = T::exact_int_cast(size)?;
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _1 = T::exact_int_cast(1)?;
    let _2 = T::exact_int_cast(2)?;

    // DERIVE RELAXATION TERM
    // Let x_bar_approx = x_bar + e, the approximate mean on finite data types
    // Let e = (n^2/2^k) / n, the mean error
    let mean_error = _size.inf_div(&_2.inf_pow(&mantissa_bits)?)?; // same for both means

    // we can reuse the sum of deviations error bound if we take the union of the bounds
    // TODO: this can be tightened
    let lower = bounds_0.0.clone().total_min(bounds_1.0.clone())?;
    let upper = bounds_0.1.clone().total_min(bounds_1.1.clone())?;

    let bound_mag_max = lower.alerting_abs()?.total_max(upper)?;
    let pert_max = mean_error.inf_mul(&mean_error.inf_add(&_2.inf_mul(&bound_mag_max)?)?)?;

    let cov_error = _size
        .inf_mul(&_size)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?;

    // Now d_sum = sum_i d_i^1 +- (e_var + n pert_max)
    let cov_relaxation = cov_error.inf_add(&_size.inf_mul(&pert_max)?)?;

    // DERIVE IDEAL SENSITIVITY TERM
    let range_0 = bounds_0.1.clone().inf_sub(&bounds_0.0)?;
    let range_1 = bounds_1.1.clone().inf_sub(&bounds_1.0)?;

    // Let ideal_sensitivity = range_0 * range_1 * (n - 1) / n
    let ideal_sensitivity = range_0
        .inf_mul(&range_1)?
        .inf_mul(&_size.neg_inf_sub(&_1)?)?
        .inf_div(&_size)?;

    // check for potential overflow
    // Bound the magnitudes of the sums when computing the means
    bounds_0.0.inf_mul(&_size)?;
    bounds_0.1.inf_mul(&_size)?;
    bounds_1.0.inf_mul(&_size)?;
    bounds_1.1.inf_mul(&_size)?;
    // The squared difference from the mean is bounded above by range^2
    range_0.inf_mul(&range_1)?;

    Ok(Transformation::new(
        SizedDomain::new(
            VectorDomain::new(BoundedDomain::new_closed((
                (bounds_0.0.clone(), bounds_1.0.clone()),
                (bounds_0.1.clone(), bounds_1.1.clone()),
            ))?),
            size,
        ),
        AllDomain::new(),
        Function::new(enclose!(_size, move |arg: &Vec<(T, T)>| {
            let (sum_l, sum_r) = arg
                .iter()
                .fold((T::zero(), T::zero()), |(s_l, s_r), (v_l, v_r)| {
                    (s_l + v_l, s_r + v_r)
                });
            let (mean_l, mean_r) = (sum_l / &_size, sum_r / &_size);

            arg.iter()
                .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                .sum::<T>()
        })),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // d_in / 2 * sensitivity + relaxation
        StabilityMap::new_fallible(move |d_in| {
            T::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&cov_relaxation)
        }),
    ))
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_make_bounded_deviations() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample =
            make_sized_bounded_sum_of_squared_deviances(5, (0., 10.)).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 10.;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop =
            make_sized_bounded_sum_of_squared_deviances(5, (0., 10.)).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 10.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))
            .unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =
            make_sized_bounded_sum_of_product_deviances(5, (0., 2.), (10., 12.)).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 10.0;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop =
            make_sized_bounded_sum_of_product_deviances(5, (0., 2.), (10., 12.)).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 10.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))
            .unwrap_test());
    }
}
