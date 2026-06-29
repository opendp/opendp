use super::*;

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
fn native_f32_certification_allows_infinite_output_cell() {
    assert!(matches!(
        certify_real_affine_rounds_to_f32(
            f64::MAX,
            1.0,
            None,
            false,
            0,
            &NativeUniform01 { prefix: 0, bits: 0 }
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
            &NativeUniform01 { prefix: 0, bits: 0 }
        ),
        F32CellCertification::Output(out) if out == 1.0
    ));
}

#[test]
fn native_fixed_scratch_affine_comparison() {
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.25, false, 0, 1, 1),
        Some(std::cmp::Ordering::Equal)
    );
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.0, false, 0, 1, 1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        compare_affine_to_boundary(1.0, 0.5, 1.5, false, 0, 1, 1),
        Some(std::cmp::Ordering::Less)
    );
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
