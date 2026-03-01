use std::ffi::c_void;

use dashu::integer::IBig;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::AtomDomain,
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::AbsoluteDistance,
    traits::{Float, SaturatingMul},
    transformations::make_lipschitz_float_mul,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_lipschitz_float_mul(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    constant: *const c_void,
    bounds: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        constant: *const c_void,
        bounds: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float + SaturatingMul,
        IBig: From<T::Bits>,
    {
        let input_domain = input_domain.downcast_ref::<AtomDomain<T>>()?.clone();
        let input_metric = input_metric.downcast_ref::<AbsoluteDistance<T>>()?.clone();
        let constant = *try_as_ref!(constant as *const T);
        let bounds = try_as_ref!(bounds).downcast_ref::<(T, T)>()?.clone();
        make_lipschitz_float_mul::<T>(input_domain, input_metric, constant, bounds).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (input_domain, input_metric, constant, bounds))
    .into()
}
