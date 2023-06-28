use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use az::SaturatingCast;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace, Metric};
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::{
    domains::{AtomDomain, VectorDomain},
    ffi::util::Type,
    measurements::{make_base_discrete_laplace_cks20, BaseDiscreteLaplaceDomain},
    traits::InfCast,
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace_cks20(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        D: Type,
        QO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: crate::traits::Integer,
        QO: crate::traits::Float + InfCast<T>,
        rug::Rational: TryFrom<QO>,
        rug::Integer: From<T> + SaturatingCast<T>,
    {
        fn monomorphize2<D, QO>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: QO,
        ) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + BaseDiscreteLaplaceDomain + Send + Sync,
            D::Atom: crate::traits::Integer,
            (D, D::InputMetric): MetricSpace,
            QO: crate::traits::Float + InfCast<D::Atom>,
            D::InputMetric: Send + Sync,
            D::Carrier: Send + Sync,
            <D::InputMetric as Metric>::Distance: Send + Sync,
            rug::Rational: TryFrom<QO>,
            rug::Integer: From<D::Atom> + SaturatingCast<D::Atom>,
        {
            let input_domain = try_!(input_domain.downcast_ref::<D>()).clone();
            let input_metric = try_!(input_metric.downcast_ref::<D::InputMetric>()).clone();
            make_base_discrete_laplace_cks20::<D, QO>(input_domain, input_metric, scale).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale))
    }
    let input_metric = try_as_ref!(input_metric);
    let input_domain = try_as_ref!(input_domain);
    let D = input_domain.type_.clone();
    let T = try_!(D.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, D, QO))
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
    fn test_make_base_discrete_laplace_cks20_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_cks20(
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
