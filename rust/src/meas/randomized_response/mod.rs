#[cfg(feature="ffi")]
mod ffi;

use std::collections::HashSet;
use std::hash::Hash;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{MaxDivergence, SymmetricDistance, IntDistance};
use crate::dom::AllDomain;
use crate::error::Fallible;
use crate::samplers::{SampleBernoulli, SampleUniformInt};
use crate::traits::{ExactIntCast, CheckNull, DistanceConstant, InfCast, InfLn, InfSub};
use num::Float;

// There are two constructors:
// 1. make_randomized_response_bool
//    a simple implementation specifically for booleans
// 2. make_randomized_response
//    for any categorical type with t > 1 categories
//
// The general rule is eps = (p / p').ln(), where p' = (1 - p) / (t - 1), and t = # categories
// See paper for more details: http://csce.uark.edu/~xintaowu/publ/DPL-2014-003.pdf
//
// In the case of privatizing a balanced coin flip,
//     t = 2, p = .75, giving eps = ln(.75 / .25) = ln(3)


pub fn make_randomized_response_bool<Q>(
    prob: Q, constant_time: bool
) -> Fallible<Measurement<AllDomain<bool>, AllDomain<bool>, SymmetricDistance, MaxDivergence<Q>>>
    where bool: SampleBernoulli<Q>,
          Q: 'static + Float + ExactIntCast<IntDistance> + DistanceConstant<IntDistance> + InfSub + InfLn,
          IntDistance: InfCast<Q> {

    // number of categories t is 2, and probability is bounded below by 1/t
    if !(Q::exact_int_cast(2)?.recip()..Q::one()).contains(&prob) {
        return fallible!(MakeTransformation, "probability must be within [0.5, 1)")
    }

    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &bool| {
            let lie = bool::sample_bernoulli(prob, constant_time)?;
            Ok(if bool::sample_bernoulli(prob, constant_time)? { *arg } else { lie })
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(
            // ln(p / (1 - prob))
            prob.inf_div(&Q::one().neg_inf_sub(&prob)?)?.inf_ln()?),
    ))
}

pub fn make_randomized_response<T, Q>(
    categories: HashSet<T>, prob: Q, constant_time: bool
) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<Q>>>
    where T: 'static + Clone + Eq + Hash + CheckNull,
          bool: SampleBernoulli<Q>,
          Q: 'static + Float + ExactIntCast<usize> + DistanceConstant<IntDistance> + InfSub + InfLn,
          IntDistance: InfCast<Q> {

    let categories = categories.into_iter().collect::<Vec<_>>();
    if categories.len() < 2 {
        return fallible!(MakeTransformation, "length of categories must be at least two")
    }
    let num_categories = Q::exact_int_cast(categories.len())?;

    if !(num_categories.recip()..Q::one()).contains(&prob) {
        return fallible!(MakeTransformation, "probability must be within [1/num_categories, 1)")
    }

    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |truth: &T| {
            // find index of truth in category set, or None
            let index = categories.iter().position(|cat| cat == truth);

            // randomly sample a lie from among the categories with equal probability
            // if truth in categories, sample among n - 1 categories
            let mut sample = usize::sample_uniform_int_0_u(
                categories.len() - if index.is_some() { 1 } else { 0 })?;
            // shift the sample by one if index is greater or equal to the index of truth
            if let Some(i) = index { if sample >= i { sample += 1 } }
            let lie = &categories[sample];

            // return the truth if we chose to be honest and the truth is in the category set
            let be_honest = bool::sample_bernoulli(prob, constant_time)?;
            let is_member = index.is_some();
            Ok(if be_honest && is_member { truth } else { lie }.clone())
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(
            // d_out >= d_in * (p / p').ln()
            // where off-diagonal probability p' = (1 - p) / (t - 1)
            // d_out >= d_in * (p / (1 - p) * (t - 1)).ln()
            // where t = num_categories
            prob.inf_mul(&num_categories.inf_sub(&Q::one())?)?
                .inf_div(&Q::one().neg_inf_sub(&prob)?)?.inf_ln()?),
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_bool() -> Fallible<()> {
        let ran_res = make_randomized_response_bool(0.75, false)?;
        let res = ran_res.invoke(&false)?;
        println!("{:?}", res);
        assert!(ran_res.check(&1, &3.0.ln())?);
        assert!(!ran_res.check(&1, &2.99999.ln())?);
        Ok(())
    }
    #[test]
    fn test_bool_extremes() -> Fallible<()> {
        // 50% chance that the output is correct means all information is lost, query is 0-dp
        let ran_res = make_randomized_response_bool(0.5, false)?;
        assert!(ran_res.check(&1, &0.0)?);
        // 100% chance that the output is correct is inf-dp, so expect an error
        assert!(make_randomized_response_bool(1.0, false).is_err());
        Ok(())
    }
    #[test]
    fn test_cat() -> Fallible<()> {
        let ran_res = make_randomized_response(
            HashSet::from_iter(vec![2, 3, 5, 6].into_iter()),
            0.75, false)?;
        let res = ran_res.invoke(&3)?;
        println!("{:?}", res);
        // (.75 * 3 / .25) = 9
        assert!(ran_res.check(&1, &9.0.ln())?);
        assert!(!ran_res.check(&1, &8.99999.ln())?);
        Ok(())
    }
    #[test]
    fn test_cat_extremes() -> Fallible<()> {
        let ran_res = make_randomized_response(
            HashSet::from_iter(vec![2, 3, 5, 7].into_iter()),
            0.25, false)?;
        assert!(ran_res.check(&1, &0.)?);
        assert!(make_randomized_response(
            HashSet::from_iter(vec![2, 3, 5, 7].into_iter()),
            1., false).is_err());
        Ok(())
    }
}