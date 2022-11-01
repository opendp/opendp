use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, MetricSpace};
use crate::ffi::any::{AnyMeasurement, AnyObject};
use crate::{
    core::IntoAnyMeasurementFfiResultExt,
    ffi::{any::Downcast, util},
};
use crate::{
    domains::{AllDomain, VectorDomain},
    ffi::util::Type,
    measurements::{make_base_discrete_laplace_linear, DiscreteLaplaceDomain},
    traits::{samplers::SampleDiscreteLaplaceLinear, Float, InfCast, Integer},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace_linear(
    scale: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        scale: *const c_void,
        bounds: *const AnyObject,
        D: Type,
        QO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<T>,
    {
        fn monomorphize2<D, QO>(
            scale: QO,
            bounds: Option<(D::Atom, D::Atom)>,
        ) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom>,
        {
            make_base_discrete_laplace_linear::<D, QO>(scale, bounds).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            Some(*try_!(bounds.downcast_ref::<(T, T)>()))
        } else {
            None
        };
        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (QO, [QO])
        ], (scale, bounds))
    }
    let D = try_!(Type::try_from(D));
    let T = try_!(D.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (scale, bounds, D, QO))
}

#[deprecated(
    since = "0.5.0",
    note = "Use `opendp_measurements__make_base_discrete_laplace` instead. For a constant-time algorithm, pass bounds into `opendp_measurements__make_base_discrete_laplace_linear`."
)]
#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_geometric(
    scale: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_measurements__make_base_discrete_laplace_linear(scale, bounds, D, QO)
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
    fn test_make_base_discrete_laplace_linear_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_linear(
            util::into_raw(0.0) as *const c_void,
            std::ptr::null(),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }

    #[test]
    fn test_constant_time_make_base_discrete_laplace_linear_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_linear(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(AnyObject::new((0, 100))),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
