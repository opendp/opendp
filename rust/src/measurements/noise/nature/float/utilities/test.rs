use core::f64;

use dashu::{ibig, rbig, ubig};

use super::*;

#[test]
fn test_find_nearest_multiple_of_2k_cases() -> Fallible<()> {
    macro_rules! min_x_k_i {
        ($x:literal, $k:literal, $i:literal) => {
            assert_eq!(
                find_nearest_multiple_of_2k(RBig::try_from($x)?, $k),
                IBig::from($i)
            );
        };
    }

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

#[test]
fn test_find_nearest_multiple_of_2k_bit_manip() -> Fallible<()> {
    let k = 0;
    let x = RBig::try_from(0.5)?;
    let i = find_nearest_multiple_of_2k(x.clone(), k);

    let x_p = RBig::try_from(f64::from_bits((0.5f64).to_bits() - 1))?;
    let i_p = find_nearest_multiple_of_2k(x_p, k);
    assert_eq!(&i - 1, i_p);

    let x_p = RBig::try_from(f64::from_bits((0.5f64).to_bits() + 1))?;
    let i_p = find_nearest_multiple_of_2k(x_p.clone(), k);
    assert_eq!(i, i_p);
    Ok(())
}

#[test]
fn test_floor_div() -> Fallible<()> {
    // negative integer division should round towards zero
    assert_eq!(ibig!(-3) / ibig!(2), ibig!(-1));
    // negative shift division should truncate
    // this assumption is used in the implementation of find_nearest_multiple_of_2k
    assert_eq!(ibig!(-3) >> 1usize, ibig!(-2));

    // floor div implements the equivalent for arbitrary denominator
    assert_eq!(floor_div(ibig!(-3), ubig!(2)), ibig!(-2));

    // positive integer division should round towards zero
    assert_eq!(floor_div(ibig!(3), ubig!(2)), ibig!(1));
    Ok(())
}

#[test]
fn test_get_min_k() -> Fallible<()> {
    // check that the smallest positive subnormals are 2 to the expected power
    assert_eq!(get_min_k::<f64>(), -1074);
    assert_eq!(RBig::try_from(f64::from_bits(1))?, rbig!(-1 / 2).pow(1074));

    assert_eq!(get_min_k::<f32>(), -149);
    assert_eq!(RBig::try_from(f32::from_bits(1))?, -rbig!(-1 / 2).pow(149));
    Ok(())
}

#[test]
fn test_get_rounding_distance() -> Fallible<()> {
    assert_eq!(
        get_rounding_distance::<f64, 1>(0, Some(1))?,
        rbig!(1) - RBig::try_from(f64::from_bits(1))?
    );

    assert_eq!(
        get_rounding_distance::<f64, 1>(0, Some(2))?,
        (rbig!(1) - RBig::try_from(f64::from_bits(1))?) * ubig!(2)
    );

    assert_eq!(
        get_rounding_distance::<f64, 1>(1, Some(1))?,
        rbig!(2) - RBig::try_from(f64::from_bits(1))?
    );

    assert_eq!(
        get_rounding_distance::<f64, 1>(get_min_k::<f64>(), Some(1))?,
        RBig::ZERO
    );

    assert_eq!(
        get_rounding_distance::<f64, 2>(0, Some(3))?,
        (rbig!(1) - RBig::try_from(f64::from_bits(1))?) * RBig::try_from((3.0).inf_sqrt()?)?
    );
    Ok(())
}

#[test]
fn test_x_mul_2k() -> Fallible<()> {
    assert_eq!(x_mul_2k(RBig::try_from(1.)?, 0), RBig::ONE); // 1 * 2^0 = 1
    assert_eq!(x_mul_2k(RBig::try_from(0.25)?, 2), RBig::ONE); // 0.25 * 2^2 = 1
    assert_eq!(x_mul_2k(RBig::try_from(1.)?, -2), RBig::try_from(0.25)?); // 1 * 2^-2 = 0.25

    // these work but take ~.5 second and 1/4 GB of memory. Unsurprising
    // x_mul_2k(rbig!(1), -(i32::MIN + 1));
    // x_mul_2k(rbig!(1), i32::MAX);
    Ok(())
}
