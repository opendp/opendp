use crate::error::ExplainUnwrap;

use super::*;

#[test]
fn test_subset_by() -> Fallible<()> {
    #[allow(deprecated)]
    let trans = make_subset_by::<String>("filter".to_string(), vec!["values".to_string()])?;

    let mut df = DataFrame::new();
    df.insert("filter".to_string(), vec![true, false, false, true].into());
    df.insert("values".to_string(), vec!["1", "2", "3", "4"].into());
    let res = trans.invoke(&df)?;

    let subset = res
        .get("values")
        .unwrap_test()
        .as_form::<Vec<&str>>()?
        .clone();

    assert_eq!(subset, vec!["1", "4"]);
    Ok(())
}
