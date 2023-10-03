use crate::{error::Fallible, traits::samplers::psrn::test::test_progression};

use super::*;

#[test]
fn test_sample_exponential_interval_progression() -> Fallible<()> {
    let mut exp = ExponentialDist::new_psrn(FBig::ZERO, FBig::ONE);
    let (l, r) = test_progression(&mut exp, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", exp.refinements());
    Ok(())
}

#[test]
fn test_exponential_psrn() -> Fallible<()> {
    fn sample_exponential() -> Fallible<f64> {
        let mut exp = ExponentialDist::new_psrn(FBig::ZERO, FBig::ONE);
        // refine it
        (0..30).try_for_each(|_| exp.refine())?;

        Ok(exp.lower().unwrap().to_f64().value())
    }
    let samples = (0..1000)
        .map(|_| sample_exponential())
        .collect::<Fallible<Vec<_>>>()?;
    println!("{:?}", samples);
    Ok(())
}
