use std::ffi::c_char;

use chrono::{NaiveDate, NaiveTime};
use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, MetricSpace},
    domains::{AtomDomain, CategoricalDomain, DatetimeDomain, OptionDomain, PrimitiveDataType},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, Type},
    },
    traits::CheckAtom,
    transformations::DatasetMetric,
};

use super::{SeriesDomain, SeriesElementDomain};

#[bootstrap(
    arguments(element_domain(c_type = "AnyDomain *", rust_type = b"null")),
    generics(DI(suppress)),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `SeriesDomain`.
///
/// # Arguments
/// * `name` - The name of the series.
/// * `element_domain` - The domain of elements in the series.
fn series_domain<DI: 'static + SeriesElementDomain>(
    name: &str,
    element_domain: DI,
) -> SeriesDomain {
    SeriesDomain::new(name, element_domain)
}

#[no_mangle]
pub extern "C" fn opendp_domains__series_domain(
    name: *mut c_char,
    element_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    let name = try_!(util::to_str(name));
    let element_domain = try_as_ref!(element_domain);
    let DA = element_domain.type_.clone();
    let T = try_!(DA.get_atom());

    if DA.descriptor.starts_with("OptionDomain") {
        if T == Type::of::<CategoricalDomain>() {
            let element_domain =
                try_!(element_domain.downcast_ref::<OptionDomain<CategoricalDomain>>()).clone();
            return Ok(AnyDomain::new(series_domain(name, element_domain))).into();
        }
        if T == Type::of::<DatetimeDomain>() {
            let element_domain =
                try_!(element_domain.downcast_ref::<OptionDomain<DatetimeDomain>>()).clone();
            return Ok(AnyDomain::new(series_domain(name, element_domain))).into();
        }

        fn monomorphize_option<T: 'static + CheckAtom + PrimitiveDataType>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain
                .downcast_ref::<OptionDomain<AtomDomain<T>>>()?
                .clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }

        dispatch!(
            monomorphize_option,
            // These types are the Polars primitive datatypes.
            // Don't forget to update the corresponding list below.
            [(
                T,
                [u32, u64, i32, i64, f32, f64, bool, String, NaiveDate, NaiveTime]
            )],
            (name, element_domain)
        )
        .into()
    } else {
        if T == Type::of::<CategoricalDomain>() {
            let element_domain = try_!(element_domain.downcast_ref::<CategoricalDomain>()).clone();
            return Ok(AnyDomain::new(series_domain(name, element_domain))).into();
        }

        if T == Type::of::<DatetimeDomain>() {
            let element_domain = try_!(element_domain.downcast_ref::<DatetimeDomain>()).clone();
            return Ok(AnyDomain::new(series_domain(name, element_domain))).into();
        }

        fn monomorphize_atom<T: 'static + CheckAtom + PrimitiveDataType>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain.downcast_ref::<AtomDomain<T>>()?.clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }
        dispatch!(
            monomorphize_atom,
            [(
                T,
                [u32, u64, i32, i64, f32, f64, bool, String, NaiveDate, NaiveTime]
            )],
            (name, element_domain)
        )
        .into()
    }
}

#[bootstrap(
    name = "categorical_domain",
    arguments(categories(rust_type = "Option<Vec<String>>", default = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `CategoricalDomain`.
/// Can be used as an argument to a Polars series domain.
///
/// # Arguments
/// * `categories` - Optional ordered set of valid string categories
#[no_mangle]
pub extern "C" fn opendp_domains__categorical_domain(
    categories: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let domain = if let Some(categories) = util::as_ref(categories) {
        let categories = try_!(categories.downcast_ref::<Vec<String>>())
            .into_iter()
            .map(|s| s.into())
            .collect();
        try_!(CategoricalDomain::new_with_categories(categories))
    } else {
        CategoricalDomain::default()
    };

    Ok(AnyDomain::new(domain)).into()
}

impl MetricSpace for (SeriesDomain, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self;

        fn monomorphize_dataset<M: 'static + DatasetMetric>(
            domain: &SeriesDomain,
            metric: &AnyMetric,
        ) -> Fallible<()>
        where
            (SeriesDomain, M): MetricSpace,
        {
            let metric = metric.downcast_ref::<M>()?;
            (domain.clone(), metric.clone()).check_space()
        }
        let M = metric.type_.clone();

        fn in_set<T>() -> Option<()> {
            Some(())
        }

        if let Some(_) = dispatch!(in_set, [(M, @dataset_metrics)]) {
            dispatch!(monomorphize_dataset, [
                (M, @dataset_metrics)
            ], (domain, metric))
        } else {
            fallible!(MetricSpace, "Unsupported metric: {}", M.descriptor)
        }
    }
}
