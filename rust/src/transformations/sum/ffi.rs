use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace};

use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast};
use crate::metrics::{InsertDeleteDistance, SymmetricDistance};
use crate::transformations::make_sum;
use crate::transformations::sum::MakeSum;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sum(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        MI: 'static + Metric,
        T: 'static + MakeSum<MI>,
        (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<T>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        make_sum::<MI, T>(input_domain, input_metric).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI = input_metric.type_.clone();
    let T = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (T, @numbers)
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
    fn test_make_sum_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_sum(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::new_closed((0., 10.))?)),
            AnyMetric::new_raw(SymmetricDistance::default()),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }

    #[test]
    fn test_make_sized_sum_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_sum(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(3)),
            AnyMetric::new_raw(SymmetricDistance::default()),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }
}
