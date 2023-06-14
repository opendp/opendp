use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    domains::{AtomDomain, DataTypeFrom, OptionDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, Downcast},
        util,
    },
    traits::CheckAtom,
};

use super::{SeriesAtomDomain, SeriesDomain};

#[bootstrap(
<<<<<<< HEAD
    arguments(element_domain(c_type = "AnyDomain *")),
    generics(DI(example = "element_domain")),
=======
    arguments(element_domain(c_type = "AnyDomain *", rust_type = b"null")),
    generics(DI(suppress)),
>>>>>>> remotes/origin/773-sum-metrics
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `SeriesDomain`.
///
/// # Arguments
/// * `name` - The name of the series.
/// * `element_domain` - The domain of elements in the series.
fn series_domain<DI: 'static + SeriesAtomDomain>(name: &str, element_domain: DI) -> SeriesDomain {
    SeriesDomain::new(name, element_domain)
}

#[no_mangle]
pub extern "C" fn opendp_domains__series_domain(
    name: *mut c_char,
    element_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    let name = try_!(util::to_str(name));
    let element_domain = try_as_ref!(element_domain);
    let DA = element_domain.type_.clone();
    let T = try_!(DA.get_atom());

    if DA.descriptor.starts_with("OptionDomain") {
        fn monomorphize_option<T: 'static + CheckAtom + DataTypeFrom>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain
                .downcast_ref::<OptionDomain<AtomDomain<T>>>()?
                .clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }
        dispatch!(
            monomorphize_option,
            [(T, [u32, u64, i32, i64, f32, f64, bool, String])],
            (name, element_domain)
        )
        .into()
    } else {
        fn monomorphize_atom<T: 'static + CheckAtom + DataTypeFrom>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain.downcast_ref::<AtomDomain<T>>()?.clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }
        dispatch!(
            monomorphize_atom,
            [(T, [u32, u64, i32, i64, f32, f64, bool, String])],
            (name, element_domain)
        )
        .into()
    }
}
