use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint, c_void};

use opendp_derive::bootstrap;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, Transformation};
use crate::error::Fallible;
use crate::metrics::{InsertDeleteDistance, SymmetricDistance, IntDistance};
use crate::domains::{AllDomain, BoundedDomain, VectorDomain, SizedDomain};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::any::Downcast;
use crate::ffi::util::Type;
use crate::traits::{CheckNull, TotalOrd};
use crate::transformations::resize::IsMetricOrdered;

#[bootstrap(
    features("contrib"),
    generics(
        MI(default = "SymmetricDistance"),
        MO(default = "SymmetricDistance")
    )
)]
/// Make a Transformation that either truncates or imputes records 
/// with `constant` in a Vec<`TA`> to match a provided `size`.
/// 
/// # Arguments
/// * `size` - Number of records in output data.
/// * `constant` - Value to impute with.
/// 
/// # Generics
/// * `MI` - Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
/// * `MO` - Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
/// * `TA` - Atomic type. If not passed, TA is inferred from the lower bound
/// 
/// # Returns
/// A vector of the same type `TA`, but with the provided `size`.
fn make_resize<MI, MO, TA>(
    size: usize,
    constant: TA,
) -> Fallible<Transformation<VectorDomain<AllDomain<TA>>, SizedDomain<VectorDomain<AllDomain<TA>>>, MI, MO>>
where
    MI: 'static + IsMetricOrdered<Distance=IntDistance>,
    MO: 'static + IsMetricOrdered<Distance=IntDistance>,
    TA: 'static + Clone + CheckNull,
{
    let atom_domain = AllDomain::new();
    super::make_resize(size, atom_domain, constant)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_resize(
    size: c_uint, constant: *const AnyObject,
    MI: *const c_char,
    MO: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, MO, TA>(
        size: usize, constant: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where 
            MI: 'static + IsMetricOrdered<Distance=IntDistance>,
            MO: 'static + IsMetricOrdered<Distance=IntDistance>,
            TA: 'static + Clone + CheckNull, {
        let constant = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
        make_resize::<MI, MO, TA>(size, constant).into_any()
    }
    let size = size as usize;
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (MO, [SymmetricDistance, InsertDeleteDistance]),
        (TA, @numbers)
    ], (size, constant))
}

#[bootstrap(
    features("contrib"),
    arguments(constant(c_type = "void *")),
    generics(
        MI(default = "SymmetricDistance"),
        MO(default = "SymmetricDistance"),
        TA(example(get_first("bounds")))
    )
)]
/// Make a Transformation that either truncates or imputes records 
/// with `constant` in a Vec<`TA`> to match a provided `size`.
/// 
/// # Arguments
/// * `size` - Number of records in output data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
/// * `constant` - Value to impute with.
/// 
/// # Generics
/// * `MI` - Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
/// * `MO` - Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
/// * `TA` - Atomic type. If not passed, TA is inferred from the lower bound
/// 
/// # Returns
/// A vector of the same type `TA`, but with the provided `size`.
pub fn make_bounded_resize<MI, MO, TA>(
    size: usize,
    bounds: (TA, TA),
    constant: TA,
) -> Fallible<Transformation<VectorDomain<BoundedDomain<TA>>, SizedDomain<VectorDomain<BoundedDomain<TA>>>, MI, MO>>
where
    TA: 'static + Clone + CheckNull + TotalOrd,
    MI: 'static + IsMetricOrdered<Distance=IntDistance>,
    MO: 'static + IsMetricOrdered<Distance=IntDistance>,
{
    let atom_domain = BoundedDomain::new_closed(bounds)?;
    super::make_resize(size, atom_domain, constant)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_bounded_resize(
    size: c_uint, bounds: *const AnyObject,
    constant: *const c_void,
    MI: *const c_char,
    MO: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, MO, TA>(
        size: usize, bounds: *const AnyObject,
        constant: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
        where 
            TA: 'static + Clone + CheckNull + TotalOrd,
            MI: 'static + IsMetricOrdered<Distance=IntDistance>,
            MO: 'static + IsMetricOrdered<Distance=IntDistance>, {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>()).clone();
        let constant = try_as_ref!(constant as *const TA).clone();
        make_bounded_resize::<MI, MO, TA>(size, bounds, constant).into_any()
    }
    let size = size as usize;
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (MO, [SymmetricDistance, InsertDeleteDistance]),
        (TA, @numbers)
    ], (size, bounds, constant))
}


#[cfg(test)]
mod tests {
    use crate::core::opendp_core__transformation_invoke;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_resize(
            4 as c_uint,
            AnyObject::new_raw(0i32),
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }


    #[test]
    fn test_make_bounded_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_bounded_resize(
            4 as c_uint,
            util::into_raw(AnyObject::new((0i32, 10))),
            util::into_raw(0i32) as *const c_void,
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}
