#[cfg(feature = "ffi")]
mod ffi;

use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

use num::One;

use crate::core::Transformation;
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{
    AlertingAbs, AlertingSub, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd,
    InfCast, InfDiv, InfMul, InfPow, InfSub, SaturatingMul,
};

use super::{
    make_lipschitz_mul_float, make_sized_bounded_sum_of_product_deviations,
    make_sized_bounded_sum_of_squared_deviations, Float,
};

pub fn make_sized_bounded_variance<T>(
    size: usize,
    bounds: (T, T),
    ddof: usize,
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
        + AlertingAbs
        + for<'a> Mul<&'a T, Output = T>
        + InfCast<T>
        + SaturatingMul,
    for<'a> &'a T: Sub<Output = T> + Add<&'a T, Output = T>,
{
    let dof = size.alerting_sub(&ddof)?;
    let constant = T::exact_int_cast(dof)?.recip();
    let _2 = T::one() + T::one();
    let size_ = T::exact_int_cast(size)?;
    let upper_var_bound = (bounds.1 - bounds.0).powi(2) / (_2 * size_);
    make_sized_bounded_sum_of_squared_deviations(size, bounds)?
        >> make_lipschitz_mul_float(constant, (T::zero(), upper_var_bound))?
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<BoundedDomain<(T, T)>>>;

pub fn make_sized_bounded_covariance<T>(
    size: usize,
    bounds_0: (T, T),
    bounds_1: (T, T),
    ddof: usize,
) -> Fallible<
    Transformation<CovarianceDomain<T>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>,
>
where
    T: 'static + Float,
    for<'a> T: Div<&'a T, Output = T> + Add<&'a T, Output = T> + Mul<&'a T, Output = T>,
    for<'a> &'a T: Sub<Output = T>,
{
    let dof = size.alerting_sub(&ddof)?;
    let constant = T::exact_int_cast(dof)?.recip();
    let _2 = T::one() + T::one();
    let size_ = T::exact_int_cast(size)?;
    let upper_cov_bound = (bounds_0.1 - bounds_0.0) * (bounds_1.1 - bounds_1.0) / (_2 * size_);
    make_sized_bounded_sum_of_product_deviations(size, bounds_0, bounds_1)?
        >> make_lipschitz_mul_float(constant, (T::zero(), upper_cov_bound))?
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_sized_bounded_variance(5, (0., 10.), 1).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_variance(5, (0., 10.), 0).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))
            .unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =
            make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 1).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop =
            make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 0).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))
            .unwrap_test());
    }
}
