use std::{any::TypeId, ffi::c_char};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, MetricSpace},
    domains::{Margin, MarginPub, SeriesDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, to_option_str, AnyDomainPtr, Type},
    },
    transformations::DatasetMetric,
};

use super::{Frame, FrameDomain, LazyFrameDomain};
use polars::prelude::*;

#[bootstrap(
    name = "lazyframe_domain",
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `LazyFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the lazyframe.
#[no_mangle]
pub extern "C" fn opendp_domains__lazyframe_domain(
    series_domains: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(returns(c_type = "FfiResult<AnyDomain *>"))]
/// Infer the lazyframe domain that a dataset is a member of.
///
/// WARNING: This function looks at the data to infer the domain,
/// and should only be used if you consider the column names and column types to be public information.
///
/// # Arguments
/// * `lazyframe` - The lazyframe to infer the domain from.
fn infer_lazyframe_domain(lazyframe: LazyFrame) -> Fallible<LazyFrameDomain> {
    LazyFrameDomain::new_from_schema(lazyframe.schema()?.as_ref().clone())
}

#[no_mangle]
pub extern "C" fn opendp_domains__infer_lazyframe_domain(
    lazyframe: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();
    Ok(AnyDomain::new(try_!(infer_lazyframe_domain(lazyframe)))).into()
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
    name = "with_margin",
    arguments(
        frame_domain(rust_type = b"null"),
        by(rust_type = "Vec<String>"),
        max_partition_length(rust_type = "Option<u32>", default = b"null"),
        max_num_partitions(rust_type = "Option<u32>", default = b"null"),
        max_partition_contributions(rust_type = "Option<u32>", default = b"null"),
        max_influenced_partitions(rust_type = "Option<u32>", default = b"null"),
        public_info(rust_type = "Option<String>", default = b"null")
    ),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
#[no_mangle]
pub extern "C" fn opendp_domains__with_margin(
    frame_domain: *mut AnyDomain,
    by: *mut AnyObject,
    max_partition_length: *mut AnyObject,
    max_num_partitions: *mut AnyObject,
    max_partition_contributions: *mut AnyObject,
    max_influenced_partitions: *mut AnyObject,
    public_info: *mut c_char,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<String>>()).clone();

    let max_partition_length = if let Some(x) = util::as_ref(max_partition_length) {
        Some(*try_!(x.downcast_ref::<u32>()))
    } else {
        None
    };
    let max_num_partitions = if let Some(x) = util::as_ref(max_num_partitions) {
        Some(*try_!(x.downcast_ref::<u32>()))
    } else {
        None
    };

    let max_partition_contributions = if let Some(x) = util::as_ref(max_partition_contributions) {
        Some(*try_!(x.downcast_ref::<u32>()))
    } else {
        None
    };
    let max_influenced_partitions = if let Some(x) = util::as_ref(max_influenced_partitions) {
        Some(*try_!(x.downcast_ref::<u32>()))
    } else {
        None
    };

    let public_info = if let Some(public_info) = try_!(to_option_str(public_info)) {
        Some(match public_info {
            "keys" => MarginPub::Keys,
            "lengths" => MarginPub::Lengths,
            _ => return err!(FFI, "public_info must be one of 'keys' or 'lengths'").into(),
        })
    } else {
        None
    };

    let frame_domain = try_as_ref!(frame_domain);
    let F = match frame_domain.type_.id {
        x if x == TypeId::of::<LazyFrameDomain>() => Type::of::<LazyFrame>(),
        _ => {
            return err!(
                FFI,
                "No match for concrete type {}",
                frame_domain.type_.descriptor
            )
            .into()
        }
    };

    fn monomorphize<F: 'static + Frame>(
        domain: &AnyDomain,
        by: Vec<String>,
        margin: Margin,
    ) -> Fallible<AnyDomain> {
        let domain = domain.downcast_ref::<FrameDomain<F>>()?.clone();
        Ok(AnyDomain::new(domain.with_margin(&by, margin)?))
    }

    let margin = Margin {
        max_partition_length,
        max_num_partitions,
        max_partition_contributions,
        max_influenced_partitions,
        public_info,
    };

    dispatch!(
        monomorphize,
        [(F, [DataFrame, LazyFrame])],
        (domain, by, margin)
    )
    .into()
}

impl<F: 'static + Frame> MetricSpace for (FrameDomain<F>, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self;

        fn monomorphize_dataset<F: Frame, M: 'static + DatasetMetric>(
            domain: &FrameDomain<F>,
            metric: &AnyMetric,
        ) -> Fallible<()>
        where
            (FrameDomain<F>, M): MetricSpace,
        {
            let metric = metric.downcast_ref::<M>()?;
            (domain.clone(), metric.clone()).check_space()
        }
        let F = Type::of::<F>();
        let M = metric.type_.clone();

        fn in_set<T>() -> Option<()> {
            Some(())
        }

        if let Some(_) = dispatch!(in_set, [(M, @dataset_metrics)]) {
            dispatch!(monomorphize_dataset, [
                (F, [F]),
                (M, @dataset_metrics)
            ], (domain, metric))
        } else {
            fallible!(MetricSpace, "invalid metric type")
        }
    }
}
