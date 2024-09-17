use super::*;
use num::Float as _;
use std::iter::FromIterator;

#[test]
fn test_bool() -> Fallible<()> {
    let ran_res = make_randomized_response_bool(0.75, false)?;
    let res = ran_res.invoke(&false)?;
    println!("{:?}", res);
    assert!(ran_res.check(&1, &3.0.ln())?);
    assert!(!ran_res.check(&1, &2.99999.ln())?);
    Ok(())
}
#[test]
fn test_bool_extremes() -> Fallible<()> {
    // 50% chance that the output is correct means all information is lost, query is 0-dp
    let ran_res = make_randomized_response_bool(0.5, false)?;
    assert!(ran_res.check(&1, &0.0)?);
    // 100% chance that the output is correct is inf-dp, so expect an error
    assert!(make_randomized_response_bool(1.0, false).is_err());
    Ok(())
}
#[test]
fn test_cat() -> Fallible<()> {
    let ran_res = make_randomized_response(HashSet::from_iter(vec![2, 3, 5, 6].into_iter()), 0.75)?;
    let res = ran_res.invoke(&3)?;
    println!("{:?}", res);
    // (.75 * 3 / .25) = 9
    assert!(ran_res.check(&1, &9.0.ln())?);
    assert!(!ran_res.check(&1, &8.99999.ln())?);
    Ok(())
}
#[test]
fn test_cat_extremes() -> Fallible<()> {
    let ran_res =
        make_randomized_response(HashSet::from_iter(vec![2, 3, 5, 7, 8].into_iter()), 1. / 5.)?;
    assert!(ran_res.check(&1, &1e-10)?);
    assert!(
        make_randomized_response(HashSet::from_iter(vec![2, 3, 5, 7].into_iter()), 1.).is_err()
    );
    Ok(())
}
