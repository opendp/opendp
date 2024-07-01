use super::*;
use crate::traits::samplers::SampleUniform;

#[test]
fn roundtrip_sign_exponent_mantissa() -> Fallible<()> {
    for _ in 0..1000 {
        let unif = f64::sample_standard_uniform(false)?;
        println!("{:?}", unif);
        let (sign, raw_exponent, mantissa) = unif.to_raw_components();
        let reconst = f64::from_raw_components(sign, raw_exponent, mantissa);
        assert_eq!(unif, reconst);
    }
    Ok(())
}
