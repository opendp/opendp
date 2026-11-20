use polars::{prelude::NamedFrom, series::Series};

use crate::{core::Domain, error::Fallible};

use super::EnumDomain;

#[test]
fn test_enum_member() -> Fallible<()> {
    let domain = EnumDomain::new(Series::new("".into(), vec!["a", "b", "c"]))?;
    assert!(domain.member(&"a".into())?);
    assert!(!domain.member(&"d".into())?);
    Ok(())
}

#[test]
fn test_enum_rejects_null_categories() {
    let categories = Series::new("".into(), [Some("a"), None]);
    assert!(EnumDomain::new(categories).is_err());
}
