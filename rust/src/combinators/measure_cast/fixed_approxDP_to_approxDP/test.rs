use crate::{
    domains::{AtomDomain, MapDomain},
    measurements::make_laplace_threshold,
    metrics::L0PInfDistance,
    traits::NextFloat,
};

use super::*;

#[test]
fn test_fixed_approxDP_to_approxDP() -> Fallible<()> {
    let meas_fixed = make_laplace_threshold(
        MapDomain::new(AtomDomain::<String>::default(), AtomDomain::new_non_nan()),
        L0PInfDistance::default(),
        1.,
        10,
        None,
    )?;
    let meas_smooth = make_fixed_approxDP_to_approxDP(meas_fixed.clone())?;

    let (eps, del) = meas_fixed.map(&(1, 1, 1))?;

    let profile = meas_smooth.map(&(1, 1, 1))?;

    assert_eq!(profile.delta(0.)?, 1.0);
    assert_eq!(profile.delta(eps.next_down_())?, 1.0);
    assert_eq!(profile.delta(eps)?, del);
    assert_eq!(profile.delta(eps.next_up_())?, del);

    Ok(())
}
