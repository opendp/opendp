use std::marker::PhantomData;
use crate::core::{DatasetMetric, SensitivityMetric, Transformation, Function, Metric, StabilityRelation};
use std::ops::{Sub, Mul, Div};
use std::iter::Sum;
use crate::traits::DistanceCast;
use crate::error::Fallible;
use crate::dom::{VectorDomain, IntervalDomain, AllDomain, SizedDomain};
use std::collections::Bound;
use crate::trans::MakeTransformation3;
use crate::dist::{HammingDistance, SymmetricDistance};
use num::{NumCast, Float};

pub struct BoundedMean<MI, MO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>
}


pub trait BoundedMeanConstant<MI: Metric, MO: Metric> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric<Distance=T>, T> BoundedMeanConstant<HammingDistance, MO> for BoundedMean<HammingDistance, MO>
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, length: usize) -> Fallible<T> {
        let length = T::from(length).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower) / length)
    }
}

// postprocessing the sum
impl<MO: Metric<Distance=T>, T> BoundedMeanConstant<SymmetricDistance, MO> for BoundedMean<SymmetricDistance, MO>
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, length: usize) -> Fallible<T> {
        let length = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _2 = T::from(2).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower) / length / _2)
    }
}


impl<MI, MO, T> MakeTransformation3<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, MO, T, T, usize> for BoundedMean<MI, MO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + DistanceCast + Float,
          for <'a> T: Sum<&'a T>,
          Self: BoundedMeanConstant<MI, MO> {
    fn make3(lower: T, upper: T, length: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }
        let _length = T::from(length).ok_or_else(|| err!(FailedCast))?;

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(
                IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
                             length),
            AllDomain::new(),
            Function::new(move |arg: &Vec<T>| arg.iter().sum::<T>() / _length),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability(lower, upper, length)?)))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity, L2Sensitivity};
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = BoundedMean::<HammingDistance, L1Sensitivity<f64>>::make(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = BoundedMean::<SymmetricDistance, L2Sensitivity<f64>>::make(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &1.).unwrap_test())
    }
}