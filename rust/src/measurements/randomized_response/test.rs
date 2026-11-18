use super::*;
use num::Float as _;

#[test]
fn test_bool() -> Fallible<()> {
    let m_rr = make_randomized_response_bool(0.75)?;
    assert!(m_rr.invoke(&false).is_ok());
    assert_eq!(m_rr.map(&1)?, 3.0.ln());
    Ok(())
}

#[test]
fn test_bool_extremes() -> Fallible<()> {
    // 50% chance that the output is correct means all information is lost, query is ε=0 dp
    let m_rr = make_randomized_response_bool(0.5)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    // 100% chance that the output is correct is ε=inf dp
    let m_rr = make_randomized_response_bool(1.0)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_cat() -> Fallible<()> {
    let m_rr = make_randomized_response(HashSet::from([2, 3, 5, 6]), 0.75)?;
    assert!(m_rr.invoke(&3).is_ok());

    // (.75 * 3 / .25) = 9
    assert_eq!(m_rr.map(&1)?, 9f64.ln());
    Ok(())
}

#[test]
fn test_cat_quantized_odds() -> Fallible<()> {
    // The uniform boundary must remain exactly 0-DP for non-power-of-two k.
    let m_rr = make_randomized_response(HashSet::from([0, 1, 2]), 1.0 / 3.0)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    // The first representable probability above the boundary requires
    // the privacy map to account for upward quantization.
    let p = f64::from_bits((1.0f64 / 3.0).to_bits() + 1);
    let m_rr = make_randomized_response(HashSet::from([0, 1, 2]), p)?;
    assert!(m_rr.map(&1)? > 0.0);

    // Avoid rounding a near-certain honesty probability to 1.0.
    let p = f64::from_bits(1.0f64.to_bits() - 1);
    let m_rr = make_randomized_response(HashSet::from([0, 1, 2]), p)?;
    assert!(m_rr.map(&1)?.is_finite());
    Ok(())
}

#[test]
fn test_cat_extremes() -> Fallible<()> {
    let categories = HashSet::from([2, 3, 5, 7, 8]);
    let m_rr = make_randomized_response(categories.clone(), 1. / 5.)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    let m_rr = make_randomized_response(categories, 1.)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    Ok(())
}
