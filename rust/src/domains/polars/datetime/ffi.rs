use std::ffi::c_char;

use opendp_derive::bootstrap;
use polars::prelude::TimeUnit;

use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::{any::AnyDomain, util},
};

use super::DatetimeDomain;

#[bootstrap(
    arguments(time_unit(default = "us"), time_zone(default = b"null")),
    generics(DI(suppress)),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `DatetimeDomain`.
///
/// Documentation on valid time zones can be found [in the Polars documentation](https://docs.pola.rs/user-guide/transformations/time-series/timezones/).
///
/// # Arguments
/// * `time_unit` - One of `ns`, `us` or `ms`, corresponding to nano-, micro-, and milliseconds
/// * `time_zone` - Optional time zone.
fn datetime_domain(time_unit: &str, time_zone: Option<&str>) -> Fallible<DatetimeDomain> {
    let time_unit = match time_unit {
        "ns" => TimeUnit::Nanoseconds,
        "us" => TimeUnit::Microseconds,
        "ms" => TimeUnit::Milliseconds,
        _ => {
            return fallible!(
                MakeDomain,
                "time unit ({time_unit}) must be of `ns`, `us` or `ms`, corresponding to nano-, micro-, and milliseconds"
            )
        }
    };
    Ok(DatetimeDomain {
        time_unit,
        time_zone: time_zone.map(|s| s.into()),
    })
}

#[no_mangle]
pub extern "C" fn opendp_domains__datetime_domain(
    time_unit: *mut c_char,
    time_zone: *mut c_char,
) -> FfiResult<*mut AnyDomain> {
    let time_unit = try_!(util::to_str(time_unit));
    let time_zone = util::to_str(time_zone).ok();

    Ok(AnyDomain::new(try_!(datetime_domain(time_unit, time_zone)))).into()
}
