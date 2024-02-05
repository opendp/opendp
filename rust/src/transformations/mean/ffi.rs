use dashu::integer::IBig;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast};
use crate::metrics::{AbsoluteDistance, InsertDeleteDistance, SymmetricDistance};
use crate::traits::{ExactIntCast, Float, InfMul};
use crate::transformations::{
    make_mean, LipschitzMulFloatDomain, LipschitzMulFloatMetric, MakeSum,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_mean(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        MI: 'static + Metric,
        T: 'static + MakeSum<MI> + ExactIntCast<usize> + Float + InfMul,
        AtomDomain<T>: LipschitzMulFloatDomain<Atom = T>,
        AbsoluteDistance<T>: LipschitzMulFloatMetric<Distance = T>,
        (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
        IBig: From<T::Bits>,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<T>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        make_mean::<MI, T>(input_domain, input_metric).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI = input_metric.type_.clone();
    let T = try_!(input_domain.type_.get_atom());

    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (T, @floats)
    ], (input_domain, input_metric))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};

    use super::*;

    #[test]
    fn test_make_mean() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_mean(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(3)),
            AnyMetric::new_raw(InsertDeleteDistance::default()),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0);
        Ok(())
    }
}
