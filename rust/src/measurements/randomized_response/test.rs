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
    let m_rr = make_randomized_response(HashSet::from([2, 3, 5, 6]), 0.625)?;
    assert!(m_rr.invoke(&3).is_ok());

    // The uniform-mixture probability is exactly 1/2, so the ratio is 5.
    let epsilon = m_rr.map(&1)?;
    assert!((5f64.inf_ln()?..=5f64.ln() + 1e-14).contains(&epsilon));
    Ok(())
}

#[test]
fn test_cat_extremes() -> Fallible<()> {
    let categories = HashSet::from([0, 1, 2]);
    // The uniform boundary must remain exactly 0-DP for non-power-of-two k.
    let m_rr = make_randomized_response(categories.clone(), 1.0 / 3.0)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    // Rounding toward uniformity can give zero privacy loss just above the boundary.
    let p = f64::from_bits((1.0f64 / 3.0).to_bits() + 1);
    let m_rr = make_randomized_response(categories.clone(), p)?;
    assert!((0.0..=1e-14).contains(&m_rr.map(&1)?));

    // A near-certain honesty probability still has finite privacy loss.
    let p = f64::from_bits(1.0f64.to_bits() - 1);
    let m_rr = make_randomized_response(categories.clone(), p)?;
    assert!(m_rr.map(&1)?.is_finite());

    let m_rr = make_randomized_response(categories.clone(), 1.)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    assert_eq!(m_rr.invoke(&2)?, 2);
    assert!(categories.contains(&m_rr.invoke(&3)?));
    Ok(())
}
