#[cfg(feature = "ffi")]
mod ffi;

use crate::core::{Domain, Measure, Measurement, Metric, PrivacyMap};
use crate::domains::SizedDomain;
use crate::error::Fallible;
use crate::measures::{FixedSmoothedMaxDivergence, MaxDivergence};
use crate::traits::{CollectionSize, ExactIntCast, InfDiv, InfExpM1, InfLn1P, InfMul};

pub trait IsSizedDomain: Domain {
    fn get_size(&self) -> Fallible<usize>;
}
impl<D: Domain> IsSizedDomain for SizedDomain<D>
where
    D::Carrier: CollectionSize,
{
    fn get_size(&self) -> Fallible<usize> {
        Ok(self.size)
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

impl<Q> AmplifiableMeasure for MaxDivergence<Q>
where
    Q: ExactIntCast<usize> + InfMul + InfExpM1 + InfLn1P + InfDiv + Clone,
{
    fn amplify(&self, epsilon: &Q, population_size: usize, sample_size: usize) -> Fallible<Q> {
        let sampling_rate =
            Q::exact_int_cast(sample_size)?.inf_div(&Q::exact_int_cast(population_size)?)?;
        epsilon
            .clone()
            .inf_exp_m1()?
            .inf_mul(&sampling_rate)?
            .inf_ln_1p()
    }
}
impl<Q> AmplifiableMeasure for FixedSmoothedMaxDivergence<Q>
where
    Q: ExactIntCast<usize> + InfMul + InfExpM1 + InfLn1P + InfDiv + Clone,
{
    fn amplify(
        &self,
        (epsilon, delta): &(Q, Q),
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<(Q, Q)> {
        let sampling_rate =
            Q::exact_int_cast(sample_size)?.inf_div(&Q::exact_int_cast(population_size)?)?;
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
/// * `DO` - Output Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Metric.
pub fn make_population_amplification<DIA, TO, MI, MO>(
    measurement: &Measurement<DIA, TO, MI, MO>,
    population_size: usize,
) -> Fallible<Measurement<DIA, TO, MI, MO>>
where
    DIA: IsSizedDomain,
    MI: 'static + Metric,
    MO: 'static + AmplifiableMeasure,
{
    let mut measurement = measurement.clone();
    let sample_size = measurement.input_domain.get_size()?;
    if population_size < sample_size {
        return fallible!(
            MakeMeasurement,
            "population size cannot be less than sample size"
        );
    }

    let privacy_map = measurement.privacy_map;
    let output_measure: MO = measurement.output_measure.clone();

    measurement.privacy_map = PrivacyMap::new_fallible(move |d_in| {
        output_measure.amplify(&privacy_map.eval(d_in)?, population_size, sample_size)
    });

    Ok(measurement)
}

#[cfg(test)]
mod test {
    use crate::combinators::make_population_amplification;
    use crate::error::Fallible;
    use crate::measurements::make_base_laplace;
    use crate::metrics::SymmetricDistance;
    use crate::transformations::make_sized_bounded_mean;

    #[test]
    fn test_amplifier() -> Fallible<()> {
        let meas = (make_sized_bounded_mean::<SymmetricDistance, _>(10, (0., 10.))?
            >> make_base_laplace(0.5, None)?)?;
        let amp = make_population_amplification(&meas, 100)?;
        amp.function.eval(&vec![1.; 10])?;
        assert!(meas.check(&2, &(2. + 1e-6))?);
        assert!(!meas.check(&2, &2.)?);
        assert!(amp.check(&2, &0.4941)?);
        assert!(!amp.check(&2, &0.494)?);
        Ok(())
    }
}
