use bitvec::prelude::{BitVec, bitvec, Lsb0};

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AtomDomain, BitVector, BitVectorDomain, VectorDomain};
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::{DiscreteDistance, HammingDistance};
use crate::traits::{samplers::sample_bernoulli_float, InfDiv, InfLn, InfMul, InfSub};

/// Make a Measurement that implements Basic RAPPOR
///
/// # Citations
/// * [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal Response](https://arxiv.org/abs/1407.6981)
///
/// # Arguments
/// * `f` - Per-bit flipping probability. Must be in $(0, 1]$.
/// * `m` - number of ones set in each boolean vector (1 if one-hot encoding, more if using a bloom filter)
///
/// eps = 2mln((2-f)/f)
pub fn make_rappor(
    input_domain: VectorDomain<BitVectorDomain>,
    input_metric: HammingDistance,
    f: f64,
    constant_time: bool,
) -> Fallible<
    Measurement<VectorDomain<BitVectorDomain>, BitVectorDomain, HammingDistance, MaxDivergence<f64>>,
> {

    if input_domain.size.is_none() && input_domain.element_domain.size.is_none(){
        return fallible!(
            MakeMeasurement,
            "RAPPOR requires a known number of categories"
        );
    }

    let m = input_domain.element_domain.max_weight.unwrap_or_else(||
        return fallible!(
            MakeMeasurement,
            "RAPPOR requires a known number of categories"
        )
    );
    
    if f <= 0.0 || f > 1.0 {
        return fallible!(MakeMeasurement, "f must be in (0, 1]");
    };

    // priv = 2mln((2-f)/f)
    let epsilon = (2.0f64)
        .inf_sub(&f)?
        .inf_div(&f)?
        .inf_ln()?
        .inf_mul(&2.0)?
        .inf_mul(&f64::from(m))?;
    let f_2 = f.inf_div(&2.0)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<BitVector>| {
            arg.iter()
                .map(|b| Ok(*b ^ sample_bernoulli_float(f_2, constant_time)?))
                .collect::<Fallible<Vec<BitVector>>>()
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
        }),
    )
}

pub fn debias_basic_rappor(answers: Vec<Vec<bool>>, f: f64) -> Fallible<Vec<f64>> {
    if answers.len() == 0 {
        return fallible!(FailedFunction, "No answers provided");
    }
    if f <= 0.0 || f > 1.0 {
        return fallible!(FailedFunction, "f must be in (0, 1]");
    }

    let n = answers.len() as f64;
    let k = answers[0].len();
    let mut counts = vec![0.0; k];

    if answers.iter().any(|a| a.len() != k) {
        return fallible!(FailedFunction, "Answers have inconsistent lengths");
    }

    answers.into_iter().for_each(|answer| {
        counts.iter_mut().zip(answer).for_each(|(c, a)| {
            if a {
                *c += 1.0;
            }
        });
    });

    Ok(counts
        .into_iter()
        .map(|y_i| (y_i - ((f / 2.0) * n)) / (1.0 - f))
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_rappor() -> Fallible<()> {
        let rappor = make_rappor(
            VectorDomain::new(BitVectorDomain::new().with_size(10).with_max_weight(1)).with_size(10),
            HammingDistance::default(),
            0.5,
            false,
        )?;
        rappor.invoke(&vec![bitvec![usize, Lsb0;
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]])?;
        assert_eq!(rappor.map(&1)?, 2.1972245773362196);
        Ok(())
    }
    #[test]
    fn test_debias_rappor() -> Fallible<()> {
        let f = 0.1;
        let mut answer = vec![0.0; 10];
        answer[0] = 1.0;

        let mut answers = vec![vec![false; 10]; 10];
        // dist is [10; 0x9]
        answers.iter_mut().for_each(|a| a[0] = true);

        let high = 10.555555555555555;
        let low = -0.5555555555555556;

        let expected_dist = vec![high, low, low, low, low, low, low, low, low, low];
        assert_eq!(debias_basic_rappor(answers, f)?, expected_dist);

        Ok(())
    }
}
