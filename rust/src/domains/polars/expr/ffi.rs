use std::{ffi::c_char, os::raw::c_void};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Metric, MetricSpace},
    domains::{polars::ffi::unpack_series_domains, Margin, MarginPub},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util,
    },
    metrics::{InsertDeleteDistance, SymmetricDistance},
};

use super::{Context, ExprDomain, WildExprDomain};

#[no_mangle]
#[bootstrap(
    name = "wild_expr_domain",
    features("contrib"),
    arguments(
        columns(rust_type = "Vec<SeriesDomain>"),
        by(
            rust_type = "Option<Vec<String>>",
            default = b"null",
            hint = "list[str]"
        ),
        max_partition_length(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        max_num_partitions(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        max_partition_contributions(
            c_type = "void *",
            rust_type = "Option<u32>",
            default = b"null"
        ),
        max_influenced_partitions(c_type = "void *", rust_type = "Option<u32>", default = b"null"),
        public_info(rust_type = "Option<String>", default = b"null")
    )
)]
/// Construct a WildExprDomain.
///
/// # Arguments
/// * `columns` - descriptors for each column in the data
/// * `by` - optional. Set if expression is applied to grouped data
/// * `margin` - descriptors for grouped data
pub extern "C" fn opendp_domains__wild_expr_domain(
    columns: *const AnyObject,
    by: *const AnyObject,
    max_partition_length: *const c_void,
    max_num_partitions: *const c_void,
    max_partition_contributions: *const c_void,
    max_influenced_partitions: *const c_void,
    public_info: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let columns = try_!(unpack_series_domains(columns));

    let context = if let Some(by) = util::as_ref(by) {
        let by = try_!(by.downcast_ref::<Vec<String>>()).clone();
        let by = by.into_iter().map(|s| s.into()).collect();

        let margin = Margin {
            max_partition_length: util::as_ref(max_partition_length as *const u32).cloned(),
            max_num_partitions: util::as_ref(max_num_partitions as *const u32).cloned(),
            max_partition_contributions: util::as_ref(max_partition_contributions as *const u32)
                .cloned(),
            max_influenced_partitions: util::as_ref(max_influenced_partitions as *const u32)
                .cloned(),
            public_info: match try_!(util::to_option_str(public_info)) {
                Some("keys") => Some(MarginPub::Keys),
                Some("lengths") => Some(MarginPub::Lengths),
                None => None,
                _ => return err!(FFI, "public_info must be one of 'keys' or 'lengths'").into(),
            },
        };
        Context::Grouping { by, margin }
    } else {
        Context::RowByRow
    };

    Ok(AnyDomain::new(WildExprDomain { columns, context })).into()
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
