use core::f64;

use super::*;
use crate::{
    domains::{AtomDomain, VectorDomain},
    metrics::{AbsoluteDistance, L1Distance},
};
use num::{One, Zero};

#[test]
fn test_make_geometric_native_types() -> Fallible<()> {
    macro_rules! test_make_geometric_type {
        ($($ty:ty),+) => {$(
            // scalar
            let meas = make_geometric(
                AtomDomain::<$ty>::new_non_nan(),
                AbsoluteDistance::<$ty>::default(),
                1., Some((0, 4))
            )?;
            let r = meas.invoke(&<$ty>::zero())?;
            assert!((0..=4).contains(&r), "sampled value out of bounds: {}", r);
            assert_eq!(meas.map(&<$ty>::one())?, 1.0);
            // vector
            let meas = make_geometric(
                VectorDomain::new(AtomDomain::<$ty>::new_non_nan()),
                L1Distance::<$ty>::default(),
                1., Some((0, 4))
            )?;
            meas.invoke(&vec![<$ty>::zero(); 100])?.into_iter().for_each(|r| {
                assert!((0..=4).contains(&r), "sampled value out of bounds: {}", r);
            });
            assert_eq!(meas.map(&<$ty>::one())?, 1.0);
        )+}
    }

    test_make_geometric_type!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);
    Ok(())
}

#[test]
fn test_make_geometric_extreme_int() -> Fallible<()> {
    // p too big
    assert!(
        make_geometric(
            AtomDomain::<u32>::default(),
            AbsoluteDistance::<u32>::default(),
            f64::MAX,
            Some((0, 200)),
        )
        .is_err()
    );

    // an extreme noise scale dominates the output, resulting in the release always being saturated
    let meas = make_geometric(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        100_000.0,
        Some((0, 200)),
    )?;
    assert!([0, 200].contains(&meas.invoke(&0)?));
    Ok(())
}

#[test]
fn test_make_noise_zexpfamily1_zero_scale() -> Fallible<()> {
    let domain = VectorDomain::<AtomDomain<u8>>::default();
    let metric = L1Distance::default();
    let distribution = ConstantTimeGeometric {
        scale: 0.0,
        bounds: (0, u8::MAX),
    };

    let meas = distribution.make_noise((domain, metric))?;
    assert_eq!(meas.invoke(&vec![0])?, vec![0]);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}
