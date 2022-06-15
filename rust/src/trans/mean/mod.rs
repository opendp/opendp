#[cfg(feature = "ffi")]
mod ffi;

use num::Float;

use crate::core::Transformation;
use crate::dist::{AbsoluteDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::ExactIntCast;

use super::{
    make_lipschitz_mul, make_sized_bounded_sum, LipschitzMulDomain, LipschitzMulMetric,
    MakeSizedBoundedSum,
};

pub fn make_sized_bounded_mean<T>(
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
    T: 'static + MakeSizedBoundedSum + ExactIntCast<usize> + Float,
    AllDomain<T>: LipschitzMulDomain<Atom = T>,
    AbsoluteDistance<T>: LipschitzMulMetric<Distance = T>,
{
    if size == 0 {
        return fallible!(MakeTransformation, "dataset size must be positive");
    }
    let size_ = T::exact_int_cast(size)?;
    make_sized_bounded_sum(size, bounds)? >> make_lipschitz_mul(size_.recip())?
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_sized_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = make_sized_bounded_mean(5, (0., 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.check(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = make_sized_bounded_mean(5, (0., 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.check(&1, &1.).unwrap_test())
    }
}
