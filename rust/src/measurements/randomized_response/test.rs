use super::*;
use num::Float as _;

#[test]
fn test_bool() -> Fallible<()> {
    let m_rr = make_randomized_response_bool(0.75, false)?;
    assert!(m_rr.invoke(&false).is_ok());
    assert_eq!(m_rr.map(&1)?, 3.0.ln());
    Ok(())
}

#[test]
fn test_bool_extremes() -> Fallible<()> {
    // 50% chance that the output is correct means all information is lost, query is ε=0 dp
    let m_rr = make_randomized_response_bool(0.5, false)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    // 100% chance that the output is correct is ε=inf dp
    let m_rr = make_randomized_response_bool(1.0, false)?;
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
fn test_cat_extremes() -> Fallible<()> {
    let categories = HashSet::from([2, 3, 5, 7, 8]);
    let m_rr = make_randomized_response(categories.clone(), 1. / 5.)?;
    assert_eq!(m_rr.map(&1)?, 2.220446049250313e-16);

    let m_rr = make_randomized_response(categories, 1.)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    Ok(())
}
