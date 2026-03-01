use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_noise_floatexpfamily() -> Fallible<()> {
    let space = (
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );

    assert!(
        FloatExpFamily::<1> { scale: 1.0, k: 0 }
            .make_noise_threshold(space.clone(), 10.0)
            .is_ok()
    );
    assert!(
        FloatExpFamily::<1> {
            scale: f64::NAN,
            k: 0
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_err()
    );
    let space = (
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );
    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MIN
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_err()
    );

    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MAX
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_ok()
    );

    Ok(())
}
