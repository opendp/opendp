use std::{any::TypeId, collections::HashSet, ffi::c_char};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Metric, MetricSpace},
    domains::{Margin, SeriesDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, AnyDomainPtr, Type},
    },
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, FrameDistance, HammingDistance,
        InsertDeleteDistance, SymmetricDistance, SymmetricIdDistance,
    },
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
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__lazyframe_domain(
    series_domains: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_columns",
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the column names of the LazyFrameDomain.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the column names from
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___lazyframe_domain_get_columns(
    lazyframe_domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let columns = (lazyframe_domain.series_domains.iter())
        .map(|s| s.name.to_string())
        .collect::<Vec<_>>();
    Ok(AnyObject::new(columns)).into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_series_domain",
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Retrieve the series domain at index `column`.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
/// * `name` - Name of the SeriesDomain to retrieve
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___lazyframe_domain_get_series_domain(
    lazyframe_domain: *const AnyDomain,
    name: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let name = try_!(util::to_str(name));
    let series_domain = try_!(lazyframe_domain.series_domain(name.into()));
    Ok(AnyDomain::new(series_domain)).into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_margin",
    arguments(by(rust_type = "Vec<Expr>")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the series domain at index 'column`.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
/// * `by` - grouping columns
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___lazyframe_domain_get_margin(
    lazyframe_domain: *const AnyDomain,
    by: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<Expr>>());
    let margin = lazyframe_domain.get_margin(&HashSet::from_iter(by.iter().cloned()));
    Ok(AnyObject::new(margin)).into()
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

#[unsafe(no_mangle)]
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
                .and_then(|ad: &AnyDomain| ad.downcast_ref::<SeriesDomain>().ok())
                .cloned()
        })
        .collect::<Option<Vec<SeriesDomain>>>()
        .ok_or_else(|| err!(FailedCast, "domain downcast failed"))
}

#[bootstrap(
    name = "with_margin",
    arguments(frame_domain(rust_type = b"null"), margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "LazyFrameDomain")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__with_margin(
    frame_domain: *mut AnyDomain,
    margin: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>()).clone();

    let frame_domain = try_as_ref!(frame_domain);
    let F_ = match frame_domain.type_.id {
        x if x == TypeId::of::<LazyFrameDomain>() => Type::of::<LazyFrame>(),
        _ => {
            return err!(
                FFI,
                "No match for concrete type {}",
                frame_domain.type_.descriptor
            )
            .into();
        }
    };

    fn monomorphize<F: 'static + Frame>(domain: &AnyDomain, margin: Margin) -> Fallible<AnyDomain> {
        let domain = domain.downcast_ref::<FrameDomain<F>>()?.clone();
        Ok(AnyDomain::new(domain.with_margin(margin)?))
    }

    dispatch!(
        monomorphize,
        [(F_, [DataFrame, LazyFrame])],
        (domain, margin)
    )
    .into()
}

impl<F: 'static + Frame> MetricSpace for (FrameDomain<F>, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self;

        fn monomorphize_dataset<F: Frame, M: 'static + Metric>(
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

        // unbounded metrics
        if dispatch!(
            in_set,
            [(
                M,
                [SymmetricDistance, SymmetricIdDistance, InsertDeleteDistance]
            )]
        )
        .is_some()
        {
            return dispatch!(
                monomorphize_dataset,
                [
                    (F, [F]),
                    (
                        M,
                        [SymmetricDistance, SymmetricIdDistance, InsertDeleteDistance]
                    )
                ],
                (domain, metric)
            );
        }

        // multi-metrics
        if let Some(_) = dispatch!(in_set, [(M, [FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>, FrameDistance<InsertDeleteDistance>])])
        {
            return dispatch!(monomorphize_dataset, [
                (F, [F]),
                (M, [FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>, FrameDistance<InsertDeleteDistance>])
            ], (domain, metric));
        }

        // bounded metrics
        if dispatch!(
            in_set,
            [(M, [ChangeOneDistance, ChangeOneIdDistance, HammingDistance])]
        )
        .is_some()
        {
            return dispatch!(
                monomorphize_dataset,
                [
                    (F, [F]),
                    (M, [ChangeOneDistance, ChangeOneIdDistance, HammingDistance])
                ],
                (domain, metric)
            );
        }

        fallible!(
            MetricSpace,
            "invalid metric type: {}",
            metric.type_.to_string()
        )
    }
}
