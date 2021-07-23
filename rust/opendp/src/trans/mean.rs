use crate::core::{Transformation, Function, StabilityRelation};
use std::ops::{Sub};
use std::iter::Sum;
use crate::traits::{DistanceConstant, ExactIntCast, InfCast, CheckedMul};
use crate::error::Fallible;
use crate::dom::{VectorDomain, IntervalDomain, AllDomain, SizedDomain};
use std::collections::Bound;
use crate::dist::{SymmetricDistance, AbsoluteDistance, IntDistance};
use num::{Float};

pub fn make_bounded_mean<T>(
    lower: T, upper: T, n: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + Float + ExactIntCast<usize>, for <'a> T: Sum<&'a T> + CheckedMul,
          IntDistance: InfCast<T> {
    let _n = T::exact_int_cast(n)?;
    let _2 = T::exact_int_cast(2)?;

    if lower.checked_mul(&_n).is_none()
        || upper.checked_mul(&_n).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))?),
                         n),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().sum::<T>() / _n),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant((upper - lower) / _n / _2)))
}



#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = make_bounded_mean(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = make_bounded_mean(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &1.).unwrap_test())
    }
}