use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, MetricSpace},
    domains::SeriesDomain,
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, AnyDomainPtr, Type},
    },
    transformations::DatasetMetric,
};

use super::{DataFrameDomain, Frame, FrameDomain, LazyFrameDomain};
use polars::prelude::*;

#[bootstrap(
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
#[allow(dead_code)]
/// Construct an instance of `LazyFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the lazyframe.
fn lazyframe_domain(series_domains: Vec<SeriesDomain>) -> Fallible<LazyFrameDomain> {
    LazyFrameDomain::new(series_domains)
}

#[no_mangle]
pub extern "C" fn opendp_domains__lazyframe_domain(
    series_domains: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
#[allow(dead_code)]
/// Construct an instance of `DataFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the dataframe.
fn dataframe_domain(series_domains: Vec<SeriesDomain>) -> Fallible<DataFrameDomain> {
    DataFrameDomain::new(series_domains)
}

#[no_mangle]
pub extern "C" fn opendp_domains__dataframe_domain(
    series_domains: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(DataFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

fn unpack_series_domains(series_domains: *mut AnyObject) -> Fallible<Vec<SeriesDomain>> {
    let vec_any = try_as_ref!(series_domains).downcast_ref::<Vec<AnyDomainPtr>>()?;

    vec_any
        .iter()
        .map(|x| {
            util::as_ref(x.clone())
                .and_then(|ad| ad.downcast_ref::<SeriesDomain>().ok())
                .cloned()
        })
        .collect::<Option<Vec<SeriesDomain>>>()
        .ok_or_else(|| err!(FailedCast, "domain downcast failed"))
}

#[bootstrap(
    name = "lazy_frame_domain_with_counts",
    arguments(
        lazy_frame_domain(rust_type = b"null"),
        counts(rust_type = "LazyFrame")
    ),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
#[no_mangle]
pub extern "C" fn opendp_domains__lazy_frame_domain_with_counts(
    lazy_frame_domain: *mut AnyDomain,
    counts: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let lazy_frame_domain =
        try_!(try_as_ref!(lazy_frame_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let counts = try_!(try_as_ref!(counts).downcast_ref::<LazyFrame>()).clone();

    let lazy_frame_domain = try_!(lazy_frame_domain.with_counts(counts));
    Ok(AnyDomain::new(lazy_frame_domain)).into()
}

impl<F: 'static + Frame> MetricSpace for (FrameDomain<F>, AnyMetric) {
    fn check(&self) -> bool {
        let (domain, metric) = self;

        fn monomorphize_dataset<F: Frame, M: 'static + DatasetMetric>(
            domain: &FrameDomain<F>,
            metric: &AnyMetric,
        ) -> Fallible<bool>
        where
            (FrameDomain<F>, M): MetricSpace,
        {
            let metric = metric.downcast_ref::<M>()?;
            Ok((domain.clone(), metric.clone()).check())
        }
        let F = Type::of::<F>();
        let M = metric.type_.clone();

        fn in_set<T>() -> Option<()> {
            Some(())
        }

        if let Some(_) = dispatch!(in_set, [(M, @dataset_metrics)]) {
            return dispatch!(monomorphize_dataset, [
                (F, [F]),
                (M, @dataset_metrics)
            ], (domain, metric))
            .unwrap_or(false);
        }

        false
    }
}
