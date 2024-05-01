use dashu::float::round::mode::Down;

use crate::{error::Fallible, traits::samplers::psrn::test::test_progression};

use super::*;

#[test]
fn test_sample_gumbel_interval_progression() -> Fallible<()> {
    let mut psrn = GumbelDist::new_psrn(RBig::ZERO, RBig::ONE);
    test_progression(&mut psrn, 100);
    Ok(())
}

#[test]
fn test_gumbel_psrn() -> Fallible<()> {
    fn sample_gumbel() -> Fallible<f64> {
        let mut gumbel = GumbelDist::new_psrn(RBig::ZERO, RBig::ONE);
        for _ in 0..10 {
            gumbel.refine()?;
        }
        Ok(gumbel
            .edge::<Down>()
            .map(|v| v.to_f64().value())
            .unwrap_or(f64::NAN))
    }
    let samples = (0..1000)
        .map(|_| sample_gumbel())
        .collect::<Fallible<Vec<_>>>()?;
    println!("{:?}", samples);
    Ok(())
}
