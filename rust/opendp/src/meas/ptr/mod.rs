/// Algorithms that depend on the propose-test-release framework.

use std::collections::HashMap;
use std::hash::Hash;

use num::Float;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{IntDistance, L1Distance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, MapDomain};
use crate::error::Fallible;
use crate::samplers::SampleLaplace;
use crate::traits::{CheckNull, InfCast};

// propose-test-release count grouped by unknown categories,
// IMPORTANT: Assumes that dataset distance is bounded above by d_in.
//  This assumption holds for count queries in L1-space.
pub fn make_base_ptr<TK, TV>(
    scale: TV, threshold: TV,
) -> Fallible<Measurement<MapDomain<AllDomain<TK>, AllDomain<TV>>, MapDomain<AllDomain<TK>, AllDomain<TV>>, L1Distance<TV>, SmoothedMaxDivergence<TV>>>
    where TK: Eq + Hash + Clone + CheckNull,
          TV: 'static + Float + CheckNull + InfCast<IntDistance> + SampleLaplace {
    let _2 = TV::inf_cast(2)?;
    Ok(Measurement::new(
        MapDomain::new(AllDomain::new(), AllDomain::new()),
        MapDomain::new(AllDomain::new(), AllDomain::new()),
        Function::new_fallible(move |data: &HashMap<TK, TV>| {
            data.clone().into_iter()
                // noise output count
                .map(|(k, v)| TV::sample_laplace(v, scale, false).map(|v| (k, v)))
                // remove counts that fall below threshold
                .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        L1Distance::default(),
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &TV, &(eps, del): &(TV, TV)| {
            if eps.is_sign_negative() || eps.is_zero() {
                return fallible!(FailedRelation, "epsilon must be positive");
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(FailedRelation, "delta must be positive");
            }

            let ideal_scale = d_in / eps;
            let ideal_threshold = (d_in / (_2 * del)).ln() * ideal_scale + d_in;

            Ok(scale >= ideal_scale && threshold >= ideal_threshold)
        })))
}


#[cfg(test)]
mod tests {
    use crate::trans::make_count_by;
    use super::*;

    #[test]
    fn test_count_by_ptr() -> Fallible<()> {

        let max_influence = 1;
        let sensitivity = max_influence as f64;
        let epsilon = 2.;
        let delta = 1e-6;
        let scale = sensitivity / epsilon;
        let threshold = (max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64;
        println!("{:?}", threshold);

        let measurement = (
            make_count_by()? >>
            make_base_ptr::<char, f64>(scale, threshold)?
        )?;
        let ret = measurement.invoke(
            &vec!['a', 'b', 'a', 'a', 'a', 'a', 'b', 'a', 'a', 'a', 'a'])?;
        println!("stability eval: {:?}", ret);

        assert!(measurement.check(&max_influence, &(epsilon, delta))?);
        Ok(())
    }
}