use crate::error::Fallible;
use crate::metrics::DiscreteDistance;
use crate::measures::MaxDivergence;
use crate::domains::{AtomDomain, VectorDomain};
use crate::core::{Function, Measurement, PrivacyMap};
use crate::traits::{InfSub, InfDiv, InfLn, InfMul, samplers::SampleBernoulli};


/// Make a Measurement that implements Basic RAPPOR
/// 
/// # Arguments
/// * `f` - Per-bit flipping probability. Must be in $(0, 1]$.
///
/// See paper for more details: https://arxiv.org/abs/1407.6981
///
/// eps = ln((2-f)/f) 2
pub fn make_rappor(
    input_domain: VectorDomain<AtomDomain<bool>>,
    input_metric: DiscreteDistance,
    f: f64,
    constant_time: bool
) -> Fallible<Measurement<VectorDomain<AtomDomain<bool>>, Vec<bool>, DiscreteDistance, MaxDivergence<f64>>> {
    if input_domain.size.is_none() {
        return fallible!(MakeMeasurement, "RAPPOR requires a known number of categories.")
    }

    if !(0.0..=1.0).contains(&f) {
        return fallible!(MakeMeasurement, "f must be in (0, 1].")
    }

    let epsilon = (2.0f64).inf_sub(&f)?.inf_div(&f)?.inf_ln()?.inf_mul(&2.0)?;
    let f_2 = f.inf_div(&2.0)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<bool>| {
            arg.iter().map(|b| {
                Ok(*b ^ bool::sample_bernoulli(f_2, constant_time)?)
            }).collect::<Fallible<Vec<bool>>>()
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |&d_in: &u32| {
            if d_in == 0 {
                return Ok(0.0);
            }
            if d_in > 1 {
                return fallible!(FailedFunction, "d_in must be 0 or 1.");
            }
            Ok(epsilon)
        })
    )
}




pub fn debias_basic_rappor(answers: Vec<Vec<bool>>, f: f64) -> Fallible<Vec<f64>> {
    if answers.len() == 0 {
        return fallible!(FailedFunction, "No answers provided.");
    }
    if !(0.0..=1.0).contains(&f) {
        return fallible!(FailedFunction, "f must be in (0, 1].")
    }
    
    let n = answers.len() as f64;
    let k = answers[0].len();
    let mut counts = vec![0.0; k];

    if answers.iter().any(|a| a.len() != k) {
        return fallible!(FailedFunction, "Answers have inconsistent lengths.");
    }

    answers.into_iter().for_each(|answer| {
        counts.iter_mut().zip(answer).for_each(|(c, a)| {
            if a {
                *c += 1.0;
            }
        });
    });

    Ok(counts.into_iter().map(|y_i| {
        (y_i - ((f / 2.0) * n)) / (1.0 - f)
    }).collect())
    
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_rappor() -> Fallible<()> {
        let rappor = make_rappor(
            VectorDomain::new(AtomDomain::default()).with_size(10),
            DiscreteDistance::default(),
            0.5,
            false
        )?;
        rappor.invoke(&vec![true, false, true, false, true, false, true, false, true, false])?;
        assert_eq!(rappor.map(&1)?, 2.1972245773362196);
        Ok(())
    }
}

