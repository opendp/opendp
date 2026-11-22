use super::*;

#[test]
fn invalid_and_deterministic_bounds_do_not_read_entropy() -> Fallible<()> {
    let mut no_entropy = |_: &mut [u8]| -> Fallible<()> { panic!("unexpected entropy request") };
    assert!(sample_uniform_ubig_below_with(UBig::ZERO, &mut no_entropy).is_err());
    assert_eq!(
        sample_uniform_ubig_below_with(UBig::ONE, &mut no_entropy)?,
        UBig::ZERO
    );
    assert!(sample_uniform_uint_below_with::<u64, 8>(0, &mut no_entropy).is_err());
    assert_eq!(
        sample_uniform_uint_below_with::<u64, 8>(1, &mut no_entropy)?,
        0
    );
    Ok(())
}

#[test]
fn masked_uniform_candidates_are_exhaustively_unbiased() -> Fallible<()> {
    for upper in [2u16, 3, 127, 128, 129, 255, 256, 257] {
        let bits = (upper - 1).ilog2() as usize + 1;
        let mut counts = vec![0usize; upper as usize];
        for raw in 0..1u16 << bits {
            let mut calls = 0;
            let value = sample_uniform_ubig_below_with(UBig::from(upper), &mut |out| {
                let value = if calls == 0 { raw } else { 0 };
                calls += 1;
                let bytes = value.to_be_bytes();
                let len = out.len();
                out.copy_from_slice(&bytes[2 - len..]);
                if bits % 8 != 0 {
                    out[0] |= !((1u8 << (bits % 8)) - 1);
                }
                Ok(())
            })?;
            if raw < upper {
                assert_eq!(value, UBig::from(raw));
                assert_eq!(calls, 1);
                counts[raw as usize] += 1;
            } else {
                assert_eq!(value, UBig::ZERO);
                assert_eq!(calls, 2);
            }
        }
        assert!(counts.iter().all(|&n| n == 1));
    }
    Ok(())
}

#[test]
fn wide_bounds_across_byte_boundaries_mask_padding_correctly() -> Fallible<()> {
    for bits in [7usize, 8, 9, 63, 64, 65, 1074] {
        let power = UBig::ONE << bits;
        for upper in [&power - 1u8, power.clone(), &power + 1u8] {
            let draw_bits = (&upper - 1u8).bit_len();
            let max = (UBig::ONE << draw_bits) - 1u8;
            let last = &upper - 1u8;
            let mut calls = 0;
            let value = sample_uniform_ubig_below_with(upper.clone(), &mut |out| {
                assert!(calls < 2);
                let bytes = if calls == 0 {
                    max.to_be_bytes()
                } else {
                    last.to_be_bytes()
                };
                calls += 1;
                out.fill(0);
                let offset = out.len() - bytes.len();
                out[offset..].copy_from_slice(&bytes);
                if draw_bits % 8 != 0 {
                    out[0] |= !((1u8 << (draw_bits % 8)) - 1);
                }
                Ok(())
            })?;
            if max < upper {
                assert_eq!(value, max);
                assert_eq!(calls, 1);
            } else {
                assert_eq!(value, last);
                assert_eq!(calls, 2);
            }
        }
    }
    Ok(())
}
