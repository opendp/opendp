use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use num::{Integer, Float, NumCast};

use crate::core::{Measurement, Function, PrivacyRelation, Metric, SensitivityMetric};
use crate::dist::{L1Sensitivity, L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::meas::{MakeMeasurement3};
use crate::samplers::{SampleLaplace, SampleGaussian};
use crate::error::Fallible;

// TIK: Type of Input Key
// TIC: Type of Input Count
// TOC: Type of Output Count

pub struct BaseStability<MI, TIK, TIC, TOC> {
    input_metric: PhantomData<MI>,
    data_key: PhantomData<TIK>,
    input_count: PhantomData<TIC>,
    output_count: PhantomData<TOC>,
}

pub type CountDomain<TIK, TIC> = SizedDomain<MapDomain<AllDomain<TIK>, AllDomain<TIC>>>;

// tie metric space with distribution
pub trait BaseStabilityNoise<TOC>: Metric<Distance=TOC> {
    fn noise(shift: TOC, scale: TOC, enforce_constant_time: bool) -> Fallible<TOC>;
}
impl<TOC: SampleLaplace> BaseStabilityNoise<TOC> for L1Sensitivity<TOC> {
    fn noise(shift: TOC, scale: TOC, enforce_constant_time: bool) -> Fallible<TOC> {
        TOC::sample_laplace(shift, scale, enforce_constant_time)
    }
}
impl<TOC: SampleGaussian> BaseStabilityNoise<TOC> for L2Sensitivity<TOC> {
    fn noise(shift: TOC, scale: TOC, enforce_constant_time: bool) -> Fallible<TOC> {
        TOC::sample_gaussian(shift, scale, enforce_constant_time)
    }
}

impl<MI, TIK, TIC, TOC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, MI, SmoothedMaxDivergence<TOC>, usize, TOC, TOC> for BaseStability<MI, TIK, TIC, TOC>
    where MI: BaseStabilityNoise<TOC> + SensitivityMetric<Distance=TOC>,
          TIK: Eq + Hash + Clone,
          TIC: Integer + Clone + NumCast,
          TOC: 'static + Float + Clone + PartialOrd + SampleLaplace + NumCast {
    fn make3(n: usize, scale: TOC, threshold: TOC) -> Fallible<Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, MI, SmoothedMaxDivergence<TOC>>> {
        let _n: TOC = TOC::from(n).ok_or_else(|| err!(FailedCast))?;
        let _2: TOC = TOC::from(2).ok_or_else(|| err!(FailedCast))?;

        Ok(Measurement::new(
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new_fallible(move |data: &HashMap<TIK, TIC>| {
                data.into_iter()
                    .map(|(k, c_in)| {
                        // cast the value to TOC (output count)
                        let c_out = TOC::from(c_in.clone()).ok_or_else(|| err!(FailedCast))?;
                        // noise output count
                        Ok((k.clone(), MI::noise(c_out, scale, false)?))
                    })
                    // remove counts that fall below threshold
                    .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
                    // fail the whole computation if any cast or noise addition failed
                    .collect()
            }),
            MI::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new_fallible(move |&d_in: &TOC, &(eps, del): &(TOC, TOC)|{
                // let _eps: f64 = NumCast::from(eps).unwrap();
                // let _del: f64 = NumCast::from(del).unwrap();
                // println!("eps, del: {:?}, {:?}", _eps, _del);
                let ideal_scale = d_in / (eps * _n);
                let ideal_threshold = (_2 / del).ln() * ideal_scale + _n.recip();
                // println!("ideal: {:?}, {:?}", ideal_sigma, ideal_threshold);

                if eps.is_sign_negative() || eps.is_zero() {
                    return fallible!(FailedRelation, "cause: epsilon <= 0")
                }
                if eps >= _n.ln() {
                    return fallible!(RelationDebug, "cause: epsilon >= n.ln()");
                }
                if del.is_sign_negative() || del.is_zero() {
                    return fallible!(FailedRelation, "cause: delta <= 0")
                }
                if del >= _n.recip() {
                    return fallible!(RelationDebug, "cause: del >= n.ln()");
                }
                if scale < ideal_scale {
                    return fallible!(RelationDebug, "cause: scale < d_in / (epsilon * n)")
                }
                if threshold < ideal_threshold {
                    return fallible!(RelationDebug, "cause: threshold < (2. / delta).ln() * d_in / (epsilon * n) + 1. / n");
                }
                return Ok(true)
            })))
    }
}


#[cfg(test)]
mod test_stability {
    use super::*;

    #[test]
    fn test_base_stability() {
        let mut arg = HashMap::new();
        arg.insert(true, 6);
        arg.insert(false, 4);
        let measurement = BaseStability::<L2Sensitivity<f64>, bool, i8, f64>::make(10, 0.5, 1.).unwrap();
        let ret = measurement.function.eval(&arg).unwrap();
        println!("stability eval: {:?}", ret);

        assert!(measurement.privacy_relation.eval(&1., &(2.3, 1e-5)).unwrap());
    }
}