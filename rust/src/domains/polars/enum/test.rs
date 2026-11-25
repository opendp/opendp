use polars::{prelude::NamedFrom, series::Series};

use crate::{core::Domain, error::Fallible};

use super::EnumDomain;

#[test]
fn test_enum_member() -> Fallible<()> {
    let domain = EnumDomain::new(Series::new("".into(), vec!["a", "b", "c"]))?;
    assert!(domain.member(&"a".into())?);
    assert!(!domain.member(&"d".into())?);
    let empty = EnumDomain::new(Series::new("".into(), Vec::<&str>::new()))?;
    assert_eq!(empty, EnumDomain::default());
    assert!(!empty.member(&"a".into())?);
    Ok(())
}

#[test]
fn test_enum_rejects_invalid_categories() {
    let categories = Series::new("".into(), [Some("a"), None]);
    assert!(EnumDomain::new(categories).is_err());
    assert!(EnumDomain::new(Series::new("".into(), ["a", "a"])).is_err());
}
