use crate::core::{Transformation, Function, StabilityRelation};
use std::ops::{Sub};
use std::iter::Sum;
use crate::traits::{DistanceConstant, ExactIntCast, InfCast, CheckedMul, CheckNull};
use crate::error::Fallible;
use crate::dom::{VectorDomain, BoundedDomain, AllDomain, SizedDomain};
use crate::dist::{SymmetricDistance, AbsoluteDistance, IntDistance};
use num::{Float};

pub fn make_sized_bounded_mean<T>(
    size: usize, bounds: (T, T)
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + Float + ExactIntCast<usize>, for <'a> T: Sum<&'a T> + CheckedMul + CheckNull,
          IntDistance: InfCast<T> {
    let _size = T::exact_int_cast(size)?;
    let _2 = T::exact_int_cast(2)?;
    let (lower, upper) = bounds.clone();

    if lower.checked_mul(&_size).is_none()
        || upper.checked_mul(&_size).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().sum::<T>() / _size),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant((upper - lower) / _size / _2)))
}



#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_sized_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = make_sized_bounded_mean(5, (0., 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = make_sized_bounded_mean(5, (0., 10.)).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &1.).unwrap_test())
    }
}