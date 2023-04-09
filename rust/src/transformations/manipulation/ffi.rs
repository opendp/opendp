use std::convert::TryFrom;
use std::os::raw::c_char;

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Domain, Metric, MetricSpace, Transformation};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::metrics::{AbsoluteDistance, InsertDeleteDistance, IntDistance, SymmetricDistance};
use crate::traits::{CheckAtom, DistanceConstant, InherentNull, Primitive};
use crate::transformations::{make_is_equal, make_is_null, DatasetMetric};

#[bootstrap(features("contrib"))]
/// Make a Transformation representing the identity function.
///
/// # Generics
/// * `D` - Domain of the identity function. Must be `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `M` - Metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AtomDomain
fn make_identity<D, M>() -> Fallible<Transformation<D, D, M, M>>
where
    D: Domain + Default,
    D::Carrier: Clone,
    M: Metric,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
    (D, M): MetricSpace,
{
    super::make_identity(Default::default(), Default::default())
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_identity(
    D: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let D = try_!(Type::try_from(D));

    match &D.contents {
        TypeContents::GENERIC { name, args } if name == &"VectorDomain" => {
            if args.len() != 1 {
                return err!(FFI, "VectorDomain only accepts one argument.").into();
            }
            let atomic_domain = try_!(Type::of_id(&args[0]));
            let T = match atomic_domain.contents {
                TypeContents::GENERIC { name, args } if name == "AtomDomain" => {
                    if args.len() != 1 {
                        return err!(FFI, "AtomDomain only accepts one argument.").into();
                    }
                    try_!(Type::of_id(&args[0]))
                }
                _ => return err!(FFI, "In FFI, make_identity's VectorDomain may only contain AtomDomain<_>").into()
            };
            fn monomorphize<M, T>() -> FfiResult<*mut AnyTransformation>
                where M: 'static + Metric<Distance=IntDistance>,
                      T: 'static + Clone + CheckAtom,
                      (VectorDomain<AtomDomain<T>>, M): MetricSpace {
                make_identity::<VectorDomain<AtomDomain<T>>, M>().into_any()
            }
            dispatch!(monomorphize, [
                (M, [InsertDeleteDistance, SymmetricDistance]),
                (T, @primitives)
            ], ())
        }
        TypeContents::GENERIC { name, args } if name == &"AtomDomain" => {
            if args.len() != 1 {
                return err!(FFI, "AtomDomain only accepts one argument.").into();
            }
            let T = try_!(Type::of_id(&args[0]));

            fn monomorphize<T>(M: Type) -> FfiResult<*mut AnyTransformation>
                where T: 'static + DistanceConstant<T> + CheckAtom + One + Clone {
                fn monomorphize<M>() -> FfiResult<*mut AnyTransformation>
                    where M: 'static + Metric,
                          M::Distance: CheckAtom + DistanceConstant<M::Distance> + One + Clone,
                          (AtomDomain<M::Distance>, M): MetricSpace {
                    make_identity::<AtomDomain<M::Distance>, M>().into_any()
                }
                dispatch!(monomorphize, [
                    (M, [AbsoluteDistance<T>])
                ], ())
            }
            dispatch!(monomorphize, [
                (T, @numbers)
            ], (M))
        }
        _ => err!(FFI, "Monomorphizations for the identity function are only available for VectorDomain<AtomDomain<_>> and AtomDomain<_>").into()
    }
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
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_identity() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_identity(
            "VectorDomain<AtomDomain<i32>>".to_char_p(),
            "SymmetricDistance".to_char_p(),
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
            util::into_raw(AnyDomain::new(VectorDomain::new(
                AtomDomain::<i32>::default(),
                None,
            ))),
            util::into_raw(AnyMetric::new(SymmetricDistance::default())),
            util::into_raw(AnyObject::new(1)) as *const AnyObject,
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![true, false, false]);
        Ok(())
    }
}
