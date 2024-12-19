use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::Float;
use crate::transformations::{
    make_variance, LipschitzMulFloatDomain, LipschitzMulFloatMetric, Pairwise, Sequential,
    UncheckedSum,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_variance(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    ddof: c_uint,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        ddof: usize,
        S: Type,
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            ddof: usize,
        ) -> Fallible<AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
            AtomDomain<S::Item>: LipschitzMulFloatDomain<Atom = S::Item>,
            AbsoluteDistance<S::Item>: LipschitzMulFloatMetric<Distance = S::Item>,
        {
            let input_domain = input_domain
                .downcast_ref::<VectorDomain<AtomDomain<S::Item>>>()?
                .clone();
            let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
            make_variance::<S>(input_domain, input_metric, ddof).into_any()
        }
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (input_domain, input_metric, ddof))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let ddof = ddof as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (input_domain, input_metric, ddof, S))
    .into()
}
