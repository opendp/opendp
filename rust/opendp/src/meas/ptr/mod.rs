/// Algorithms that depend on the propose-test-release framework.

use std::collections::HashMap;
use std::hash::Hash;

use num::{Float, One, Zero};

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{IntDistance, SmoothedMaxDivergence, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, VectorDomain};
use crate::error::Fallible;
use crate::samplers::SampleLaplace;
use crate::traits::{CheckNull, InfCast, SaturatingAdd};

// propose-test-release count grouped by unknown categories
pub fn make_count_by_ptr<TIA, TOC>(
    scale: TOC, threshold: TOC,
) -> Fallible<Measurement<VectorDomain<AllDomain<TIA>>, MapDomain<AllDomain<TIA>, AllDomain<TOC>>, SymmetricDistance, SmoothedMaxDivergence<TOC>>>
    where TIA: Eq + Hash + Clone + CheckNull,
          TOC: 'static + Float + Zero + One + SaturatingAdd + CheckNull + InfCast<IntDistance> + SampleLaplace {
    let _2 = TOC::inf_cast(2)?;
    Ok(Measurement::new(
        VectorDomain::new_all(),
        MapDomain::new(AllDomain::new(), AllDomain::new()),
        Function::new_fallible(move |data: &Vec<TIA>| {
            let mut counts = HashMap::new();
            data.iter().for_each(|v| {
                let count = counts.entry(v.clone()).or_insert_with(TOC::zero);
                *count = TOC::one().saturating_add(count);
            });
            counts.into_iter()
                // noise output count
                .map(|(k, v)| TOC::sample_laplace(v, scale, false).map(|v| (k, v)))
                // remove counts that fall below threshold
                .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        SymmetricDistance::default(),
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &IntDistance, &(eps, del): &(TOC, TOC)| {
            if eps.is_sign_negative() || eps.is_zero() {
                return fallible!(FailedRelation, "epsilon must be positive");
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(FailedRelation, "delta must be positive");
            }
            let d_in = TOC::inf_cast(d_in)?;

            let ideal_scale = d_in / eps;
            let ideal_threshold = (d_in / (_2 * del)).ln() * ideal_scale + d_in;

            Ok(scale >= ideal_scale && threshold >= ideal_threshold)
        })))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ptr_count_by() -> Fallible<()> {

        let max_influence = 1;
        let sensitivity = max_influence as f64;
        let epsilon = 2.;
        let delta = 1e-6;
        let scale = sensitivity / epsilon;
        let threshold = (max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64;
        println!("{:?}", threshold);
        let measurement = make_count_by_ptr::<char, f64>( scale, threshold)?;
        let ret = measurement.invoke(
            &vec!['a', 'b', 'a', 'a', 'a', 'a', 'b', 'a', 'a', 'a', 'a'])?;
        println!("stability eval: {:?}", ret);

        assert!(measurement.check(&max_influence, &(epsilon, delta))?);
        Ok(())
    }
}