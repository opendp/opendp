use super::*;

#[test]
fn native_f64_to_f32_rounded_gaussian_smoke() -> Fallible<()> {
    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native(1.25, 0.0)?,
        1.25_f32
    );
    assert!(sample_rounded_gaussian_f64_to_f32_native(0.0, 1.0).is_err());

    for _ in 0..1_000 {
        assert!(
            sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 1.0, Some(1.0))?.is_finite()
        );
    }
    Ok(())
}

#[test]
fn native_f64_to_f32_clipped_rounded_gaussian_clips_input_and_output() -> Fallible<()> {
    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native_clipped(2.0, 0.0, Some(1.0))?,
        1.0_f32
    );
    assert!(sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 1.0, Some(524_288.0)).is_err());

    for _ in 0..100 {
        let out = sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 100.0, Some(1.0))?;
        assert!((-1.0..=1.0).contains(&out));
    }

    for _ in 0..10 {
        let out = sample_rounded_gaussian_f64_to_f32_native_clipped(f64::MAX, 1.0, Some(1.0))?;
        assert!((-1.0..=1.0).contains(&out));
    }

    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native_clipped(
            1.0,
            f64::from(f32::from_bits(1)) * 0.5,
            Some(0.0),
        )?,
        0.0
    );

    Ok(())
}

#[test]
fn exact_clipped_rounded_gaussian_clips_input_and_output() -> Fallible<()> {
    assert_eq!(sample_rounded_gaussian_clipped(2.0_f64, 0.0, 1.0)?, 1.0);

    for _ in 0..100 {
        let out = sample_rounded_gaussian_clipped(0.0_f32, 100.0, 1.0)?;
        assert!((-1.0..=1.0).contains(&out));
    }

    Ok(())
}
