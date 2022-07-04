#[cfg(feature = "ffi")]
mod ffi;

use num::Float as _;

use crate::core::Transformation;
use crate::core::{AbsoluteDistance, SymmetricDistance};
use crate::core::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{AlertingSub, ExactIntCast, Float};

use super::{
    make_lipschitz_mul, make_sized_bounded_sum_of_squared_deviations, LipschitzMulDomain,
    LipschitzMulMetric, Pairwise, UncheckedSum,
};

pub fn make_sized_bounded_variance<S>(
    size: usize,
    bounds: (S::Item, S::Item),
    ddof: usize,
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<S::Item>>>,
        AllDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float,
    AllDomain<S::Item>: LipschitzMulDomain<Atom = S::Item>,
    AbsoluteDistance<S::Item>: LipschitzMulMetric<Distance = S::Item>,
{
    if ddof >= size {
        return fallible!(MakeTransformation, "size - ddof must be greater than zero")
    }

    let ddof = size.alerting_sub(&ddof)?;
    make_sized_bounded_sum_of_squared_deviations::<Pairwise<_>>(size, bounds)?
        >> make_lipschitz_mul(S::Item::exact_int_cast(ddof)?.recip())?
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_sized_bounded_variance::<Pairwise<_>>(5, (0., 10.), 1).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_variance::<Pairwise<_>>(5, (0., 10.), 0).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))
            .unwrap_test());
    }
}
