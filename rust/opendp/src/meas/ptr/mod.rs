/// Algorithms that depend on the propose-test-release framework.

use std::hash::Hash;

use num::{Float, Zero};

use crate::core::{Function, Measurement, PrivacyRelation, Transformation, Domain};
use crate::dist::{IntDistance, SmoothedMaxDivergence, SymmetricDistance, L1Distance};
use crate::dom::{AllDomain, MapDomain};
use crate::error::Fallible;
use crate::samplers::SampleLaplace;
use crate::traits::{CheckNull, InfCast, SaturatingAdd};


pub fn make_parallel_ptr<DI, TK, TC>(
    query: Transformation<DI, MapDomain<AllDomain<TK>, AllDomain<TC>>, SymmetricDistance, L1Distance<TC>>,
    scale: TC, threshold: TC
) -> Fallible<Measurement<DI, MapDomain<AllDomain<TK>, AllDomain<TC>>, SymmetricDistance, SmoothedMaxDivergence<TC>>>
    where DI: 'static + Domain,
          TK: 'static + CheckNull + Eq + Hash,
          TC: 'static + SampleLaplace + PartialOrd + Float + CheckNull + Clone + InfCast<IntDistance> {

    let Transformation {
        input_domain, output_domain, function,
        input_metric, stability_relation, ..
    } = query;

    let forward_map = stability_relation.forward_map
        .ok_or_else(|| err!(MakeMeasurement, "query's forward map must be defined"))?;
    let _2 = TC::inf_cast(2)?;

    Ok(Measurement::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |data: &DI::Carrier|
            function.eval(data)?.into_iter()
                // noise aggregates
                .map(|(k, v)| TC::sample_laplace(v, scale, false).map(|v| (k, v)))
                // remove aggregates that fall below threshold
                .filter(|res| res.as_ref().map(|(_k, c)| c >= &threshold).unwrap_or(true))
                // fail the whole computation if any noise addition failed
                .collect()),
        input_metric,
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |d_in: &IntDistance, &(eps, del): &(TC, TC)| {
            if eps.is_sign_negative() || eps.is_zero() {
                return fallible!(FailedRelation, "epsilon must be positive");
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(FailedRelation, "delta must be positive");
            }
            let l1_sensitivity = forward_map(d_in)?;
            let d_in = TC::inf_cast(*d_in)?;

            let ideal_scale = l1_sensitivity / eps;

            // scalar sensitivity is bounded above by l1 sensitivity
            // this makes ideal_threshold loose
            let scalar_sensitivity = l1_sensitivity;

            let ideal_threshold = (d_in / (_2 * del)).ln() * ideal_scale + scalar_sensitivity;

            Ok(scale >= ideal_scale && threshold >= ideal_threshold)
        })
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::make_count_by;

    #[test]
    fn test_ptr_count_by() -> Fallible<()> {

        let max_influence = 1;
        let sensitivity = max_influence as f64;
        let epsilon = 2.;
        let delta = 1e-6;
        let scale = sensitivity / epsilon;
        let threshold = (max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64;
        println!("{:?}", threshold);
        let measurement = make_parallel_ptr(make_count_by::<char, f64>()?, scale, threshold)?;
        let ret = measurement.invoke(
            &vec!['a', 'b', 'a', 'a', 'a', 'a', 'b', 'a', 'a', 'a', 'a'])?;
        println!("stability eval: {:?}", ret);

        assert!(measurement.check(&max_influence, &(epsilon, delta))?);
        Ok(())
    }
}