use std::{convert::TryFrom, ffi::c_void, os::raw::c_char};

use dashu::integer::IBig;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace},
    domains::AtomDomain,
    error::Fallible,
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    metrics::AbsoluteDistance,
    traits::Float,
    traits::{FloatBits, SaturatingMul},
    transformations::{make_lipschitz_float_mul, LipschitzMulFloatDomain, LipschitzMulFloatMetric},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_lipschitz_float_mul(
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
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float + SaturatingMul,
        IBig: From<T::Bits>,
    {
        fn monomorphize2<D, M>(
            constant: D::Atom,
            bounds: (D::Atom, D::Atom),
        ) -> Fallible<AnyTransformation>
        where
            D: 'static + LipschitzMulFloatDomain,
            D::Atom: Float + SaturatingMul,
            M: 'static + LipschitzMulFloatMetric<Distance = D::Atom>,
            (D, M): MetricSpace,
            IBig: From<<D::Atom as FloatBits>::Bits>,
        {
            make_lipschitz_float_mul::<D, M>(constant, bounds).into_any()
        }

        let constant = *try_as_ref!(constant as *const T);
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>());
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>]),
            (M, [AbsoluteDistance<T>])
        ], (constant, *bounds))
    }

    let D = try_!(Type::try_from(D));
    let M = try_!(Type::try_from(M));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (constant, bounds, D, M))
    .into()
}
