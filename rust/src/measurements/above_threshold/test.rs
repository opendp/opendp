use crate::measures::MaxDivergence;

use super::*;

#[test]
fn test_above_threshold() -> Fallible<()> {
    let m_above = make_sparse_vector(
        StreamDomain::new(AtomDomain::default()),
        LInfDistance::new(true),
        MaxDivergence,
        1.,
        100.,
        Optimize::Max,
    )?;

    let mut qbl_at = m_above.invoke(&Queryable::new_external(|query: &f64| Ok(*query))?)?;

    let msg = |r: Fallible<bool>| r.map_err(|e| e.message.unwrap());

    println!("too small:       {:?}", msg(qbl_at.eval(&1.)));
    println!("maybe true:      {:?}", msg(qbl_at.eval(&100.)));
    println!("definitely true: {:?}", msg(qbl_at.eval(&1000.)));
    println!("exhausted:       {:?}", msg(qbl_at.eval(&1000.)));

    Ok(())
}
