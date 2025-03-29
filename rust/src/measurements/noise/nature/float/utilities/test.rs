use dashu::{ibig, integer::IBig, rational::RBig, ubig};

use crate::{
    error::Fallible,
    measurements::nature::float::{find_nearest_multiple_of_2k, utilities::floor_div, x_mul_2k},
};

#[test]
fn test_extreme_rational() -> Fallible<()> {
    // rationals with greater magnitude than MAX saturate to infinity
    let rat = RBig::try_from(f64::MAX).unwrap();
    assert!((rat * IBig::from(2u8)).to_f64().value().is_infinite());

    Ok(())
}

#[test]
fn test_shr() -> Fallible<()> {
    assert_eq!(x_mul_2k(RBig::try_from(1.)?, 0), RBig::ONE);
    assert_eq!(x_mul_2k(RBig::try_from(0.25)?, 2), RBig::ONE);
    assert_eq!(x_mul_2k(RBig::try_from(1.)?, -2), RBig::try_from(0.25)?);
    Ok(())
}

macro_rules! min_x_k_i {
    ($x:literal, $k:literal, $i:literal) => {
        assert_eq!(
            find_nearest_multiple_of_2k(RBig::try_from($x)?, $k),
            IBig::from($i)
        );
    };
}

#[test]
fn test_bigint_div_rounds_to_zero() -> Fallible<()> {
    // negative integer division should round towards zero
    assert_eq!(ibig!(-3) / ibig!(2), ibig!(-1));
    // negative shift division should truncate
    // this assumption is used in the implementation of find_nearest_multiple_of_2k
    assert_eq!(ibig!(-3) >> 1usize, ibig!(-2));

    // floor div implements the equivalent for arbitrary denominator
    assert_eq!(floor_div(ibig!(-3), ubig!(2)), ibig!(-2));
    Ok(())
}

#[test]
fn test_find_nearest_multiple_of_2k() -> Fallible<()> {
    // x=0
    min_x_k_i!(0.0, 0, 0); // 0 * 2^0 is nearest to 0   (exact)
    min_x_k_i!(0.0, 1, 0); // 0 * 2^1 is nearest to 0   (exact)
    min_x_k_i!(0.0, -1, 0); // 0 * 2^-1 is nearest to 0 (exact)
    // x=1
    min_x_k_i!(1.0, 0, 1); // 1 * 2^0 is nearest to 1   (exact)
    min_x_k_i!(1.0, 1, 1); // 1 * 2^1 is nearest to 1   (tied and rounded up)
    min_x_k_i!(1.0, -1, 2); // 2 * 2^-1 is nearest to 1 (exact)
    // x=-1
    min_x_k_i!(-1.0, 0, -1); // -1 * 2^0 is nearest to -1   (exact)
    min_x_k_i!(-1.0, 1, 0); // 0 * 2^1 is nearest to -1     (tied and rounded up)
    min_x_k_i!(-1.0, -1, -2); // -2 * 2^-1 is nearest to -1 (exact)
    // x=2
    min_x_k_i!(2.0, 0, 2); // 2 * 2^0 is nearest to 2   (exact)
    min_x_k_i!(2.0, 1, 1); // 1 * 2^1 is nearest to 2   (exact)
    min_x_k_i!(2.0, -1, 4); // 4 * 2^-1 is nearest to 2 (exact)
    // x=-2
    min_x_k_i!(-2.0, 0, -2); // -2 * 2^0 is nearest to -2   (exact)
    min_x_k_i!(-2.0, 1, -1); // -1 * 2^1 is nearest to -2   (exact)
    min_x_k_i!(-2.0, -1, -4); // -4 * 2^-1 is nearest to -2 (exact)

    // edge cases
    // negative half-boundary
    min_x_k_i!(-1.5, 0, -1); // -1 * 2^0 is nearest to -1.5 (tied and rounded up)
    // negative half-boundary at zero
    min_x_k_i!(-0.5, 0, 0); // 0 * 2^0 is nearest to -0.5   (tied and rounded up)
    // positive half-boundary at zero
    min_x_k_i!(0.5, 0, 1); // 1 * 2^0 is nearest to 0.5     (tied and rounded up)
    // positive half-boundary
    min_x_k_i!(1.5, 0, 2); // 2 * 2^0 is nearest to 1.5     (tied and rounded up)

    // negative round down
    min_x_k_i!(-1.75, 0, -2); // -2 * 2^0 is nearest to -1.75   (rounded down)
    // negative round up
    min_x_k_i!(-1.25, 0, -1); // 2 * 2^0 is nearest to 1.75   (rounded up)

    // positive round down
    min_x_k_i!(1.25, 0, 1); // 1 * 2^0 is nearest to 1.25   (rounded down)
    // positive round up
    min_x_k_i!(1.75, 0, 2); // 2 * 2^0 is nearest to 1.75   (rounded up)

    Ok(())
}
