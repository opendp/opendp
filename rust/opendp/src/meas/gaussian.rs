use std::marker::PhantomData;

use num::NumCast;

use crate::core::Measurement;
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::AllDomain;
use crate::meas::{sample_gaussian, MakeMeasurement1};

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / PI).ln();
const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

pub struct GaussianMechanism<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence, f64> for GaussianMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence> {
        let input_domain = AllDomain::new();
        let output_domain = AllDomain::new();
        let function = move |arg: &T| -> T {
            // TODO: switch to gaussian
            <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + sample_gaussian(0., sigma, false))).unwrap()
        };
        let input_metric = L2Sensitivity::new();
        let output_measure = SmoothedMaxDivergence::new();
        // https://docs.google.com/spreadsheets/d/132rAzbSDVCKqFZWeE-P8oOl9f23PzkvNwsrDV5LPkw4/edit#gid=0
        let privacy_relation = move |d_in: &f64, d_out: &(f64, f64)| {
            let (eps, delta) = d_out.clone();
            eps.min(1.) >= (*d_in / sigma) * (ADDITIVE_GAUSS_CONST + 2. * (1. / delta).ln()).sqrt()
        };
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = GaussianMechanism::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)));
    }
}
