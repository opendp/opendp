use std::{any::TypeId, ffi::c_char, os::raw::c_void};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, MetricSpace},
    domains::{Margin, MarginPub, SeriesDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, AnyDomainPtr, Type},
    },
    transformations::DatasetMetric,
};

use super::{Frame, FrameDomain, LazyFrameDomain};
use polars::prelude::*;

#[bootstrap(
    name = "lazyframe_domain",
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "LazyFrameDomain")
)]
/// Construct an instance of `LazyFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the lazyframe.
#[no_mangle]
pub extern "C" fn opendp_domains__lazyframe_domain(
    series_domains: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_series_domain",
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Retrieve the series domain at index `column`.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
/// * `column` - Index of the SeriesDomain
#[no_mangle]
pub extern "C" fn opendp_domains___lazyframe_domain_get_series_domain(
    lazyframe_domain: *const AnyDomain,
    column: u32,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let series_domain = try_!(lazyframe_domain
        .series_domains
        .get(column as usize)
        .ok_or_else(|| err!(FFI, "column {column} does not exist in the dataframe")));
    Ok(AnyDomain::new(series_domain.clone())).into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_margin",
    arguments(by(rust_type = "Vec<String>")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the series domain at index 'column`.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
/// * `by` - grouping columns
#[no_mangle]
pub extern "C" fn opendp_domains___lazyframe_domain_get_margin(
    lazyframe_domain: *const AnyDomain,
    by: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<String>>());
    let by = by.iter().map(|v| PlSmallStr::from(v)).collect();
    let margin = lazyframe_domain.get_margin(&by);
    Ok(AnyObject::new(margin)).into()
}

#[bootstrap(
    name = "_margin_get_max_partition_length",
    arguments(margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the max partition length from a margin
///
/// # Arguments
/// * `margin` - Margin to introspect
#[no_mangle]
pub extern "C" fn opendp_domains___margin_get_max_partition_length(
    margin: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>());
    Ok(AnyObject::new(
        margin.max_partition_length.clone().map(AnyObject::new),
    ))
    .into()
}

#[bootstrap(
    name = "_margin_get_max_num_partitions",
    arguments(margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the max num partitions from a margin
///
/// # Arguments
/// * `margin` - Margin to introspect
#[no_mangle]
pub extern "C" fn opendp_domains___margin_get_max_num_partitions(
    margin: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>());
    Ok(AnyObject::new(
        margin.max_num_partitions.clone().map(AnyObject::new),
    ))
    .into()
}

#[bootstrap(
    name = "_margin_get_max_partition_contributions",
    arguments(margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the max partition contributions from a margin
///
/// # Arguments
/// * `margin` - Margin to introspect
#[no_mangle]
pub extern "C" fn opendp_domains___margin_get_max_partition_contributions(
    margin: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>());
    Ok(AnyObject::new(
        margin
            .max_partition_contributions
            .clone()
            .map(AnyObject::new),
    ))
    .into()
}

#[bootstrap(
    name = "_margin_get_max_influenced_partitions",
    arguments(margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the max influenced partitions from a margin
///
/// # Arguments
/// * `margin` - Margin to introspect
#[no_mangle]
pub extern "C" fn opendp_domains___margin_get_max_influenced_partitions(
    margin: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>());
    Ok(AnyObject::new(
        margin.max_influenced_partitions.clone().map(AnyObject::new),
    ))
    .into()
}

#[bootstrap(
    name = "_margin_get_public_info",
    arguments(margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the public info invariant from a margin.
///
/// # Arguments
/// * `margin` - Margin to introspect
#[no_mangle]
pub extern "C" fn opendp_domains___margin_get_public_info(
    margin: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>());
    Ok(AnyObject::new(
        margin
            .public_info
            .map(|info| match info {
                MarginPub::Keys => "keys".to_string(),
                MarginPub::Lengths => "lengths".to_string(),
            })
            .map(AnyObject::new),
    ))
    .into()
}

#[bootstrap()]
/// Construct an empty LazyFrame with the same schema as in the LazyFrameDomain.
///
/// This is useful for creating a dummy lazyframe used to write a query plan.
///
/// # Arguments
/// * `domain` - A LazyFrameDomain.
fn _lazyframe_from_domain(domain: LazyFrameDomain) -> Fallible<LazyFrame> {
    Ok(DataFrame::from_rows_and_schema(&[], &domain.schema())?.lazy())
}

#[no_mangle]
pub extern "C" fn opendp_domains___lazyframe_from_domain(
    domain: *mut AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let domain = try_!(try_as_ref!(domain).downcast_ref::<LazyFrameDomain>()).clone();
    _lazyframe_from_domain(domain).map(AnyObject::new).into()
}

pub(crate) fn unpack_series_domains(
    series_domains: *const AnyObject,
) -> Fallible<Vec<SeriesDomain>> {
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
        max_partition_length(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        max_num_partitions(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        max_partition_contributions(
            c_type = "void *",
            rust_type = "Option<u32>",
            default = b"null"
        ),
        max_influenced_partitions(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        public_info(rust_type = "Option<String>", default = b"null")
    ),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "LazyFrameDomain")
)]
#[no_mangle]
pub extern "C" fn opendp_domains__with_margin(
    frame_domain: *mut AnyDomain,
    by: *mut AnyObject,
    max_partition_length: *mut c_void,
    max_num_partitions: *mut c_void,
    max_partition_contributions: *mut c_void,
    max_influenced_partitions: *mut c_void,
    public_info: *mut c_char,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<String>>()).clone();

    let margin = Margin {
        max_partition_length: util::as_ref(max_partition_length as *const u32).cloned(),
        max_num_partitions: util::as_ref(max_num_partitions as *const u32).cloned(),
        max_partition_contributions: util::as_ref(max_partition_contributions as *const u32)
            .cloned(),
        max_influenced_partitions: util::as_ref(max_influenced_partitions as *const u32).cloned(),
        public_info: match try_!(util::to_option_str(public_info)) {
            Some("keys") => Some(MarginPub::Keys),
            Some("lengths") => Some(MarginPub::Lengths),
            None => None,
            _ => return err!(FFI, "public_info must be one of 'keys' or 'lengths'").into(),
        },
    };

    let frame_domain = try_as_ref!(frame_domain);
    let F_ = match frame_domain.type_.id {
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

    dispatch!(
        monomorphize,
        [(F_, [DataFrame, LazyFrame])],
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
