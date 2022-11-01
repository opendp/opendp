use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::measurements::{make_base_discrete_laplace, DiscreteLaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceLinear;
use crate::traits::{Float, InfCast, Integer};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace(
    scale: *const c_void,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    #[cfg(feature = "use-mpfr")]
    fn monomorphize<T, QO>(
        scale: *const c_void,
        D: Type,
        QO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<T> + InfCast<T>,
        rug::Rational: TryFrom<QO>,
        rug::Integer: From<T> + az::SaturatingCast<T>,
    {
        fn monomorphize2<D, QO>(scale: QO) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom> + InfCast<D::Atom>,
            rug::Rational: TryFrom<QO>,
            rug::Integer: From<D::Atom> + az::SaturatingCast<D::Atom>,
        {
            make_base_discrete_laplace::<D, QO>(scale).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (QO, [QO])
        ], (scale))
    }
    #[cfg(not(feature = "use-mpfr"))]
    fn monomorphize<T, QO>(
        scale: *const c_void,
        D: Type,
        QO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<T>,
    {
        fn monomorphize2<D, QO>(scale: QO) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom>,
        {
            make_base_discrete_laplace::<D, QO>(scale).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (QO, [QO])
        ], (scale))
    }
    let D = try_!(Type::try_from(D));
    let T = try_!(D.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (scale, D, QO))
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
    fn test_make_base_discrete_laplace_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace(
            util::into_raw(0.0) as *const c_void,
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
