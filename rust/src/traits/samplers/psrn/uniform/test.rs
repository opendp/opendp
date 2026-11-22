use super::*;
use dashu::integer::UBig;

fn uniform(prefix: UBig, precision: usize) -> PartialUniform01 {
    PartialUniform01 {
        randomness: prefix,
        refinements: precision,
        distribution: Uniform01RV,
    }
}

// Compare the integer certificate against the original rational certificate
// on all short prefixes, unequal precisions, and both Gaussian predicates.
#[test]
fn integer_comparison_matches_rational_endpoints() -> Fallible<()> {
    for p in 0..=5 {
        for q in 0..=5 {
            for a in 0u64..1 << p {
                for b in 0u64..1 << q {
                    for shift in 0..=1 {
                        check_disjoint(a.into(), p, b.into(), q, shift)?;
                    }
                }
            }
        }
    }
    for p in [64, 65, 128, 1074] {
        for q in [64, 65, 128, 1074] {
            for a in [UBig::ZERO, UBig::ONE << (p - 1), (UBig::ONE << p) - 1u8] {
                for b in [UBig::ZERO, UBig::ONE << (q - 1), (UBig::ONE << q) - 1u8] {
                    for shift in 0..=1 {
                        check_disjoint(a.clone(), p, b.clone(), q, shift)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_disjoint(a: UBig, p: usize, b: UBig, q: usize, shift: usize) -> Fallible<()> {
    let mut a = uniform(a, p);
    let mut b = uniform(b, q);
    let factor = RBig::from(UBig::ONE << shift);
    let expected = if a.upper().unwrap() * &factor < b.lower().unwrap() {
        true
    } else if b.upper().unwrap() < a.lower().unwrap() * &factor {
        false
    } else {
        // Overlapping prefixes do not yet certify an answer.
        return Ok(());
    };
    assert_eq!(a.less_than_scaled_uniform(&mut b, shift)?, expected);
    assert_eq!((a.refinements, b.refinements), (p, q));
    Ok(())
}

#[test]
fn overlapping_prefixes_refine_past_machine_precision() -> Fallible<()> {
    for shift in 0..=1 {
        // a and b/2^shift start at the same lower endpoint.
        let mut a = uniform(UBig::ONE, 128 + shift);
        let mut b = uniform(UBig::ONE, 128);
        let result = a.less_than_scaled_uniform(&mut b, shift)?;
        let factor = RBig::from(UBig::ONE << shift);
        assert!(a.refinements > 128 + shift || b.refinements > 128);
        if result {
            assert!(a.upper().unwrap() * factor < b.lower().unwrap());
        } else {
            assert!(b.upper().unwrap() < a.lower().unwrap() * factor);
        }
    }
    Ok(())
}

#[test]
fn touching_endpoints_are_refined() -> Fallible<()> {
    let mut a = uniform(0u8.into(), 1);
    let mut b = uniform(1u8.into(), 1);
    assert!(a.less_than_scaled_uniform(&mut b, 0)?);
    assert!(a.refinements > 1 || b.refinements > 1);
    Ok(())
}
