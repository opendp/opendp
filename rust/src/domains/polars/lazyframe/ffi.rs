use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    domains::SeriesDomain,
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util::{self, AnyDomainPtr},
    },
};

use super::LazyFrameDomain;

#[bootstrap(
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
#[allow(dead_code)]
/// Construct an instance of `LazyFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the lazyframe.
fn lazy_frame_domain(series_domains: Vec<SeriesDomain>) -> Fallible<LazyFrameDomain> {
    LazyFrameDomain::new(series_domains)
}

#[no_mangle]
pub extern "C" fn opendp_domains__lazy_frame_domain(
    series_domains: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let vec_any = try_!(try_as_ref!(series_domains).downcast_ref::<Vec<AnyDomainPtr>>());

    let series_domains = try_!(vec_any
        .iter()
        .map(|x| {
            util::as_ref(x.clone())
                .and_then(|ad| ad.downcast_ref::<SeriesDomain>().ok())
                .cloned()
        })
        .collect::<Option<Vec<SeriesDomain>>>()
        .ok_or_else(|| err!(FailedCast, "domain downcast failed")));

    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(series_domains)))).into()
<<<<<<< HEAD
}
=======
}
>>>>>>> remotes/origin/773-sum-metrics
