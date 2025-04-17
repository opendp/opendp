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
        FloatExpFamily::<1, _> {
            scale: 1.0,
            k: 0,
            radius: None
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_ok()
    );
    assert!(
        FloatExpFamily::<1, _> {
            scale: 1.0,
            k: 0,
            radius: None
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
        FloatExpFamily::<2, _> {
            scale: 1.0,
            k: i32::MIN,
            radius: None
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_err()
    );

    assert!(
        FloatExpFamily::<2, _> {
            scale: 1.0,
            k: i32::MAX,
            radius: None
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_ok()
    );

    Ok(())
}
