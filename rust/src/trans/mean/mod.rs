#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Transformation, Function, StabilityRelation};
use std::iter::Sum;
use crate::traits::{DistanceConstant, ExactIntCast, InfCast, CheckNull, InfDiv, InfSub};
use crate::error::Fallible;
use crate::dom::{VectorDomain, BoundedDomain, AllDomain, SizedDomain};
use crate::dist::{SymmetricDistance, AbsoluteDistance, IntDistance};
use num::{Float};

pub fn make_sized_bounded_mean<T>(
    size: usize, bounds: (T, T)
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>> where
    T: DistanceConstant<IntDistance> + ExactIntCast<usize>, for <'a> T: Sum<&'a T>
    + InfSub + CheckNull + Float + InfDiv,
    IntDistance: InfCast<T> {
    let _size = T::exact_int_cast(size)?;
    let _2 = T::exact_int_cast(2)?;
    let (lower, upper) = bounds.clone();

    lower.inf_mul(&_size).or(upper.inf_mul(&_size))
        .map_err(|_| err!(MakeTransformation, "potential for overflow when computing function"))?;

    let c = upper.inf_sub(&lower)?.inf_div(&_size)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().sum::<T>() / _size),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2)
                .and_then(|d_in| d_in.inf_mul(&c)))
    ))
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