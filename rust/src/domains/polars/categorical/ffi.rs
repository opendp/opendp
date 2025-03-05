use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util,
    },
};

use super::CategoricalDomain;

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
