use std::marker::PhantomData;

use num::NumCast;

use crate::core::Measurement;
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::meas::{MakeMeasurement1, sample_laplace};

pub struct LaplaceMechanism<T> {
    data: PhantomData<T>
}

// laplace for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence, f64> for LaplaceMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence> {
        let input_domain = AllDomain::new();
        let output_domain = AllDomain::new();
        let function = move |arg: &T| -> T {
            <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + sample_laplace(sigma).unwrap())).unwrap()
        };
        let input_metric = L1Sensitivity::new();
        let output_measure = MaxDivergence::new();
        let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}

pub struct VectorLaplaceMechanism<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl<T> MakeMeasurement1<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence, f64> for VectorLaplaceMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence> {
        let input_domain = VectorDomain::new_all();
        let output_domain = VectorDomain::new_all();
        let function = move |arg: &Vec<T>| -> Vec<T> {
            arg.iter()
                .map(|v| <f64 as NumCast>::from(*v).and_then(|v| T::from(v + sample_laplace(sigma).unwrap())))
                .collect::<Option<_>>().unwrap()
        };
        let input_metric = L1Sensitivity::new();
        let output_measure = MaxDivergence::new();
        let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = LaplaceMechanism::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = VectorLaplaceMechanism::<f64>::make(1.0);
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }
}

