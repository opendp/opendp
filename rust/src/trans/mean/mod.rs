#[cfg(feature = "ffi")]
mod ffi;

use num::Float;

use crate::core::{Metric, Transformation};
use crate::dist::AbsoluteDistance;
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::ExactIntCast;

use super::{
    make_lipschitz_float_mul, make_sized_bounded_sum, LipschitzMulFloatDomain, LipschitzMulFloatMetric,
    MakeSizedBoundedSum,
};

pub fn make_sized_bounded_mean<MI, T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        MI,
        AbsoluteDistance<T>,
    >,
>
where
    MI: 'static + Metric,
    T: 'static + MakeSizedBoundedSum<MI> + ExactIntCast<usize> + Float,
    AllDomain<T>: LipschitzMulFloatDomain<Atom = T>,
    AbsoluteDistance<T>: LipschitzMulFloatMetric<Distance = T>,
{
    if size == 0 {
        return fallible!(MakeTransformation, "dataset size must be positive");
    }
    let size_ = T::exact_int_cast(size)?;
    let sum_bounds = (size_ * bounds.0, size_ * bounds.1);
    make_sized_bounded_sum::<MI, T>(size, bounds)? >> make_lipschitz_float_mul(size_.recip(), sum_bounds)?
}

#[cfg(test)]
mod tests {
    use crate::dist::SymmetricDistance;
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_sized_bounded_mean;

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
