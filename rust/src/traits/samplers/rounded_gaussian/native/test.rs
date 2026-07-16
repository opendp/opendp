use super::*;

fn uniform(prefix: u128, bits: u32) -> NativeUniform01 {
    let mut out = NativeUniform01::new();
    out.set_prefix(prefix);
    out.set_bits(bits);
    out
}

fn factorial_f64(n: u32) -> f64 {
    (1..=n).map(f64::from).product()
}

fn sampler_failure_bound() -> f64 {
    let factories = NATIVE_SAMPLER_K_MAX * NATIVE_SAMPLER_K_MAX + NATIVE_SAMPLER_K_MAX + 2;
    let comparisons = 2 * u64::from(NATIVE_BERNOULLI_MAX_DEPTH) * factories;
    std::f64::consts::E
        * comparisons as f64
        * 2.0f64.powi(1 - NATIVE_SAMPLER_UNIFORM_MAX_BITS as i32)
        + std::f64::consts::E * factories as f64 / factorial_f64(NATIVE_BERNOULLI_MAX_DEPTH)
}

#[test]
fn native_uniform_packs_the_paper_profile_in_sixteen_bytes() {
    assert_eq!(std::mem::size_of::<NativeUniform01>(), 16);

    let prefix = (1u128 << NATIVE_SAMPLER_UNIFORM_MAX_BITS) - 1;
    let mut value = uniform(prefix, NATIVE_SAMPLER_UNIFORM_MAX_BITS);
    assert_eq!(value.prefix(), prefix);
    assert_eq!(value.bits(), 112);
    assert_eq!(value.denominator_bits(), 112);

    value.words[3] |= NATIVE_UNIFORM_SCALE_SHIFT_FLAG;
    assert_eq!(value.prefix(), prefix);
    assert_eq!(value.bits(), 112);
    assert_eq!(value.denominator_bits(), 113);
}

#[test]
fn native_uniform_less_than_half_decision() {
    let mut u = uniform(0b0001, 4);
    let mut x = uniform(0b1000, 4);
    assert_eq!(uniform_less_than_half_decided(&u, &x), Some(true));

    u.set_prefix(0b0101);
    x.set_prefix(0b0110);
    assert_eq!(uniform_less_than_half_decided(&u, &x), Some(false));

    u.set_prefix(0b0011);
    x.set_prefix(0b0111);
    assert_eq!(uniform_less_than_half_decided(&u, &x), None);
}

#[test]
fn native_uniform_interval_comparison_aligns_denominators() {
    let low = uniform(0b001, 3);
    let high = uniform(0b011, 3);
    let overlapping = uniform(0b0010, 4);

    assert_eq!(high.greater_than_decided(&low), Some(true));
    assert_eq!(low.greater_than_decided(&high), Some(false));
    assert_eq!(low.greater_than_decided(&overlapping), None);

    let high = uniform((1u128 << 112) - 1, 112);
    let low = uniform(0, 1);
    assert_eq!(high.greater_than_decided(&low), Some(true));
    assert_eq!(uniform_less_than_half_decided(&high, &high), Some(false));
}

#[test]
fn native_scale_snaps_and_decomposes_exactly() -> Fallible<()> {
    assert_eq!(snap_scale_up_to_f32(1.0)?, 1.0_f32);
    assert_eq!(
        snap_scale_up_to_f32(f64::from(f32::from_bits(1)) / 2.0)?,
        f32::from_bits(1)
    );

    let halfway = 1.0 + f64::from(f32::EPSILON) * 0.75;
    assert!(f64::from(snap_scale_up_to_f32(halfway)?) >= halfway);

    let one = positive_f32_scale_parts(1.0);
    assert_eq!(one.mantissa, 1 << 23);
    assert_eq!(one.exponent, -23);
    let min_subnormal = positive_f32_scale_parts(f32::from_bits(1));
    assert_eq!(min_subnormal.mantissa, 1);
    assert_eq!(min_subnormal.exponent, -149);
    Ok(())
}

#[test]
fn grid_index_rounds_to_nearest_ties_to_even() {
    let even = rounded_f64_grid_index(2.5, 0).unwrap();
    let odd = rounded_f64_grid_index(3.5, 0).unwrap();
    let negative = rounded_f64_grid_index(-2.5, 0).unwrap();

    assert_eq!(even, I160::new(false, U160::from_u128(2)).unwrap());
    assert_eq!(odd, I160::new(false, U160::from_u128(4)).unwrap());
    assert_eq!(negative, I160::new(true, U160::from_u128(2)).unwrap());
    assert_eq!(
        floor_positive_f64_grid_index(3.75, 0),
        Some(U160::from_u128(3))
    );
}

#[test]
fn negative_fraction_normalization_preserves_the_midpoint() {
    let base = I160::new(false, U160::from_u128(3)).unwrap();
    let (negative, bin) = normalize_grid_bin(base, true, 0).unwrap();
    assert!(!negative);
    assert_eq!(bin.integer, U160::from_u128(2));
    assert_eq!(bin.fraction_bin, u64::MAX);

    let base = I160::new(true, U160::from_u128(3)).unwrap();
    let (negative, bin) = normalize_grid_bin(base, false, u64::MAX).unwrap();
    assert!(negative);
    assert_eq!(bin.integer, U160::from_u128(2));
    assert_eq!(bin.fraction_bin, 0);
}

#[test]
fn direct_dyadic_converter_handles_ieee_edges() {
    let min_subnormal = round_dyadic_to_f32(false, U224([1, 0, 0, 0, 0, 0, 0]), -149);
    assert_eq!(min_subnormal, Some(f32::from_bits(1)));

    // Half of the minimum subnormal is a tie to even zero.
    let half_min_subnormal = round_dyadic_to_f32(false, U224([1, 0, 0, 0, 0, 0, 0]), -150);
    assert_eq!(half_min_subnormal, Some(0.0));

    let negative_zero = round_dyadic_to_f32(true, U224::ZERO, -150).unwrap();
    assert_eq!(negative_zero.to_bits(), 0.0f32.to_bits());

    let overflow = round_dyadic_to_f32(
        false,
        U224([u32::MAX, u32::MAX, u32::MAX, u32::MAX, 0, 0, 0]),
        32,
    );
    assert_eq!(overflow, Some(f32::INFINITY));
}

#[test]
fn direct_dyadic_converter_preserves_f32_values_and_ties() {
    for bits in [
        1,
        2,
        0x007f_ffff,
        0x0080_0000,
        1.0f32.to_bits(),
        f32::MAX.to_bits(),
    ] {
        let value = f32::from_bits(bits);
        let parts = positive_f32_scale_parts(value);
        let converted = round_dyadic_to_f32(
            false,
            U224([parts.mantissa, 0, 0, 0, 0, 0, 0]),
            parts.exponent,
        );
        assert_eq!(converted, Some(value));
    }

    // 1 + 2^-24 is halfway from 1.0 to its successor and rounds to even 1.0.
    assert_eq!(
        round_dyadic_to_f32(false, U224([(1 << 24) + 1, 0, 0, 0, 0, 0, 0]), -24),
        Some(1.0)
    );
    // The next midpoint rounds away from the odd low endpoint to the even one.
    assert_eq!(
        round_dyadic_to_f32(false, U224([(1 << 24) + 3, 0, 0, 0, 0, 0, 0]), -24),
        Some(f32::from_bits(1.0f32.to_bits() + 2))
    );
}

#[test]
fn native_profile_matches_the_paper_constants_and_admission_checks() -> Fallible<()> {
    assert_eq!(NATIVE_SAMPLER_K_MAX, (1 << 20) - 1);
    assert_eq!(NATIVE_BERNOULLI_MAX_DEPTH, 32);
    assert_eq!(NATIVE_SAMPLER_UNIFORM_MAX_BITS, 112);
    assert_eq!(NATIVE_COMPLETION_BITS, 64);
    assert!(sampler_failure_bound() < 2.0f64.powf(-63.5));

    let profile = NativeF32Profile::new(1.0, NATIVE_CLIP_SCALE_MAX_RATIO)?;
    assert_eq!(profile.snapped_scale, 1.0);
    assert_eq!(profile.grid_exponent, -135);
    assert!(NativeF32Profile::new(1.0, NATIVE_CLIP_SCALE_MAX_RATIO + 1.0).is_err());
    assert!(NativeF32Profile::new(2.0f64.powi(50), 0.0).is_err());
    Ok(())
}

#[test]
fn native_profile_zero_clip_has_deterministic_output() -> Fallible<()> {
    let profile = NativeF32Profile::new(1.0, 0.0)?;
    assert!(profile.sample(f64::NAN).is_err());
    assert!(matches!(
        profile.sample(f64::MAX)?,
        NativeF32Sample::Output(value) if value.to_bits() == 0.0f32.to_bits()
    ));
    Ok(())
}
