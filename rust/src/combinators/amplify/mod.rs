#[cfg(feature = "ffi")]
mod ffi;

use crate::core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap};
use crate::domains::VectorDomain;
use crate::error::Fallible;
use crate::measures::{Approximate, MaxDivergence};
use crate::traits::{ExactIntCast, InfDiv, InfExpM1, InfLn1P, InfMul};

pub trait IsSizedDomain: Domain {
    fn get_size(&self) -> Fallible<usize>;
}
impl<D: Domain> IsSizedDomain for VectorDomain<D> {
    fn get_size(&self) -> Fallible<usize> {
        self.size.ok_or_else(|| {
            err!(
                FailedFunction,
                "elements of the vector domain have unknown size"
            )
        })
    }
}

pub trait AmplifiableMeasure: Measure {
    fn amplify(
        &self,
        budget: &Self::Distance,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<Self::Distance>;
}

impl AmplifiableMeasure for MaxDivergence {
    fn amplify(&self, epsilon: &f64, population_size: usize, sample_size: usize) -> Fallible<f64> {
        let sampling_rate =
            f64::exact_int_cast(sample_size)?.inf_div(&f64::exact_int_cast(population_size)?)?;
        epsilon
            .clone()
            .inf_exp_m1()?
            .inf_mul(&sampling_rate)?
            .inf_ln_1p()
    }
}
impl AmplifiableMeasure for Approximate<MaxDivergence> {
    fn amplify(
        &self,
        (epsilon, delta): &(f64, f64),
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<(f64, f64)> {
        let sampling_rate =
            f64::exact_int_cast(sample_size)?.inf_div(&f64::exact_int_cast(population_size)?)?;
        Ok((
            epsilon
                .clone()
                .inf_exp_m1()?
                .inf_mul(&sampling_rate)?
                .inf_ln_1p()?,
            delta.inf_mul(&sampling_rate)?,
        ))
    }
}

/// Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
/// This measurement does not perform any sampling.
/// It is useful when you have a dataset on-hand that is a simple random sample from a larger population.
///
/// The `DIA`, `DO`, `MI` and `MO` between the input measurement and amplified output measurement all match.
///
/// # Arguments
/// * `measurement` - the computation to apply privacy amplification to
/// * `population_size` - the size of the population from which the input dataset is a simple sample
///
/// # Generics
/// * `DIA` - Atomic Input Domain. The domain of individual records in the input dataset.
/// * `TO` - Output Type.
/// * `MI` - Input Metric.
/// * `MO` - Output Metric.
pub fn make_population_amplification<DI, TO, MI, MO>(
    measurement: &Measurement<DI, TO, MI, MO>,
    population_size: usize,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: IsSizedDomain,
    MI: 'static + Metric,
    MO: 'static + AmplifiableMeasure,
    (DI, MI): MetricSpace,
{
    let sample_size = measurement.input_domain.get_size()?;
    if population_size < sample_size {
        return fallible!(
            MakeMeasurement,
            "population size ({:?}) cannot be less than sample size ({:?})",
            population_size,
            sample_size
        );
    }

    let privacy_map = measurement.privacy_map.clone();
    let output_measure: MO = measurement.output_measure.clone();

    measurement.with_map(
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in| {
            output_measure.amplify(&privacy_map.eval(d_in)?, population_size, sample_size)
        }),
    )
}

#[cfg(all(test, feature = "partials"))]
mod test;
