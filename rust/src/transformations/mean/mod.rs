#[cfg(feature = "ffi")]
mod ffi;

use num::Float;
use opendp_derive::bootstrap;

use crate::core::{Metric, Transformation};
use crate::domains::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::AbsoluteDistance;
use crate::traits::{ExactIntCast, InfMul};

use super::{
    make_lipschitz_float_mul, make_sized_bounded_sum, LipschitzMulFloatDomain,
    LipschitzMulFloatMetric, MakeSizedBoundedSum,
};

#[bootstrap(
    features("contrib"),
    generics(MI(default = "SymmetricDistance"), T(example = "$get_first(bounds)"))
)]
/// Make a Transformation that computes the mean of bounded data.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size.
/// Use `make_clamp` to bound data and `make_resize` to establish dataset size.
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `MI` - Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`
/// * `T` - Atomic Input Type and Output Type.
pub fn make_sized_bounded_mean<MI, T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        MI,
        AbsoluteDistance<T>,
    >,
>
where
    MI: 'static + Metric,
    T: 'static + MakeSizedBoundedSum<MI> + ExactIntCast<usize> + Float + InfMul,
    AllDomain<T>: LipschitzMulFloatDomain<Atom = T>,
    AbsoluteDistance<T>: LipschitzMulFloatMetric<Distance = T>,
{
    if size == 0 {
        return fallible!(MakeTransformation, "dataset size must be positive");
    }
    let size_ = T::exact_int_cast(size)?;
    // don't loosen the bounds by the relaxation term because any value greater than nU is pure error
    let sum_bounds = (size_.neg_inf_mul(&bounds.0)?, size_.inf_mul(&bounds.1)?);
    make_sized_bounded_sum::<MI, T>(size, bounds)?
        >> make_lipschitz_float_mul(size_.recip(), sum_bounds)?
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;
    use crate::metrics::SymmetricDistance;
    use crate::transformations::mean::make_sized_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation =
            make_sized_bounded_mean::<SymmetricDistance, f64>(5, (0., 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.check(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation =
            make_sized_bounded_mean::<SymmetricDistance, _>(5, (0f64, 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.check(&1, &1.).unwrap_test())
    }
}
