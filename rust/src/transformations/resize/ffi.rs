use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyObject, AnyTransformation};
use crate::ffi::any::{AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::metrics::{InsertDeleteDistance, IntDistance, SymmetricDistance};
use crate::traits::CheckAtom;
use crate::transformations::resize::IsMetricOrdered;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_resize(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    size: c_uint,
    constant: *const AnyObject,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let constant = try_as_ref!(constant);
    let T = try_!(input_domain.type_.get_atom());
    let MI_ = input_metric.type_.clone();
    let MO_ = try_!(Type::try_from(MO));

    fn monomorphize_all<MI, MO, TA: 'static + CheckAtom + Clone>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        size: usize,
        constant: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        MI: 'static + IsMetricOrdered<Distance = IntDistance>,
        MO: 'static + IsMetricOrdered<Distance = IntDistance>,
        (VectorDomain<AtomDomain<TA>>, MI): MetricSpace,
        (VectorDomain<AtomDomain<TA>>, MO): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        let constant = constant.downcast_ref::<TA>()?.clone();
        super::make_resize::<_, MI, MO>(input_domain, input_metric, size, constant).into_any()
    }

    dispatch!(monomorphize_all, [
        (MI_, [SymmetricDistance, InsertDeleteDistance]),
        (MO_, [SymmetricDistance, InsertDeleteDistance]),
        (T, @primitives)
    ], (input_domain, input_metric, size, constant))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core::opendp_core__transformation_invoke;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_resize(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<i32>::default())),
            AnyMetric::new_raw(SymmetricDistance::default()),
            4 as c_uint,
            AnyObject::new_raw(0i32),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }

    #[test]
    fn test_make_bounded_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_resize(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<i32>::new_closed((
                0i32, 10,
            ))?)),
            AnyMetric::new_raw(SymmetricDistance::default()),
            4 as c_uint,
            AnyObject::new_raw(0i32),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}
