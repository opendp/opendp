use chrono::{NaiveDate, NaiveTime};
use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    domains::{AtomDomain, CategoricalDomain, DatetimeDomain, DynSeriesElementDomain, EnumDomain},
    error::Fallible,
    ffi::any::{AnyDomain, Downcast},
};

use super::ArrayDomain;

#[bootstrap(
    name = "array_domain",
    arguments(nullable(default = true)),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `ArrayDomain`.
/// Can be used as an argument to a Polars series domain.
///
/// # Arguments
/// * `element_domain` - The domain of each element in the array.
/// * `width` - The width of the array.
/// * `nullable` - Whether elements within an array may be null.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__array_domain(
    element_domain: *const AnyDomain,
    width: u32,
    nullable: bool,
) -> FfiResult<*mut AnyDomain> {
    let element_domain = try_as_ref!(element_domain);

    fn monomorphize<D: DynSeriesElementDomain + Clone>(
        element_domain: &AnyDomain,
        width: u32,
        nullable: bool,
    ) -> Fallible<AnyDomain> {
        let element_domain = try_!(element_domain.downcast_ref::<D>()).clone();
        Ok(AnyDomain::new(
            ArrayDomain::new(element_domain, width as usize).with_nullable(nullable),
        ))
    }

    let D = element_domain.type_.clone();

    dispatch!(monomorphize, [
        (D, [
            AtomDomain<bool>, AtomDomain<String>,
            AtomDomain<u32>, AtomDomain<u64>,
            AtomDomain<i8>, AtomDomain<i16>, AtomDomain<i32>, AtomDomain<i64>,
            AtomDomain<f32>, AtomDomain<f64>,
            AtomDomain<NaiveTime>, AtomDomain<NaiveDate>,
            CategoricalDomain, DatetimeDomain, EnumDomain, ArrayDomain
        ])
    ], (element_domain, width, nullable))
    .into()
}
