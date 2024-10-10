use crate::{error::ExplainUnwrap, metrics::SymmetricDistance};

use super::*;

#[test]
fn test_df_cast_default() -> Fallible<()> {
    #[allow(deprecated)]
    let trans = make_df_cast_default::<String, i32, bool, _>(
        Default::default(),
        SymmetricDistance::default(),
        "filter".to_string(),
    )?;

    let mut df = DataFrame::new();
    df.insert("filter".to_string(), vec![0, 1, 3, 0].into());
    df.insert("values".to_string(), vec!["1", "2", "3", "4"].into());
    let res = trans.invoke(&df)?;

    let filter = res
        .get("filter")
        .unwrap_test()
        .as_form::<Vec<bool>>()?
        .clone();

    assert_eq!(filter, vec![false, true, true, false]);

    Ok(())
}

#[test]
fn test_df_is_equal() -> Fallible<()> {
    #[allow(deprecated)]
    let trans = make_df_is_equal(
        Default::default(),
        SymmetricDistance::default(),
        0,
        "true".to_string(),
    )?;

    let mut df = DataFrame::new();
    df.insert(
        0,
        vec![
            "false".to_string(),
            "true".to_string(),
            "true".to_string(),
            "false".to_string(),
        ]
        .into(),
    );
    df.insert(1, vec![12., 23., 94., 128.].into());
    let res = trans.invoke(&df)?;

    let filter = res.get(&0).unwrap_test().as_form::<Vec<bool>>()?.clone();

    assert_eq!(filter, vec![false, true, true, false]);

    Ok(())
}
