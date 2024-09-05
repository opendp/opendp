use dashu::float::{
    round::mode::{Down, Up},
    FBig,
};

use crate::{
    error::Fallible,
    traits::{cast::ToFloatRounded, InfCast},
};

#[allow(dead_code)]
enum Diff {
    Equal,
    Prev,
    Next,
    Less,
    Greater,
}

fn check_rounded_cast(input: f64, diff: Diff) {
    let casted = f32::inf_cast(input).unwrap() as f64;
    if input.is_nan() {
        assert!(casted.is_nan());
        return;
    }

    let error = match diff {
        Diff::Equal => (casted != input).then(|| "casted value must be equal to input"),
        Diff::Greater => (casted <= input).then(|| "casted value must be greater than input value"),
        Diff::Less => (casted >= input).then(|| "casted value must be less than input value"),
        Diff::Next => (f64::from_bits(input.to_bits() + 1) != casted)
            .then(|| "casted must be one step greater than input"),
        Diff::Prev => (f64::from_bits(input.to_bits() - 1) != casted)
            .then(|| "casted must be one step less than input"),
    };
    if let Some(message) = error {
        println!("bits      {:064b}", input.to_bits());
        println!("input     {}", input);
        println!("output    {}", casted);
        panic!("{}", message)
    }
}

#[test]
// ignored test because it can take a while to run
#[ignore]
fn test_f64_f32() {
    check_rounded_cast(0., Diff::Equal);
    // check that the f64 one step above zero casts to a value that is greater
    check_rounded_cast(f64::MIN_POSITIVE, Diff::Greater);
    // check that the f64 one step below 2 casts to exactly 2
    check_rounded_cast(1.9999999999999998, Diff::Next);
    // for each non-negative, nonzero f32
    for u32_bits in 1..u32::MAX / 2 {
        let f64_value = f32::from_bits(u32_bits) as f64;
        let u64_bits = f64_value.to_bits();

        if u32_bits % 100_000_000 == 0 {
            println!("checkpoint every 300 million tests: {}", f64_value);
        }
        // check that the f64 equivalent to the current f32 casts to a value that is equivalent
        check_rounded_cast(f64_value, Diff::Equal);
        // check that the f64 one step below the f64 equivalent to the current f32 casts to a value that is one step greater
        check_rounded_cast(f64::from_bits(u64_bits - 1), Diff::Next);
        // check that the f64 one step above the f64 equivalent to the current f32 casts to a value that is greater
        check_rounded_cast(f64::from_bits(u64_bits + 1), Diff::Greater);
    }
}

#[test]
fn test_subnormal_fail() -> Fallible<()> {
    let min = FBig::<Up>::try_from(f32::from_bits(1))?;
    let half: FBig<Up> = min / 2;
    // this behavior is wrong. The conversion should be conducted with rounding up, but is not
    // If this test fails, then check to see if the bug has been fixed in dashu:
    // https://github.com/cmpute/dashu/issues/53

    // When Dashu is fixed, you should be able to replace `to_fxx_rounded()` with `to_fxx().value()`.
    // This PR introduced these changes:
    // https://github.com/opendp/opendp/pull/1998
    assert_eq!(half.to_f32().value(), 0.);

    // valid subnormals still convert exactly, as expected
    let min = FBig::<Up>::try_from(f32::from_bits(1))?;
    assert_eq!(min.to_f32().value(), f32::from_bits(1));
    Ok(())
}

#[test]
fn test_to_native_subnormal() -> Fallible<()> {
    // a number smaller than subnormal should convert to the smallest subnormal when converting to float

    // f32 positive
    let min = FBig::<Up>::try_from(f32::from_bits(1))?;
    let half: FBig<Up> = min / 2;
    assert_eq!(half.to_f32_rounded(), f32::from_bits(1));

    // f32 negative
    let min = -FBig::<Down>::try_from(f32::from_bits(1))?;
    let half: FBig<Down> = min / 2;
    assert_eq!(half.to_f32_rounded(), -f32::from_bits(1));

    // f64 positive
    let min = FBig::<Up>::try_from(f64::from_bits(1))?;
    let half: FBig<Up> = min / 2;
    assert_eq!(half.to_f64_rounded(), f64::from_bits(1));

    // f64 negative
    let min = -FBig::<Down>::try_from(f64::from_bits(1))?;
    let half: FBig<Down> = min / 2;
    assert_eq!(half.to_f64_rounded(), -f64::from_bits(1));

    Ok(())
}

#[test]
fn test_to_native_zero() -> Fallible<()> {
    // rounding should not take next up or down
    assert_eq!(FBig::<Up>::try_from(0f32)?.to_f32_rounded(), 0f32);
    assert_eq!(FBig::<Up>::try_from(0f64)?.to_f64_rounded(), 0f64);
    Ok(())
}

#[test]
fn test_to_native_inf() -> Fallible<()> {
    // a number much larger than max should convert to max, not inf, when converting with rounding down

    // f32 positive
    let max = FBig::<Down>::try_from(f32::MAX)?;
    let double: FBig<Down> = max * 2;
    assert_eq!(double.to_f32_rounded(), f32::MAX);

    // f32 negative
    let min = FBig::<Up>::try_from(f32::MIN)?;
    let double: FBig<Up> = min * 2;
    assert_eq!(double.to_f32_rounded(), f32::MIN);

    // f64 positive
    let max = FBig::<Down>::try_from(f64::MAX)?;
    let double: FBig<Down> = max * 2;
    assert_eq!(double.to_f64_rounded(), f64::MAX);

    // f64 negative
    let min = FBig::<Up>::try_from(f64::MIN)?;
    let double: FBig<Up> = min * 2;
    assert_eq!(double.to_f64_rounded(), f64::MIN);

    Ok(())
}
