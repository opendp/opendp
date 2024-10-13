use polars::prelude::TimeUnit;

use crate::{core::Domain, error::Fallible};

/// A domain that represents a datetime.
#[derive(Debug, Clone, PartialEq)]
pub struct DatetimeDomain {
    pub time_unit: TimeUnit,
    pub time_zone: Option<String>,
}

impl Domain for DatetimeDomain {
    type Carrier = u64;

    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        Ok(true)
    }
}
