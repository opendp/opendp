use std::collections::HashMap;
use std::hash::Hash;

use num::{Integer, Float, NumCast, Zero};

use crate::core::{Measurement, Function, PrivacyRelation, SensitivityMetric};
use crate::dist::{L1Distance, L2Distance, SmoothedMaxDivergence, EpsilonDelta};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::samplers::{SampleLaplace, SampleGaussian};
use crate::error::Fallible;
use crate::chain::BasicCompositionDistance;

// TIK: Type of Input Key
// TIC: Type of Input Count
// TOC: Type of Output Count (equal to MI::Distance)

pub type CountDomain<TIK, TIC> = SizedDomain<MapDomain<AllDomain<TIK>, AllDomain<TIC>>>;

// tie metric with distribution
pub trait BaseStabilityNoise: SensitivityMetric {
    fn noise(shift: Self::Distance, scale: Self::Distance, constant_time: bool) -> Fallible<Self::Distance>;
}
impl<TOC: SampleLaplace> BaseStabilityNoise for L1Distance<TOC> {
    fn noise(shift: Self::Distance, scale: Self::Distance, constant_time: bool) -> Fallible<Self::Distance> {
        Self::Distance::sample_laplace(shift, scale, constant_time)
    }
}
impl<TOC: SampleGaussian> BaseStabilityNoise for L2Distance<TOC> {
    fn noise(shift: Self::Distance, scale: Self::Distance, constant_time: bool) -> Fallible<Self::Distance> {
        Self::Distance::sample_gaussian(shift, scale, constant_time)
    }
}

pub fn make_base_stability<MI, TIK, TIC>(
    n: usize, scale: MI::Distance, threshold: MI::Distance
) -> Fallible<Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, MI::Distance>, MI, SmoothedMaxDivergence<MI::Distance>>>
    where MI: BaseStabilityNoise,
          TIK: Eq + Hash + Clone,
          TIC: Integer + Clone + NumCast,
          MI::Distance: 'static + Float + Clone + PartialOrd + NumCast + BasicCompositionDistance {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    if threshold.is_sign_negative() {
        return fallible!(MakeMeasurement, "threshold must not be negative")
    }
    let _n = num_cast!(n; MI::Distance)?;
    let _2 = num_cast!(2; MI::Distance)?;

    Ok(Measurement::new(
        SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
        SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
        Function::new_fallible(move |data: &HashMap<TIK, TIC>| {
            data.iter()
                .map(|(k, c_in)| {
                    // cast the value to MI::Distance (output count)
                    let c_out = num_cast!(c_in.clone(); MI::Distance)?;
                    // noise output count
                    Ok((k.clone(), MI::noise(c_out, scale, false)?))
                })
                // remove counts that fall below threshold
                .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        MI::default(),
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &MI::Distance, d_out: &EpsilonDelta<MI::Distance>|{
            let EpsilonDelta { epsilon, delta } = d_out.clone();

            let ideal_scale = d_in / (epsilon * _n);
            let ideal_threshold = (_2 / delta).ln() * ideal_scale + _n.recip();
            // println!("ideal: {:?}, {:?}", ideal_sigma, ideal_threshold);

            if epsilon.is_sign_negative() || epsilon.is_zero() {
                return fallible!(FailedRelation, "cause: epsilon <= 0")
            }
            if epsilon >= _n.ln() {
                return fallible!(RelationDebug, "cause: epsilon >= n.ln()");
            }
            if delta.is_sign_negative() || delta.is_zero() {
                return fallible!(FailedRelation, "cause: delta <= 0")
            }
            if delta >= _n.recip() {
                return fallible!(RelationDebug, "cause: del >= n.ln()");
            }
            if scale < ideal_scale {
                return fallible!(RelationDebug, "cause: scale < d_in / (epsilon * n)")
            }
            if threshold < ideal_threshold {
                return fallible!(RelationDebug, "cause: threshold < (2. / delta).ln() * d_in / (epsilon * n) + 1. / n");
            }
            Ok(true)
        })
    ))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_stability() -> Fallible<()> {
        let mut arg = HashMap::new();
        arg.insert(true, 6);
        arg.insert(false, 4);
        let measurement = make_base_stability::<L2Distance<f64>, bool, i8>(10, 0.5, 1.)?;
        let _ret = measurement.function.eval(&arg)?;
        // println!("stability eval: {:?}", ret);

        assert!(measurement.privacy_relation.eval(&1., &(2.3, 1e-5))?);
        Ok(())
    }
}