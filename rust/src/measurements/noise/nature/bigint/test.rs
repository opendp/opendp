use dashu::{ibig, rbig};

use super::*;

#[test]
fn test_make_noise_atomdomain_ibig() -> Fallible<()> {
    let distribution = ZExpFamily::<2> { scale: rbig!(1) };
    let meas =
        distribution.make_noise((AtomDomain::<IBig>::default(), AbsoluteDistance::default()))?;
    assert!(i8::try_from(meas.invoke(&ibig!(0))?).is_ok());
    assert_eq!(meas.map(&rbig!(0))?, 0.0);
    assert_eq!(meas.map(&rbig!(1))?, 0.5);
    assert!(meas.map(&rbig!(-1)).is_err());
    Ok(())
}
