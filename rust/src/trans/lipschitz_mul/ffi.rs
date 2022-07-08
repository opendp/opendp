use std::{convert::TryFrom, ffi::c_void, os::raw::c_char};


use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    metrics::{AbsoluteDistance, L1Distance, L2Distance},
    domains::{AllDomain, VectorDomain},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    traits::SaturatingMul,
    trans::{make_lipschitz_float_mul, LipschitzMulFloatDomain, LipschitzMulFloatMetric, Float},
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_lipschitz_float_mul(
    constant: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        constant: *const c_void,
        bounds: *const AnyObject,
        D: Type,
        M: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float + SaturatingMul,
    {
        fn monomorphize2<D, M>(
            constant: D::Atom,
            bounds: (D::Atom, D::Atom),
        ) -> FfiResult<*mut AnyTransformation>
        where
            D: 'static + LipschitzMulFloatDomain,
            D::Atom: Float + SaturatingMul,
            M: 'static + LipschitzMulFloatMetric<Distance = D::Atom>,
        {
            make_lipschitz_float_mul::<D, M>(constant, bounds).into_any()
        }

        let constant = try_as_ref!(constant as *const T).clone();
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>());
        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (M, [AbsoluteDistance<T>, L1Distance<T>, L2Distance<T>])
        ], (constant, bounds.clone()))
    }

    let D = try_!(Type::try_from(D));
    let M = try_!(Type::try_from(M));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (constant, bounds, D, M))
}
