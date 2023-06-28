use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use az::SaturatingCast;
use rug::{Integer, Rational};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::measurements::{
    make_base_discrete_gaussian, BaseDiscreteGaussianDomain, DiscreteGaussianMeasure,
};
use crate::measures::ZeroConcentratedDivergence;
use crate::traits::{CheckAtom, Float, InfCast, Number};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_gaussian(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QI, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        D: Type,
        MO: Type,
        QI: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: 'static + CheckAtom + Clone,
        Integer: From<T> + SaturatingCast<T>,

        QI: Number,
        QO: Float + InfCast<QI>,
        Rational: TryFrom<QO>,
    {
        fn monomorphize2<D, MO, QI>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: MO::Atom,
        ) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + BaseDiscreteGaussianDomain<QI> + Send + Sync,
            D::Carrier: Send + Sync,
            D::InputMetric: Send + Sync,
            (D, D::InputMetric): MetricSpace,
            Integer: From<D::Atom> + SaturatingCast<D::Atom>,

            MO: 'static + DiscreteGaussianMeasure<D, QI> + Send + Sync,
            MO::Distance: Send + Sync,
            Rational: TryFrom<MO::Atom>,

            QI: Number,
        {
            let input_domain = try_!(input_domain.downcast_ref::<D>()).clone();
            let input_metric = try_!(input_metric.downcast_ref::<D::InputMetric>()).clone();
            make_base_discrete_gaussian::<D, MO, QI>(input_domain, input_metric, scale).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [VectorDomain<AtomDomain<T>>, AtomDomain<T>]),
            (MO, [ZeroConcentratedDivergence<QO>]),
            (QI, [QI])
        ], (input_domain, input_metric, scale))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let D = input_domain.type_.clone();
    let MO = try_!(Type::try_from(MO));
    let QI = input_metric.distance_type.clone();
    let T = try_!(D.get_atom());
    let QO = try_!(MO.get_atom());

    dispatch!(monomorphize, [
        (T, @integers),
        (QI, @numbers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, D, MO, QI))
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
    fn test_make_base_discrete_gaussian() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_gaussian(
            AnyDomain::new_raw(AtomDomain::<i32>::default()),
            AnyMetric::new_raw(AbsoluteDistance::<f64>::default()),
            util::into_raw(0.0) as *const c_void,
            "ZeroConcentratedDivergence<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
