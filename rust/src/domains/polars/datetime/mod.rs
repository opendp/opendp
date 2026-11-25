use polars::prelude::{DataType, Series, TimeUnit, TimeZone};

use crate::{core::Domain, error::Fallible};

use super::SeriesElementDomain;

#[cfg(feature = "ffi")]
mod ffi;

/// A domain that represents a datetime.
/// Number of milli/nano/micro seconds since Unix epoch.
///
/// Consider the set of all possible time points distinct for each setting of time_zone.
/// Calculations that convert between time zones map between these disjoint sets of time points.
#[derive(Debug, Clone, PartialEq)]
pub struct DatetimeDomain {
    pub time_unit: TimeUnit,
    /// See https://docs.pola.rs/user-guide/transformations/time-series/timezones/
    pub time_zone: Option<TimeZone>,
}

impl Domain for DatetimeDomain {
    // i64 is the physical type. Not using chrono::NaiveDateTime (nanoseconds) because the time unit may vary
    type Carrier = i64;

    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        // all integers correspond to valid datetimes in the domain
        Ok(true)
    }
}

impl SeriesElementDomain for DatetimeDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Datetime(self.time_unit.clone(), self.time_zone.clone())
    }

    fn check_values(&self, _: &Series) -> Fallible<bool> {
        // All i64 timestamps are in the domain once the time unit and zone match.
        Ok(true)
    }

    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}
