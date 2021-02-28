
use crate::core::Measurement;
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::{AllDomain, SizedDomain, VectorDomain};
use crate::meas::{MakeMeasurement1, MakeMeasurement2, sample_laplace};

pub struct LaplaceMechanism;

// laplace for scalar-valued query
impl MakeMeasurement1<AllDomain<f64>, AllDomain<f64>, L1Sensitivity<f64>, MaxDivergence, f64> for LaplaceMechanism {
    fn make1(sigma: f64) -> Measurement<AllDomain<f64>, AllDomain<f64>, L1Sensitivity<f64>, MaxDivergence> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            move |v: &f64| v + sample_laplace(sigma).unwrap(),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            move |&d_in: &f64, &d_out: &f64| d_out >= d_in / sigma
        )
    }
}

pub struct VectorLaplaceMechanism;

// laplace for vector-valued query
impl MakeMeasurement2<SizedDomain<VectorDomain<AllDomain<f64>>>, VectorDomain<AllDomain<f64>>, L1Sensitivity<f64>, MaxDivergence, usize, f64> for VectorLaplaceMechanism {
    fn make2(length: usize, sigma: f64) -> Measurement<SizedDomain<VectorDomain<AllDomain<f64>>>, VectorDomain<AllDomain<f64>>, L1Sensitivity<f64>, MaxDivergence> {
        Measurement::new(
            SizedDomain::new(VectorDomain::new_all(), length),
            VectorDomain::new_all(),
            move |arg: &Vec<f64>| arg.into_iter()
                .map(|v| v + sample_laplace(sigma).unwrap())
                .collect(),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            move |&d_in: &f64, &d_out: &f64| d_out >= d_in / sigma)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = LaplaceMechanism::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = VectorLaplaceMechanism::make(3, 1.0);
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }
}

