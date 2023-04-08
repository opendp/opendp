use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::metrics::SymmetricDistance;
use crate::traits::{CheckAtom, TotalOrd};
use crate::transformations::{make_clamp, DatasetMetric};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_clamp(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    bounds: *const AnyObject,
    TA: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));
    let M = try_!(Type::try_from(M));

    fn monomorphize_dataset<TA, M>(
        input_domain: *const AnyDomain,
        input_metric: *const AnyMetric,
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TA: 'static + Clone + TotalOrd + CheckAtom,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
    {
        let input_domain =
            try_!(try_as_ref!(input_domain).downcast_ref::<VectorDomain<AtomDomain<TA>>>()).clone();
        let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<M>()).clone();
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>()).clone();
        make_clamp::<TA, M>(input_domain, input_metric, bounds).into_any()
    }
    dispatch!(monomorphize_dataset, [
        (TA, @numbers),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, bounds))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_vector_clamp() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_clamp(
            util::into_raw(AnyDomain::new(VectorDomain::new(
                AtomDomain::<f64>::default(),
                None,
            ))),
            util::into_raw(AnyMetric::new(SymmetricDistance::default())),
            util::into_raw(AnyObject::new((0.0, 10.0))),
            "f64".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }
}
