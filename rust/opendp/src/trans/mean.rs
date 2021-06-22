use crate::core::{DatasetMetric, Transformation, Function, StabilityRelation};
use std::ops::{Sub, Div};
use std::iter::Sum;
use crate::traits::DistanceConstant;
use crate::error::Fallible;
use crate::dom::{VectorDomain, IntervalDomain, AllDomain, SizedDomain};
use std::collections::Bound;
use crate::dist::{HammingDistance, SymmetricDistance, AbsoluteDistance};
use num::{NumCast, Float};

pub trait BoundedMeanConstant<T> {
    fn get_stability(lower: T, upper: T, n: usize) -> Fallible<T>;
}

impl<T> BoundedMeanConstant<T> for HammingDistance
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, n: usize) -> Fallible<T> {
        let n = T::from(n).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower) / n)
    }
}

// postprocessing the sum
impl<T> BoundedMeanConstant<T> for SymmetricDistance
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, n: usize) -> Fallible<T> {
        Ok((upper - lower) / num_cast!(n; T)? / num_cast!(2; T)?)
    }
}

pub fn make_bounded_mean<MI, T>(
    lower: T, upper: T, n: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, AbsoluteDistance<T>>>
    where MI: BoundedMeanConstant<T> + DatasetMetric,
          T: DistanceConstant + Sub<Output=T> + Float,
          for <'a> T: Sum<&'a T> {
    let _n = num_cast!(n; T)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))?),
                         n),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().sum::<T>() / _n),
        MI::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(MI::get_stability(lower, upper, n)?)))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = make_bounded_mean::<HammingDistance, f64>(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = make_bounded_mean::<SymmetricDistance, f64>(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &1.).unwrap_test())
    }
}