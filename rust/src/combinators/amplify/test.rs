use crate::combinators::make_population_amplification;
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::measurements::then_laplace;
use crate::metrics::SymmetricDistance;
use crate::transformations::make_mean;

#[test]
fn test_amplifier() -> Fallible<()> {
    let meas = (make_mean(
        VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(10),
        SymmetricDistance::default(),
    ) >> then_laplace(0.5, None))?;
    let amp = make_population_amplification(&meas, 100)?;
    amp.function.eval(&vec![1.; 10])?;
    assert!(meas.check(&2, &(2. + 1e-6))?);
    assert!(!meas.check(&2, &2.)?);
    assert!(amp.check(&2, &0.4941)?);
    assert!(!amp.check(&2, &0.494)?);
    Ok(())
}
