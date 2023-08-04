use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, MetricSpace, Metric},
    domains::{LazyFrameDomain, LazyGroupByDomain},
    ffi::{
        any::{AnyDomain, AnyObject, Downcast, AnyMetric},
        util::{as_ref, to_option_str},
    }, error::Fallible, metrics::{SymmetricDistance, InsertDeleteDistance},
};

use super::{ExprDomain, LazyFrameContext, LazyGroupByContext};

#[no_mangle]
#[bootstrap(
    name = "expr_domain",
    features("contrib"),
    arguments(
        lazyframe_domain(c_type = "AnyDomain *", rust_type = b"null"),
        context(default = b"null", rust_type = b"null"),
        grouping_columns(rust_type = "Option<Vec<String>>", default = b"null"),
        active_column(c_type = "AnyObject *", rust_type = "Option<String>", default = b"null")
    )
)]
/// Construct an ExprDomain from a LazyFrameDomain.
///
/// Must pass either `context` or `grouping_columns`.
///
/// # Arguments
/// * `lazyframe_domain` - the domain of the LazyFrame to be constructed
/// * `context` - used when the constructor is called inside a lazyframe context constructor
/// * `grouping_columns` - used when the constructor is called inside a groupby context constructor
/// * `active_column` - which column to apply expressions to
pub extern "C" fn opendp_domains__expr_domain(
    lazyframe_domain: *const AnyDomain,
    context: *const c_char,
    grouping_columns: *const AnyObject,
    active_column: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe_domain =
        try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>()).clone();

    let active_column = if let Some(object) = as_ref(active_column) {
        Some(try_!(object.downcast_ref::<String>()).clone())
    } else {
        None
    };

    Ok(if let Some(context) = try_!(to_option_str(context)) {
        let context = match context.to_lowercase().as_str() {
            "select" => LazyFrameContext::Select,
            "filter" => LazyFrameContext::Filter,
            "with_columns" => LazyFrameContext::WithColumns,
            _ => {
                return err!(
                    FFI,
                    "unrecognized context, must be one of select, filter or with_columns"
                )
                .into()
            }
        };

        AnyDomain::new(ExprDomain::<LazyFrameDomain>::new(lazyframe_domain, context, active_column))
    } else if let Some(object) = as_ref(grouping_columns) {
        let columns = try_!(object.downcast_ref::<Vec<String>>()).clone();
        AnyDomain::new(ExprDomain::<LazyGroupByDomain>::new(
            lazyframe_domain,
            LazyGroupByContext { columns },
            active_column,
        ))
    } else {
        return err!(FFI, "must provide either context or grouping_columns").into();
    })
    .into()
}


impl MetricSpace for (ExprDomain<LazyFrameDomain>, AnyMetric) {
    fn check(&self) -> bool {
        let (domain, metric) = self.clone();

        fn monomorphize<M: 'static + Metric>(
            domain: ExprDomain<LazyFrameDomain>,
            metric: AnyMetric,
        ) -> Fallible<bool>
        where
            (ExprDomain<LazyFrameDomain>, M): MetricSpace,
        {
            let input_metric = metric.downcast_ref::<M>()?;
            Ok((domain.clone(), input_metric.clone()).check())
        }

        dispatch!(monomorphize, [
            (metric.type_, [SymmetricDistance, InsertDeleteDistance])
        ], (domain, metric))
        .unwrap_or(false)
    }
}
