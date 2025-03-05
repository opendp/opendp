use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::metrics::IntDistance;
use crate::traits::{CheckAtom, InherentNull, RoundCast};
use crate::transformations::{make_cast, make_cast_default, make_cast_inherent, DatasetMetric};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_cast(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let M_ = input_metric.type_.clone();
    let TIA_ = try_!(input_domain.type_.get_atom());
    let TOA_ = try_!(Type::try_from(TOA));

    fn monomorphize<M, TIA, TOA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        M: 'static + DatasetMetric<Distance = IntDistance>,
        TIA: 'static + Clone + CheckAtom,
        TOA: 'static + RoundCast<TIA> + CheckAtom,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<OptionDomain<AtomDomain<TOA>>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        make_cast::<M, TIA, TOA>(input_domain, input_metric).into_any()
    }
    dispatch!(monomorphize, [
        (M_, @dataset_metrics),
        (TIA_, @primitives), 
        (TOA_, @primitives)
    ], (input_domain, input_metric))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_cast_default(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA_ = try_!(input_domain.type_.get_atom());
    let TOA_ = try_!(Type::try_from(TOA));
    let M_ = input_metric.type_.clone();

    fn monomorphize<M, TIA, TOA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        M: 'static + DatasetMetric,
        TIA: 'static + Clone + CheckAtom,
        TOA: 'static + RoundCast<TIA> + Default + CheckAtom,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        make_cast_default::<M, TIA, TOA>(input_domain, input_metric).into_any()
    }
    dispatch!(monomorphize, [
        (M_, @dataset_metrics),
        (TIA_, @primitives), 
        (TOA_, @primitives)
    ], (input_domain, input_metric))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_cast_inherent(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let M_ = input_metric.type_.clone();
    let TIA_ = try_!(input_domain.type_.get_atom());
    let TOA_ = try_!(Type::try_from(TOA));

    fn monomorphize<M, TIA, TOA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        M: 'static + DatasetMetric,
        TIA: 'static + Clone + CheckAtom,
        TOA: 'static + RoundCast<TIA> + InherentNull + CheckAtom,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        make_cast_inherent::<M, TIA, TOA>(input_domain, input_metric).into_any()
    }
    dispatch!(monomorphize, [
        (M_, @dataset_metrics),
        (TIA_, @primitives), 
        (TOA_, @floats)
    ], (input_domain, input_metric))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util::ToCharP;
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_cast() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_cast(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<i32>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<Option<f64>> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![Some(1.0), Some(2.0), Some(3.0)]);
        Ok(())
    }

    #[test]
    fn test_make_cast_default() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_cast_default(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<String>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "1".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0, 1]);
        Ok(())
    }

    #[test]
    fn test_make_cast_inherent() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_cast_inherent(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<String>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "1".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert!(res[0].is_nan());
        assert_eq!(res[1], 1.);
        Ok(())
    }
}
