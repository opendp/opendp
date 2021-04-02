use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{AddAssign, Add, Mul, Div};

use num::{Integer, One, Zero, Float, NumCast};

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Sensitivity, L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::meas::{MakeMeasurement3};
use crate::samplers::{SampleLaplace, SampleGaussian};
use crate::error::Fallible;

fn privacy_relation<TOC>(d_in: TOC, (eps, del): (TOC, TOC), n: usize, sigma: TOC, threshold: TOC) -> bool
    where TOC: Float + Add<Output=TOC> + Mul<Output=TOC> + Div<Output=TOC> + PartialOrd + NumCast {
    let n: TOC = if let Some(x) = TOC::from(n) {x} else {return false};
    let _2_: TOC = if let Some(x) = TOC::from(2) {x} else {return false};
    let ideal_sigma = d_in / (eps * n);
    let ideal_threshold = (_2_ / del).ln() * ideal_sigma + n.recip();
    // println!("ideal: {:?}, {:?}", ideal_sigma, ideal_threshold);

    if eps.is_sign_negative() || eps.is_zero() || eps >= n.ln() {
        // println!("failed:   eps >= n.ln()");
        return false
    }
    if del.is_sign_negative() || del.is_zero() || del >= n.recip() {
        // println!("failed:   del >= 1. / n");
        return false
    }
    // check that sigma is large enough
    if sigma < ideal_sigma {
        // println!("failed:   sigma < d_in / (eps * n)");
        return false
    }
    // check that threshold is large enough
    if threshold < ideal_threshold {
        // println!("failed:   threshold < (2. / del).ln() * sigma + 1. / n");
        return false
    }
    return true
}

// TIK: Type of Input Key
// TIC: Type of Input Count
// TOC: Type of Output Count
fn stability_mechanism<TIK, TIC, TOC, F: Fn(TOC) -> Fallible<TOC>>(
    counts: &HashMap<TIK, TIC>,
    noise: F,
    threshold: TOC,
) -> Fallible<HashMap<TIK, TOC>>
    where TIK: Eq + Hash + Clone,
          TIC: NumCast + Clone,
          TOC: NumCast + PartialOrd {
    counts.into_iter()
        // noise the float version of each count
        .map(|(k, c)| Ok((k.clone(), noise(TOC::from(c.clone()).ok_or_else(|| err!(FailedCast))?)?)))
        // remove counts that fall below threshold
        .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
        // fail the whole computation if any cast or noise addition failed
        .collect::<Result<_, _>>()
}

pub struct BaseStability<MI, TIK, TIC, TOC> {
    input_metric: PhantomData<MI>,
    data_key: PhantomData<TIK>,
    input_count: PhantomData<TIC>,
    output_count: PhantomData<TOC>,
}

pub type CountDomain<TIK, TIC> = SizedDomain<MapDomain<AllDomain<TIK>, AllDomain<TIC>>>;

// L1
impl<TIK, TIC, TOC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, L1Sensitivity<TOC>, SmoothedMaxDivergence<TOC>, usize, TOC, TOC> for BaseStability<L1Sensitivity<TOC>, TIK, TIC, TOC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone + NumCast,
          TOC: 'static + Clone + Float + PartialOrd + SampleLaplace + NumCast {
    fn make3(n: usize, scale: TOC, threshold: TOC) -> Fallible<Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, L1Sensitivity<TOC>, SmoothedMaxDivergence<TOC>>> {
        Ok(Measurement::new(
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new_fallible(enclose!((scale, threshold), move |data: &HashMap<TIK, TIC>|
                stability_mechanism(data, |shift| TOC::sample_laplace(shift, scale, false), threshold))),
            L1Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new(move |&d_in: &TOC, &d_out: &(TOC, TOC)|
                privacy_relation(d_in, d_out, n, scale, threshold))))
    }
}

// L2
impl<TIK, TIC, TOC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, L2Sensitivity<TOC>, SmoothedMaxDivergence<TOC>, usize, TOC, TOC> for BaseStability<L2Sensitivity<TOC>, TIK, TIC, TOC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone + NumCast,
          TOC: 'static + Clone + Float + SampleGaussian + PartialOrd + NumCast {
    fn make3(n: usize, sigma: TOC, threshold: TOC) -> Fallible<Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, TOC>, L2Sensitivity<TOC>, SmoothedMaxDivergence<TOC>>> {
        Ok(Measurement::new(
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new_fallible(enclose!((sigma, threshold), move |data: &HashMap<TIK, TIC>|
                stability_mechanism(data, |shift| TOC::sample_gaussian(shift, sigma, false), threshold))),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new(move |&d_in: &TOC, &d_out: &(TOC, TOC)|
                privacy_relation(d_in, d_out, n, sigma, threshold))))
    }
}