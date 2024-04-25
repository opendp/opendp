use crate::metrics::{HammingDistance, InsertDeleteDistance, SymmetricDistance};

use super::*;

#[test]
fn test_cast() -> Fallible<()> {
    let data = vec![1., 1e10, 0.5, f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
    let caster = make_cast::<_, f64, i64>(
        VectorDomain::new(AtomDomain::default()),
        InsertDeleteDistance::default(),
    )?;
    assert_eq!(
        caster.invoke(&data)?,
        vec![Some(1), Some(10000000000), Some(0), None, None, None]
    );

    let caster = make_cast::<_, f64, u8>(
        VectorDomain::new(AtomDomain::default()).with_size(4),
        HammingDistance::default(),
    )?;
    assert_eq!(
        caster.invoke(&vec![-1., f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
        vec![None; 4]
    );
    Ok(())
}

#[test]
fn test_cast_combinations() -> Fallible<()> {
    macro_rules! test_pair {
        ($from:ty, $to:ty) => {
            let caster = make_cast::<_, $from, $to>(
                VectorDomain::new(AtomDomain::default()),
                SymmetricDistance::default(),
            )?;
            caster.invoke(&vec![<$from>::default()])?;
            let caster = make_cast::<_, $from, $to>(
                VectorDomain::new(AtomDomain::default()),
                SymmetricDistance::default(),
            )?;
            caster.invoke(&vec![<$from>::default()])?;
        };
    }
    macro_rules! test_cartesian {
        ([];[$first:ty, $($end:ty),*]) => {
            test_pair!($first, $first);
            $(test_pair!($first, $end);)*

            test_cartesian!{[$first];[$($end),*]}
        };
        ([$($start:ty),*];[$mid:ty, $($end:ty),*]) => {
            $(test_pair!($mid, $start);)*
            test_pair!($mid, $mid);
            $(test_pair!($mid, $end);)*

            test_cartesian!{[$($start),*, $mid];[$($end),*]}
        };
        ([$($start:ty),*];[$last:ty]) => {
            test_pair!($last, $last);
            $(test_pair!($last, $start);)*
        };
    }
    test_cartesian! {[];[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, bool]}
    Ok(())
}

#[test]
fn test_cast_default_unsigned() -> Fallible<()> {
    let caster = make_cast_default::<_, f64, u8>(Default::default(), SymmetricDistance::default())?;
    assert_eq!(caster.invoke(&vec![-1.])?, vec![u8::default()]);
    Ok(())
}

#[test]
fn test_cast_default_parse() -> Fallible<()> {
    let data = vec![
        "2".to_string(),
        "3".to_string(),
        "a".to_string(),
        "".to_string(),
    ];

    let caster =
        make_cast_default::<_, String, u8>(Default::default(), SymmetricDistance::default())?;
    assert_eq!(
        caster.invoke(&data)?,
        vec![2, 3, u8::default(), u8::default()]
    );

    let caster =
        make_cast_default::<_, String, f64>(Default::default(), SymmetricDistance::default())?;
    assert_eq!(
        caster.invoke(&data)?,
        vec![2., 3., f64::default(), f64::default()]
    );
    Ok(())
}

#[test]
fn test_cast_default_floats() -> Fallible<()> {
    let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
    let caster =
        make_cast_default::<_, f64, String>(Default::default(), SymmetricDistance::default())?;
    assert_eq!(
        caster.invoke(&data)?,
        vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]
    );

    let caster = make_cast_default::<_, f64, u8>(Default::default(), SymmetricDistance::default())?;
    assert_eq!(
        caster.invoke(&vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
        vec![u8::default(), u8::default(), u8::default()]
    );

    let data = vec![
        "1e+2",
        "1e2",
        "1e+02",
        "1.e+02",
        "1.0E+02",
        "1.0E+00002",
        "01.E+02",
        "1.0E2",
    ]
    .into_iter()
    .map(|v| v.to_string())
    .collect();
    let caster =
        make_cast_default::<_, String, f64>(Default::default(), SymmetricDistance::default())?;
    assert!(caster.invoke(&data)?.into_iter().all(|v| v == 100.));
    Ok(())
}

#[test]
fn test_cast_inherent() -> Fallible<()> {
    let data = vec!["abc".to_string(), "1".to_string(), "1.".to_string()];
    let caster = make_cast_inherent::<_, String, f64>(
        VectorDomain::default(),
        SymmetricDistance::default(),
    )?;
    let res = caster.invoke(&data)?;
    assert!(res[0].is_nan());
    assert_eq!(res[1..], vec![1., 1.]);
    Ok(())
}
