use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::measurements::{make_base_discrete_laplace, BaseDiscreteLaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceLinear;
use crate::traits::{Float, InfCast, Integer};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    #[cfg(feature = "use-mpfr")]
    fn monomorphize<T, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        QO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<T> + InfCast<T>,
        rug::Rational: TryFrom<QO>,
        rug::Integer: From<T> + az::SaturatingCast<T>,
    {
        fn monomorphize2<D, QO>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: QO,
        ) -> Fallible<AnyMeasurement>
        where
            D: 'static + BaseDiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom> + InfCast<D::Atom>,
            rug::Rational: TryFrom<QO>,
            rug::Integer: From<D::Atom> + az::SaturatingCast<D::Atom>,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_base_discrete_laplace::<D, QO>(input_domain, input_metric, scale).into_any()
        }
        let D = input_domain.type_.clone();
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale))
    }
    #[cfg(not(feature = "use-mpfr"))]
    fn monomorphize<T, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        QO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<T>,
    {
        fn monomorphize2<D, QO>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: QO,
        ) -> Fallible<AnyMeasurement>
        where
            D: 'static + BaseDiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom>,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_base_discrete_laplace::<D, QO>(input_domain, input_metric, scale).into_any()
        }
        let D = input_domain.type_.clone();
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T = try_!(input_domain.type_.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, QO))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;
    use crate::metrics::AbsoluteDistance;

    use super::*;

    #[test]
    fn test_make_base_discrete_laplace_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace(
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            util::into_raw(0.0) as *const c_void,
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
