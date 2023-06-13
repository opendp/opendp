use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Metric, MetricSpace},
    domains::LazyFrameDomain,
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{as_ref, to_option_str},
    },
    metrics::{InsertDeleteDistance, SymmetricDistance},
};

use super::{ExprContext, ExprDomain};

#[no_mangle]
#[bootstrap(
    name = "expr_domain",
    features("contrib"),
    arguments(
        lazyframe_domain(c_type = "AnyDomain *", rust_type = b"null"),
        context(default = b"null", rust_type = b"null"),
        grouping_columns(rust_type = "Option<Vec<String>>", default = b"null", hint = "List[str]"),
        active_column(
            c_type = "AnyObject *",
            rust_type = "Option<String>",
            default = b"null"
        )
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
pub extern "C" fn opendp_domains__expr_domain(
    lazyframe_domain: *const AnyDomain,
    context: *const c_char,
    grouping_columns: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let lf_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>()).clone();

    let context = if let Some(context_str) = try_!(to_option_str(context)) {
        match context_str.to_lowercase().as_str() {
            "select" => ExprContext::Select,
            "filter" => ExprContext::Filter,
            "with_columns" => ExprContext::WithColumns,
            _ => {
                return err!(
                FFI,
                "context must be one of `select`, `filter` or `with_columns`, or pass `grouping_columns`"
            )
                .into()
            }
        }
    } else if let Some(object) = as_ref(grouping_columns) {
        let grouping_columns = try_!(object.downcast_ref::<Vec<String>>()).clone();
        ExprContext::Aggregate(grouping_columns.into_iter().collect())
    } else {
        return err!(FFI, "must pass either `context` or `grouping_columns`").into();
    };

    Ok(AnyDomain::new(ExprDomain::new(lf_domain, context))).into()
}

impl MetricSpace for (ExprDomain, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self.clone();

        fn monomorphize<M: 'static + Metric>(domain: ExprDomain, metric: AnyMetric) -> Fallible<()>
        where
            (ExprDomain, M): MetricSpace,
        {
            let input_metric = metric.downcast_ref::<M>()?;
            (domain.clone(), input_metric.clone()).check_space()
        }

        dispatch!(
            monomorphize,
            [(metric.type_, [SymmetricDistance, InsertDeleteDistance])],
            (domain, metric)
        )
    }
}
