use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{BitVector, BitVectorDomain};
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;
use crate::traits::samplers::{
    bernoulli_u64_probability, bernoulli_u64_threshold, sample_bernoulli_u64,
};
use crate::traits::{InfCast, InfDiv, InfLn, InfMul, InfSub};

#[cfg(feature = "ffi")]
mod ffi;

// Round positive flip probabilities upward, including subnormal inputs.
fn bitvec_flip_threshold(f: f64) -> u128 {
    bernoulli_u64_threshold(f / 2.0).max(1)
}

#[bootstrap(features("contrib"))]
/// Make a Measurement that implements randomized response on a bit vector.
///
/// This primitive can be useful for implementing RAPPOR.
///
/// # Citations
/// * [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal Response](https://arxiv.org/abs/1407.6981)
///
/// # Arguments
/// * `input_domain` - BitVectorDomain with max_weight and size
/// * `input_metric` - DiscreteDistance
/// * `f` - Twice the target per-bit flipping probability. Must be in $(0, 1]$.
pub fn make_randomized_response_bitvec(
    input_domain: BitVectorDomain,
    input_metric: DiscreteDistance,
    f: f64,
) -> Fallible<Measurement<BitVectorDomain, DiscreteDistance, MaxDivergence, BitVector>> {
    let m = input_domain.max_weight.ok_or_else(|| {
        err!(
            MakeMeasurement,
            "make_randomized_response_bitvec requires a max_weight on the input_domain"
        )
    })?;

    if input_domain.size.is_none() {
        return fallible!(
            MakeMeasurement,
            "make_randomized_response_bitvec requires a size on the input_domain"
        );
    }

    if !(f > 0.0 && f <= 1.0) {
        return fallible!(MakeMeasurement, "f must be in (0, 1]");
    }

    // Compute epsilon from the actual quantized flip probability.
    let threshold = bitvec_flip_threshold(f);
    let q = bernoulli_u64_probability(threshold);
    let epsilon = 1.0f64
        .inf_sub(&q)?
        .inf_div(&q)?
        .inf_ln()?
        .inf_mul(&2.0)?
        .inf_mul(&f64::inf_cast(m)?)?;
    Measurement::new(
        input_domain,
        input_metric,
        MaxDivergence,
        Function::new_fallible(move |arg: &BitVector| {
            let mut output = arg.clone();
            for mut bit in output.iter_mut() {
                *bit ^= sample_bernoulli_u64(threshold)?;
            }
            Ok(output)
        }),
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { epsilon }),
    )
}

#[bootstrap(features("contrib"))]
/// Convert a vector of randomized response bitvec responses to a frequency estimate
///
/// For $n$ responses with per-bit flip probability $p$, returns
/// $\frac{Y - np}{1 - 2p}$, where $Y$ is the vector of bit counts.
///
/// # Arguments
/// * `answers` - A vector of BitVectors with consistent size
/// * `f` - The parameter used to encode `answers`. Must be in $(0, 1)$.
///
pub fn debias_randomized_response_bitvec(answers: Vec<BitVector>, f: f64) -> Fallible<Vec<f64>> {
    if answers.is_empty() {
        return fallible!(FailedFunction, "No answers provided");
    }
    if !(f > 0.0 && f < 1.0) {
        return fallible!(FailedFunction, "f must be in (0, 1)");
    }

    // Account for the same upward quantization used by the measurement.
    let flip_prob = bernoulli_u64_probability(bitvec_flip_threshold(f));

    let n = answers.len() as f64;
    let k = answers[0].len();
    let mut counts = vec![0.0; k];

    for answer in answers {
        if answer.len() != k {
            return fallible!(FailedFunction, "Answers have inconsistent lengths");
        }
        for (c, a) in counts.iter_mut().zip(answer) {
            if a {
                *c += 1.0;
            }
        }
    }

    Ok(counts
        .into_iter()
        .map(|y_i| (y_i - flip_prob * n) / (1.0 - 2.0 * flip_prob))
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;
    use bitvec::prelude::{Lsb0, bitvec};

    #[test]
    fn test_make_randomized_response_bitvec() -> Fallible<()> {
        let m_rr = make_randomized_response_bitvec(
            BitVectorDomain::new().with_max_weight(1).with_size(10),
            DiscreteDistance,
            0.5,
        )?;
        m_rr.invoke(&bitvec![u8, Lsb0;
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ])?;
        assert_eq!(m_rr.map(&1)?, 2.1972245773362196);
        assert_eq!(m_rr.map(&2)?, m_rr.map(&1)?);
        Ok(())
    }

    #[test]
    fn test_make_randomized_response_bitvec_validation() -> Fallible<()> {
        use crate::core::Domain;

        let domain = BitVectorDomain::new().with_max_weight(1).with_size(8);
        assert!(domain.member(&bitvec![u8, Lsb0; 1, 0, 0, 0, 0, 0, 0, 0])?);
        assert!(!domain.member(&bitvec![u8, Lsb0; 1, 0, 0, 0, 0, 0, 0, 0, 0])?);
        assert!(!domain.member(&bitvec![u8, Lsb0; 1, 1, 0, 0, 0, 0, 0, 0])?);
        assert!(
            make_randomized_response_bitvec(domain.clone(), DiscreteDistance, f64::NAN).is_err()
        );
        assert!(
            make_randomized_response_bitvec(domain.without_size(), DiscreteDistance, 0.5).is_err()
        );
        Ok(())
    }

    #[test]
    fn test_make_randomized_response_bitvec_randomizes_last_bit() -> Fallible<()> {
        let m_rr = make_randomized_response_bitvec(
            BitVectorDomain::new().with_max_weight(1).with_size(4),
            DiscreteDistance,
            1.0,
        )?;
        let input = bitvec![u8, Lsb0; 0, 0, 0, 1];

        let mut seen_true = false;
        let mut seen_false = false;

        for _ in 0..64 {
            let release = m_rr.invoke(&input)?;
            let last_bit = release[release.len() - 1];
            seen_true |= last_bit;
            seen_false |= !last_bit;
            if seen_true && seen_false {
                break;
            }
        }

        assert!(
            seen_true && seen_false,
            "expected last bit to be randomized"
        );
        Ok(())
    }

    #[test]
    fn test_debias_rr_bitvec_quantized_flip_probability() -> Fallible<()> {
        let estimate =
            debias_randomized_response_bitvec(vec![bitvec![u8, Lsb0; 0]], f64::MIN_POSITIVE)?;
        assert_eq!(estimate, vec![-2f64.powi(-64)]);
        assert!(debias_randomized_response_bitvec(vec![bitvec![u8, Lsb0; 0]], 0.0).is_err());
        assert!(debias_randomized_response_bitvec(vec![bitvec![u8, Lsb0; 0]], 1.0).is_err());
        assert!(debias_randomized_response_bitvec(vec![bitvec![u8, Lsb0; 0]], f64::NAN).is_err());
        Ok(())
    }

    #[test]
    fn test_debias_rr_bitvec() -> Fallible<()> {
        let f = 0.1;
        let answers = vec![bitvec![u8, Lsb0; 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; 10];

        let high = 10.555555555555555;
        let low = -0.5555555555555556;

        let expected_dist = vec![high, low, low, low, low, low, low, low, low, low];
        assert_eq!(
            debias_randomized_response_bitvec(answers, f)?,
            expected_dist
        );

        Ok(())
    }
}
