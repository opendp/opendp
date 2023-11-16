use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, MetricSpace};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast};
use crate::{core::IntoAnyMeasurementFfiResultExt, ffi::util};
use crate::{
    domains::{AtomDomain, VectorDomain},
    ffi::util::Type,
    measurements::{make_base_discrete_laplace_linear, BaseDiscreteLaplaceDomain},
    traits::{samplers::SampleDiscreteLaplaceLinear, Float, InfCast, Integer},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace_linear(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    bounds: *const AnyObject,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        bounds: *const AnyObject,
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
            bounds: Option<(D::Atom, D::Atom)>,
        ) -> Fallible<AnyMeasurement>
        where
            D: 'static + BaseDiscreteLaplaceDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
            QO: Float + InfCast<D::Atom>,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_base_discrete_laplace_linear::<D, QO>(input_domain, input_metric, scale, bounds)
                .into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            Some(*bounds.downcast_ref::<(T, T)>()?)
        } else {
            None
        };
        let D = input_domain.type_.clone();
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale, bounds))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T = try_!(input_domain.type_.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, bounds, QO)).into()
}

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_geometric(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    bounds: *const AnyObject,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_measurements__make_base_discrete_laplace_linear(
        input_domain,
        input_metric,
        scale,
        bounds,
        QO,
    )
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
    fn test_make_base_discrete_laplace_linear_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_linear(
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            util::into_raw(0.0) as *const c_void,
            std::ptr::null(),
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
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            util::into_raw(0.0) as *const c_void,
            util::into_raw(AnyObject::new((0, 100))),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
