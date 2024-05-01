use dashu::float::round::mode::Down;

use crate::traits::samplers::psrn::test::test_progression;

use super::*;

#[test]
fn test_sample_gumbel_interval_progression() -> Fallible<()> {
    test_progression(&mut GumbelPSRN::new(RBig::ZERO, RBig::ONE), 100);
    Ok(())
}

#[test]
fn test_gumbel_psrn() -> Fallible<()> {
    fn sample_gumbel() -> Fallible<f64> {
        let mut gumbel = GumbelPSRN::new(RBig::ZERO, RBig::ONE);
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
