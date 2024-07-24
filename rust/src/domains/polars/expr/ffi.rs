use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, FfiSlice},
    domains::{polars::ffi::unpack_series_domains, Margin},
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util,
    },
};

use super::{Context, WildExprDomain};

#[unsafe(no_mangle)]
#[bootstrap(
    name = "wild_expr_domain",
    features("contrib"),
    arguments(
        columns(rust_type = "Vec<SeriesDomain>"),
        margin(rust_type = "Option<Margin>", default = b"null")
    )
)]
/// Construct a WildExprDomain.
///
/// # Arguments
/// * `columns` - descriptors for each column in the data
/// * `by` - optional. Set if expression is applied to grouped data
/// * `margin` - descriptors for grouped data
pub extern "C" fn opendp_domains__wild_expr_domain(
    columns: *const FfiSlice,
    margin: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {

    let columns = try_!(unpack_series_domains(columns));

    let context = if let Some(margin) = util::as_ref(margin) {
        Context::Aggregation {
            margin: try_!(margin.downcast_ref::<Margin>()).clone(),
        }
    } else {
        Context::RowByRow
    };

    Ok(AnyDomain::new(WildExprDomain { columns, context })).into()
}
