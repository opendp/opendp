use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    domains::LazyFrameDomain,
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util::{as_ref, to_option_str, to_str},
    },
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
        active_column(rust_type = b"null")
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
    active_column: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe_domain =
        try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>()).clone();

    let active_column = try_!(to_str(active_column)).to_string();

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

        AnyDomain::new(ExprDomain::new(
            lazyframe_domain,
            context,
            Some(active_column),
            true,
        ))
    } else if let Some(object) = as_ref(grouping_columns) {
        let columns = try_!(object.downcast_ref::<Vec<String>>()).clone();
        AnyDomain::new(ExprDomain::new(
            lazyframe_domain,
            LazyGroupByContext { columns },
            Some(active_column),
            true,
        ))
    } else {
        return err!(FFI, "must provide either context or grouping_columns").into();
    })
    .into()
}
