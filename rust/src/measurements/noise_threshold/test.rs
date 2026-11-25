use core::f64;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::*;
use crate::{
    domains::VectorDomain,
    measures::{Approximate, MaxDivergence},
    metrics::SymmetricDistance,
    traits::{CheckAtom, CheckNull, InfCast},
    transformations::make_count_by,
};

#[test]
fn test_count_by_threshold() -> Fallible<()> {
    let max_influence = 1;
    let sensitivity = max_influence as f64;
    let epsilon = 2.;
    let delta = 1e-6;
    let scale = sensitivity / epsilon;
    let threshold =
        u32::inf_cast((max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64)?;
    println!("{:?}", threshold);

    let t_count = make_count_by(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    )?;
    let (dom, met) = t_count.output_space();
    let m_noise = make_laplace_threshold(dom, met, scale, threshold, None)?;
    let m_count = (t_count >> m_noise)?;
    let ret = m_count.invoke(&vec![
        'a', 'a', 'a', 'a', 'a', 'a', 'a', 'a', 'b', 'b', 'b', 'b', 'b', 'b', 'b',
    ])?;
    println!("stability eval: {:?}", ret);

    let epsilon_p = m_count.map(&max_influence)?.0;
    assert_eq!(epsilon_p, epsilon);
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CollidingKey(u32);

impl Hash for CollidingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0u8.hash(state);
    }
}

impl fmt::Display for CollidingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "K{}", self.0)
    }
}

impl CheckNull for CollidingKey {
    const NULLABLE: bool = false;
}

impl CheckAtom for CollidingKey {}

#[test]
fn test_noise_threshold_shuffles_colliding_keys() -> Fallible<()> {
    let measurement = make_noise_threshold(
        MapDomain::new(
            AtomDomain::<CollidingKey>::default(),
            AtomDomain::<IBig>::default(),
        ),
        L0PInfDistance::<1, _>(AbsoluteDistance::<RBig>::default()),
        Approximate(MaxDivergence),
        0.0,
        IBig::from(10),
        None,
    )?;

    let input: HashMap<_, _> = (1..=3)
        .map(|i| (CollidingKey(i), IBig::from(100)))
        .collect();
    let input_order: Vec<_> = input.keys().map(|key| key.0).collect();

    // With identical hashes, an unshuffled output retains input cyclic order,
    // even when the random hasher changes the starting bucket.
    let mut saw_non_rotation = false;
    for _ in 0..64 {
        let output = measurement.invoke(&input)?;
        assert_eq!(output, input);
        let output_order: Vec<_> = output.keys().map(|key| key.0).collect();
        let is_rotation = (0..input_order.len()).any(|offset| {
            output_order
                .iter()
                .enumerate()
                .all(|(i, key)| *key == input_order[(i + offset) % input_order.len()])
        });
        if !is_rotation {
            saw_non_rotation = true;
            break;
        }
    }

    assert!(
        saw_non_rotation,
        "output order retained the input cyclic order in all 64 trials"
    );
    Ok(())
}

#[test]
fn test_threshold_delta_exhaustive() -> Fallible<()> {
    // Discrete Laplace with exp(-1/scale) = 1/2.
    fn p(t: u32) -> f64 {
        2f64.powi(-(t as i32)) / 3.
    }
    fn exhaustive(n: u32, s: u32, b: u32, t: u32) -> f64 {
        if n == 0 || s == 0 {
            return 0.;
        }
        (0..=s.min(b))
            .map(|a| {
                let first = if a == 0 { 0. } else { p(t - a) };
                let rest = exhaustive(n - 1, s - a, b, t);
                first + (1. - first) * rest
            })
            .fold(0., f64::max)
    }
    for n in 0..=5 {
        for s in 0..=15 {
            for b in 0..=5 {
                let t = 7;
                let expected = exhaustive(n, s, b, t);
                let actual =
                    threshold_delta(n, UBig::from(s), UBig::from(b), UBig::from(t), |x| {
                        Ok(p(count(x)?))
                    })?;
                assert!(
                    (actual - expected).abs() < 1e-14,
                    "{n}, {s}, {b}: {actual} != {expected}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn test_threshold_delta_rare_events() -> Fallible<()> {
    let p = 1e-24;
    assert_eq!(repeated_probability(p, 1)?, p);
    let joint = repeated_probability(p, 100)?;
    assert!(joint >= 99.99 * p && joint <= 100.01 * p);
    Ok(())
}
