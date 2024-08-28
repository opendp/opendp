use std::any::TypeId;

use crate::core::MetricSpace;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::traits::{CheckAtom, InherentNull, Primitive};
use crate::transformations::{make_is_equal, make_is_null, DatasetMetric};

#[cfg(feature = "honest-but-curious")]
#[no_mangle]
pub extern "C" fn opendp_transformations__make_identity(
    domain: *const AnyDomain,
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let domain = try_as_ref!(domain).clone();
    let metric = try_as_ref!(metric).clone();
    super::make_identity(domain, metric).into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_is_equal(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    value: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let value = try_as_ref!(value);

    let TIA_ = try_!(input_domain.type_.get_atom());
    let M_ = input_metric.type_.clone();

    fn monomorphize<TIA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        value: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TIA: Primitive,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
    {
        let input_domain = try_as_ref!(input_domain)
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = try_as_ref!(input_metric).downcast_ref::<M>()?.clone();
        let value = try_as_ref!(value).downcast_ref::<TIA>()?.clone();
        make_is_equal::<TIA, M>(input_domain, input_metric, value).into_any()
    }
    dispatch!(monomorphize, [
        (TIA_, @primitives),
        (M_, @dataset_metrics)
    ], (input_domain, input_metric, value))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_is_null(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let M = input_metric.type_.clone();
    let DI = input_domain.type_.clone();
    let DIA = if let TypeContents::GENERIC {
        name: "VectorDomain",
        args,
    } = DI.contents
    {
        let type_id =
            try_!(<[TypeId; 1]>::try_from(args)
                .map_err(|_| err!(FFI, "Vec must have one type argument")))[0];
        try_!(Type::of_id(&type_id))
    } else {
        return err!(FFI, "Invalid type name.").into();
    };
    let TIA = try_!(DIA.get_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<M, TIA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
            ) -> Fallible<AnyTransformation>
            where
                TIA: 'static + CheckAtom,
                M: 'static + DatasetMetric,
                (VectorDomain<OptionDomain<AtomDomain<TIA>>>, M): MetricSpace,
                (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<OptionDomain<AtomDomain<TIA>>>>()?
                    .clone();
                let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
                make_is_null(input_domain, input_metric).into_any()
            }
            dispatch!(monomorphize, [(M, @dataset_metrics), (TIA, @primitives)], (input_domain, input_metric)).into()
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<M, TIA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
            ) -> Fallible<AnyTransformation>
            where
                TIA: 'static + CheckAtom + InherentNull,
                M: 'static + DatasetMetric,
                (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
                (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
                    .clone();
                let input_metric = input_metric.downcast_ref::<M>()?.clone();
                make_is_null(input_domain, input_metric).into_any()
            }
            dispatch!(monomorphize, [(M, @dataset_metrics), (TIA, [f64, f32])], (input_domain, input_metric)).into()
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionDomain<AtomDomain<T>> or an AtomDomain<T>"
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[cfg(feature = "honest-but-curious")]
    #[test]
    fn test_make_identity() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_identity(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<i32>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
        ))?;
        let arg = AnyObject::new_raw(vec![123]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![123]);
        Ok(())
    }

    #[test]
    fn test_make_is_equal() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_is_equal(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<i32>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            AnyObject::new_raw(1) as *const AnyObject,
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![true, false, false]);
        Ok(())
    }

    #[test]
    fn test_make_is_null() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_is_null(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<f64>::new_nullable())),
            AnyMetric::new_raw(SymmetricDistance::default()),
        ))?;
        let arg = AnyObject::new_raw(vec![1., 2., f64::NAN]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![false, false, true]);
        Ok(())
    }
}
