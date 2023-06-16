use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::MetricSpace;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::traits::{CheckAtom, InherentNull, Primitive};
use crate::transformations::{make_is_equal, make_is_null, DatasetMetric};

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

    let TIA = try_!(input_domain.type_.get_atom());
    let M = input_metric.type_.clone();

    fn monomorphize<TIA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        value: &AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: Primitive,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
    {
        let input_domain =
            try_!(try_as_ref!(input_domain).downcast_ref::<VectorDomain<AtomDomain<TIA>>>())
                .clone();
        let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<M>()).clone();
        let value = try_!(try_as_ref!(value).downcast_ref::<TIA>()).clone();
        make_is_equal::<TIA, M>(input_domain, input_metric, value).into_any()
    }
    dispatch!(monomorphize, [
        (TIA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, value))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_is_null(
    input_atom_domain: *const AnyDomain,
    DIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DIA = try_!(Type::try_from(DIA));
    let TIA = try_!(DIA.get_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<TIA>(
                input_atom_domain: *const AnyDomain,
            ) -> FfiResult<*mut AnyTransformation>
            where
                TIA: 'static + CheckAtom,
            {
                let input_atom_domain =
                    try_!(try_as_ref!(input_atom_domain)
                        .downcast_ref::<OptionDomain<AtomDomain<TIA>>>())
                    .clone();
                make_is_null(input_atom_domain).into_any()
            }
            dispatch!(monomorphize, [(TIA, @primitives)], (input_atom_domain))
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<TIA>(
                input_atom_domain: *const AnyDomain,
            ) -> FfiResult<*mut AnyTransformation>
            where
                TIA: 'static + CheckAtom + InherentNull,
            {
                let input_atom_domain =
                    try_!(try_as_ref!(input_atom_domain).downcast_ref::<AtomDomain<TIA>>()).clone();
                make_is_null::<AtomDomain<TIA>>(input_atom_domain).into_any()
            }
            dispatch!(monomorphize, [(TIA, [f64, f32])], (input_atom_domain))
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
}
