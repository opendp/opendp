use crate::traits::samplers::psrn::test::test_progression;

use super::*;

#[test]
fn test_sample_laplace_interval_progression() -> Fallible<()> {
    let mut laplace = LaplacePSRN::new(FBig::ZERO, FBig::ONE)?;
    let (l, r) = test_progression(&mut laplace, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", laplace.refinements());
    Ok(())
}

#[test]
fn test_laplace_psrn() -> Fallible<()> {
    fn sample_laplace() -> Fallible<f64> {
        let mut laplace = LaplacePSRN::new(FBig::ZERO, FBig::ONE)?;
        // refine it
        (0..30).try_for_each(|_| laplace.refine())?;

        Ok(laplace.lower().unwrap().to_f64().value())
    }
    let samples = (0..1000)
        .map(|_| sample_laplace())
        .collect::<Fallible<Vec<_>>>()?;
    println!("{:?}", samples);
    Ok(())
}
