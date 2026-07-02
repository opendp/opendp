use super::*;

#[derive(Debug)]
struct NativeProfileAccounting {
    structural_tail: f64,
    factory_depth_failure: f64,
    comparison_failure: f64,
    sampler_overhead: f64,
    approximate_delta_overhead: f64,
    comb_overhead: f64,
    total_overhead: f64,
}

fn factorial_f64(n: u32) -> f64 {
    (1..=n).map(f64::from).product()
}

fn hybrid_native_profile_accounting(
    alpha: f64,
    dimension: usize,
    comb_probability_per_coordinate: f64,
) -> Fallible<NativeProfileAccounting> {
    if alpha <= 1.0 {
        return fallible!(FailedFunction, "alpha must be greater than one");
    }
    if comb_probability_per_coordinate < 0.0 || comb_probability_per_coordinate >= 1.0 {
        return fallible!(
            FailedFunction,
            "comb probability per coordinate must be in [0, 1)"
        );
    }

    let k_max = NATIVE_SAMPLER_K_MAX as f64;
    let b_s = f64::from(NATIVE_SAMPLER_UNIFORM_MAX_BITS);
    let l_max = NATIVE_BERNOULLI_MAX_DEPTH;
    let b_factories = NATIVE_SAMPLER_K_MAX * NATIVE_SAMPLER_K_MAX + NATIVE_SAMPLER_K_MAX + 2;
    let m_comparisons = 2 * u64::from(NATIVE_BERNOULLI_MAX_DEPTH) * b_factories;

    let structural_tail = (-0.5 * k_max * k_max).exp() / k_max;
    let factory_depth_failure = b_factories as f64 * std::f64::consts::E / factorial_f64(l_max + 1);
    let comparison_failure = m_comparisons as f64 * 4.0 * 2.0f64.powf(-b_s);
    let sampler_success_failure = comparison_failure;

    let comb_probability = dimension as f64 * comb_probability_per_coordinate;
    if structural_tail >= 1.0 || sampler_success_failure >= 1.0 || comb_probability >= 1.0 {
        return fallible!(
            FailedFunction,
            "finite-budget failure probabilities must remain below one"
        );
    }

    let structural_overhead = -(-structural_tail).ln_1p() / (alpha - 1.0);
    let sampler_success_overhead = -(-sampler_success_failure).ln_1p() * alpha / (alpha - 1.0);
    let sampler_overhead = structural_overhead + sampler_success_overhead;
    let approximate_delta_overhead = (1.0 + std::f64::consts::E) * factory_depth_failure;
    let comb_overhead = -(-comb_probability).ln_1p() * alpha / (alpha - 1.0);

    Ok(NativeProfileAccounting {
        structural_tail,
        factory_depth_failure,
        comparison_failure,
        sampler_overhead,
        approximate_delta_overhead,
        comb_overhead,
        total_overhead: sampler_overhead + comb_overhead,
    })
}

#[test]
fn native_uniform_less_than_half_decision() {
    let mut u = NativeUniform01 {
        prefix: 0b0001,
        bits: 4,
    };
    let mut x = NativeUniform01 {
        prefix: 0b1000,
        bits: 4,
    };
    assert_eq!(uniform_less_than_half_decided(&u, &x), Some(true));

    u.prefix = 0b0101;
    x.prefix = 0b0110;
    assert_eq!(uniform_less_than_half_decided(&u, &x), Some(false));

    u.prefix = 0b0011;
    x.prefix = 0b0111;
    assert_eq!(uniform_less_than_half_decided(&u, &x), None);
}

#[test]
fn native_uniform_integer_interval_comparison() {
    let low = NativeUniform01 {
        prefix: 0b001,
        bits: 3,
    };
    let high = NativeUniform01 {
        prefix: 0b011,
        bits: 3,
    };
    let overlapping = NativeUniform01 {
        prefix: 0b0010,
        bits: 4,
    };

    assert_eq!(high.greater_than_decided(&low), Some(true));
    assert_eq!(low.greater_than_decided(&high), Some(false));
    assert_eq!(low.greater_than_decided(&overlapping), None);
}

#[test]
fn native_uniform_comparisons_allow_full_128_bit_prefixes() {
    let high = NativeUniform01 {
        prefix: u128::MAX,
        bits: 128,
    };
    let low = NativeUniform01 { prefix: 0, bits: 1 };
    let overlapping_top = NativeUniform01 {
        prefix: u128::MAX >> 1,
        bits: 127,
    };

    assert_eq!(high.greater_than_decided(&low), Some(true));
    assert_eq!(low.greater_than_decided(&high), Some(false));
    assert_eq!(high.greater_than_decided(&overlapping_top), None);
    assert_eq!(uniform_less_than_half_decided(&high, &high), Some(false));
}

#[test]
fn native_f32_certification_allows_infinite_output_cell() {
    assert!(matches!(
        certify_real_affine_rounds_to_f32(
            f64::MAX,
            1.0,
            None,
            false,
            0,
            &NativeUniform01 { prefix: 0, bits: 0 },
            false
        ),
        F32CellCertification::Output(out) if out.is_infinite()
    ));
}

#[test]
fn native_f32_certification_clips_before_rounding() {
    assert!(matches!(
        certify_real_affine_rounds_to_f32(
            10.0,
            1.0,
            Some(1.0),
            false,
            0,
            &NativeUniform01 { prefix: 0, bits: 0 },
            false
        ),
        F32CellCertification::Output(out) if out == 1.0
    ));
}

#[test]
fn native_f32_cell_bounds_match_midpoints() {
    let (lower, upper) = f32_cell_bounds(1.0);
    assert_eq!(lower, Some((f64::from(1.0_f32.next_down_()) + 1.0) * 0.5));
    assert_eq!(upper, Some((1.0 + f64::from(1.0_f32.next_up_())) * 0.5));

    let (zero_lower, zero_upper) = f32_cell_bounds(0.0);
    assert_eq!(zero_lower, Some(f64::from(0.0_f32.next_down_()) * 0.5));
    assert_eq!(zero_upper, Some(f64::from(0.0_f32.next_up_()) * 0.5));
}

#[test]
fn native_clipped_endpoint_comparison_handles_clip_thresholds() {
    assert_eq!(
        compare_clipped_endpoint_to_boundary(10.0, 1.0, Some(1.0), false, 0, 0, 0, 1.0),
        Some(DyadicSign::Exact(std::cmp::Ordering::Equal))
    );
    assert_eq!(
        compare_clipped_endpoint_to_boundary(-10.0, 1.0, Some(1.0), false, 0, 0, 0, -1.0),
        Some(DyadicSign::Exact(std::cmp::Ordering::Equal))
    );
}

#[test]
fn native_f32_certification_marks_unresolved() {
    assert!(matches!(
        certify_real_affine_rounds_to_f32(
            0.0,
            1.0,
            None,
            false,
            0,
            &NativeUniform01 { prefix: 0, bits: 0 },
            false
        ),
        F32CellCertification::Unresolved
    ));
}

#[test]
fn native_fixed_scratch_affine_comparison() {
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.25, false, 0, 1, 1),
        Some(DyadicSign::Exact(std::cmp::Ordering::Equal))
    );
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.0, false, 0, 1, 1),
        Some(DyadicSign::Exact(std::cmp::Ordering::Greater))
    );
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.5, false, 0, 1, 1),
        Some(DyadicSign::Exact(std::cmp::Ordering::Less))
    );
}

#[test]
fn native_top_window_affine_comparison_certifies_near() {
    assert!(matches!(
        sign_dyadic_sum(&[
            Dyadic {
                negative: false,
                mantissa: 1,
                exponent: 1023,
            },
            Dyadic {
                negative: true,
                mantissa: 1,
                exponent: 1023,
            },
            Dyadic {
                negative: false,
                mantissa: 1,
                exponent: -1074,
            },
        ]),
        Some(DyadicSign::Near {
            units: 2,
            exponent: _
        })
    ));

    assert!(matches!(
        sign_dyadic_sum(&[
            Dyadic {
                negative: false,
                mantissa: 1,
                exponent: 1023,
            },
            Dyadic {
                negative: true,
                mantissa: 1,
                exponent: 1023,
            },
            Dyadic {
                negative: true,
                mantissa: 1,
                exponent: -1074,
            },
        ]),
        Some(DyadicSign::Near {
            units: 2,
            exponent: _
        })
    ));
}

#[test]
fn native_scale_snaps_up_to_f32() -> Fallible<()> {
    assert_eq!(snap_scale_up_to_f32(1.0)?, 1.0_f32);
    assert_eq!(
        snap_scale_up_to_f32(f64::from(f32::from_bits(1)) / 2.0)?,
        f32::from_bits(1)
    );

    let halfway = 1.0 + f64::from(f32::EPSILON) * 0.75;
    assert!(f64::from(snap_scale_up_to_f32(halfway)?) >= halfway);
    Ok(())
}

#[test]
fn hybrid_native_profile_accounting_stays_below_paper_floor() -> Fallible<()> {
    let terms = hybrid_native_profile_accounting(2.0, 1, 0.0)?;

    assert_eq!(NATIVE_SAMPLER_K_MAX, (1 << 20) - 1);
    assert_eq!(NATIVE_BERNOULLI_MAX_DEPTH, 40);
    assert_eq!(NATIVE_SAMPLER_UNIFORM_MAX_BITS, 128);
    assert_eq!(NATIVE_FINALIZATION_UNIFORM_MAX_BITS, 96);
    assert!(terms.structural_tail == 0.0);
    assert!(terms.factory_depth_failure < 2.0f64.powf(-123.0));
    assert!(terms.approximate_delta_overhead < 2.0f64.powf(-121.0));
    assert!(terms.comparison_failure < 2.0f64.powf(-79.6));
    assert!(terms.sampler_overhead < 2.0f64.powf(-78.6));
    assert_eq!(terms.comb_overhead, 0.0);
    assert_eq!(terms.total_overhead, terms.sampler_overhead);
    Ok(())
}
