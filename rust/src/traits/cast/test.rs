use crate::traits::InfCast;

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
