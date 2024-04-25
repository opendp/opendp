use super::*;

#[test]
fn test_sample_gumbel_interval_progression() -> Fallible<()> {
    let mut gumbel = GumbelPSRN::new(RBig::ZERO, RBig::ONE);
    for _ in 0..10 {
        println!(
            "{:?}, {:?}, {}",
            gumbel.value::<Down>()?.to_f64(),
            gumbel.value::<Up>()?.to_f64(),
            gumbel.precision
        );
        gumbel.refine()?;
    }
    Ok(())
}

#[test]
fn test_gumbel_psrn() -> Fallible<()> {
    fn sample_gumbel() -> Fallible<f64> {
        let mut gumbel = GumbelPSRN::new(RBig::ZERO, RBig::ONE);
        for _ in 0..10 {
            gumbel.refine()?;
        }
        Ok(gumbel.value::<Down>()?.to_f64().value())
    }
    let samples = (0..1000)
        .map(|_| sample_gumbel())
        .collect::<Fallible<Vec<_>>>()?;
    println!("{:?}", samples);
    Ok(())
}
