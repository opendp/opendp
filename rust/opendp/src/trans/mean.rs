use crate::core::{DatasetMetric, SensitivityMetric, Transformation, Function, Metric, StabilityRelation};
use std::ops::{Sub, Div};
use std::iter::Sum;
use crate::traits::DistanceConstant;
use crate::error::Fallible;
use crate::dom::{VectorDomain, IntervalDomain, AllDomain, SizedDomain};
use std::collections::Bound;
use crate::dist::{HammingDistance, SymmetricDistance};
use num::{NumCast, Float};

pub trait BoundedMeanConstant<MI: Metric, MO: Metric> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric<Distance=T>, T> BoundedMeanConstant<HammingDistance, MO> for (HammingDistance, MO)
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, length: usize) -> Fallible<T> {
        let length = T::from(length).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower) / length)
    }
}

// postprocessing the sum
impl<MO: Metric<Distance=T>, T> BoundedMeanConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where T: Sub<Output=T> + Div<Output=T> + NumCast {
    fn get_stability(lower: T, upper: T, length: usize) -> Fallible<T> {
        Ok((upper - lower) / num_cast!(length; T)? / num_cast!(2; T)?)
    }
}

pub fn make_bounded_mean<MI, MO>(
    lower: MO::Distance, upper: MO::Distance, length: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<MO::Distance>>>, AllDomain<MO::Distance>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric,
          MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Float,
          for <'a> MO::Distance: Sum<&'a MO::Distance>,
          (MI, MO): BoundedMeanConstant<MI, MO> {
    if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }
    let _length = num_cast!(length; MO::Distance)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
                         length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<MO::Distance>| arg.iter().sum::<MO::Distance>() / _length),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability(lower, upper, length)?)))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity, L2Sensitivity};
    use crate::error::ExplainUnwrap;
    use crate::trans::mean::make_bounded_mean;

    #[test]
    fn test_make_bounded_mean_hamming() {
        let transformation = make_bounded_mean::<HammingDistance, L1Sensitivity<f64>>(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &2.).unwrap_test())
    }

    #[test]
    fn test_make_bounded_mean_symmetric() {
        let transformation = make_bounded_mean::<SymmetricDistance, L2Sensitivity<f64>>(0., 10., 5).unwrap_test();
        let arg = vec![1., 2., 3., 4., 5.];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3.;
        assert_eq!(ret, expected);
        assert!(transformation.stability_relation.eval(&1, &1.).unwrap_test())
    }
}