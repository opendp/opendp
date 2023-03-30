use std::convert::TryFrom;
use std::os::raw::c_char;

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Domain, Metric, Transformation};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AllDomain, InherentNullDomain, OptionNullDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::metrics::{
    AbsoluteDistance, ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance,
    L1Distance, L2Distance, SymmetricDistance,
};
use crate::traits::{CheckNull, DistanceConstant, InherentNull, Primitive};
use crate::transformations::{make_is_equal, make_is_null};

#[bootstrap(features("contrib"))]
/// Make a Transformation representing the identity function.
///
/// # Generics
/// * `D` - Domain of the identity function. Must be `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
/// * `M` - Metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AllDomain
fn make_identity<D, M>() -> Fallible<Transformation<D, D, M, M>>
where
    D: Domain + Default,
    D::Carrier: Clone,
    M: Metric,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
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
                TypeContents::GENERIC { name, args } if name == "AllDomain" => {
                    if args.len() != 1 {
                        return err!(FFI, "AllDomain only accepts one argument.").into();
                    }
                    try_!(Type::of_id(&args[0]))
                }
                _ => return err!(FFI, "In FFI, make_identity's VectorDomain may only contain AllDomain<_>").into()
            };
            fn monomorphize<M, T>() -> FfiResult<*mut AnyTransformation>
                where M: 'static + Metric<Distance=IntDistance>,
                      T: 'static + Clone + CheckNull {
                make_identity::<VectorDomain<AllDomain<T>>, M>().into_any()
            }
            dispatch!(monomorphize, [
                (M, [ChangeOneDistance, InsertDeleteDistance, SymmetricDistance, HammingDistance]),
                (T, @primitives)
            ], ())
        }
        TypeContents::GENERIC { name, args } if name == &"AllDomain" => {
            if args.len() != 1 {
                return err!(FFI, "AllDomain only accepts one argument.").into();
            }
            let T = try_!(Type::of_id(&args[0]));

            fn monomorphize<T>(M: Type) -> FfiResult<*mut AnyTransformation>
                where T: 'static + DistanceConstant<T> + CheckNull + One + Clone {
                fn monomorphize<M>() -> FfiResult<*mut AnyTransformation>
                    where M: 'static + Metric,
                          M::Distance: CheckNull + DistanceConstant<M::Distance> + One + Clone {
                    make_identity::<AllDomain<M::Distance>, M>().into_any()
                }
                dispatch!(monomorphize, [
                    (M, [AbsoluteDistance<T>, L1Distance<T>, L2Distance<T>])
                ], ())
            }
            dispatch!(monomorphize, [
                (T, @numbers)
            ], (M))
        }
        _ => err!(FFI, "Monomorphizations for the identity function are only available for VectorDomain<AllDomain<_>> and AllDomain<_>").into()
    }
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_is_equal(
    value: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TIA = try_!(Type::try_from(TIA));

    fn monomorphize<TIA>(value: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        TIA: Primitive,
    {
        let value: TIA = try_!(try_as_ref!(value).downcast_ref::<TIA>()).clone();
        make_is_equal::<TIA>(value).into_any()
    }
    dispatch!(monomorphize, [(TIA, @primitives)], (value))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_is_null(
    DIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DIA = try_!(Type::try_from(DIA));
    let TIA = try_!(DIA.get_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionNullDomain" => {
            fn monomorphize<TIA>() -> FfiResult<*mut AnyTransformation>
            where
                TIA: 'static + CheckNull,
            {
                make_is_null::<OptionNullDomain<AllDomain<TIA>>>().into_any()
            }
            dispatch!(monomorphize, [(TIA, @primitives)], ())
        }
        TypeContents::GENERIC { name, .. } if name == &"InherentNullDomain" => {
            fn monomorphize<TIA>() -> FfiResult<*mut AnyTransformation>
            where
                TIA: 'static + InherentNull,
            {
                make_is_null::<InherentNullDomain<AllDomain<TIA>>>().into_any()
            }
            dispatch!(monomorphize, [(TIA, [f64, f32])], ())
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>"
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
            "VectorDomain<AllDomain<i32>>".to_char_p(),
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
            util::into_raw(AnyObject::new(1)) as *const AnyObject,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![true, false, false]);
        Ok(())
    }
}
