use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::traits::{CheckAtom, ProductOrd};
use crate::transformations::{make_clamp, DatasetMetric};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_clamp(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    bounds: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let bounds = try_as_ref!(bounds);
    let TA_ = try_!(input_domain.type_.get_atom());
    let M_ = input_metric.type_.clone();

    fn monomorphize_dataset<TA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        bounds: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TA: 'static + Clone + ProductOrd + CheckAtom,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
    {
        let input_domain =
            try_!(input_domain.downcast_ref::<VectorDomain<AtomDomain<TA>>>()).clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let bounds = bounds.downcast_ref::<(TA, TA)>()?.clone();
        make_clamp::<TA, M>(input_domain, input_metric, bounds).into_any()
    }
    dispatch!(monomorphize_dataset, [
        (TA_, @numbers),
        (M_, @dataset_metrics)
    ], (input_domain, input_metric, bounds))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_vector_clamp() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_clamp(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<f64>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            AnyObject::new_raw((0.0, 10.0)),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }
}
