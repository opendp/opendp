use std::convert::TryFrom;
use std::ffi::c_void;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure, Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::{Type, as_ref};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{DiscreteGaussian, MakeNoise, NoiseDomain, make_gaussian};
use crate::measures::{Approximate, ZeroConcentratedDivergence};
use crate::metrics::{AbsoluteDistance, L2Distance};
use crate::traits::Number;

trait GaussianDomain<Q>: NoiseDomain {
    type Metric: Metric;
}

impl<T: Number, Q: Number> GaussianDomain<Q> for AtomDomain<T> {
    type Metric = AbsoluteDistance<Q>;
}
impl<T: Number, Q: Number> GaussianDomain<Q> for VectorDomain<AtomDomain<T>> {
    type Metric = L2Distance<Q>;
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_gaussian(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    radius: *const c_void,
    k: *const i32,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure, T: 'static + Number + Nature, QI: 'static + Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        radius: *const c_void,
        k: Option<i32>,
        MO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T::RV<2>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
            + MakeNoise<VectorDomain<AtomDomain<T>>, L2Distance<QI>, MO>,
    {
        fn monomorphize2<DI: 'static + GaussianDomain<QI>, MO: 'static + Measure, QI: Number>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: f64,
            radius: Option<DI::Atom>,
            k: Option<i32>,
        ) -> Fallible<AnyMeasurement>
        where
            DiscreteGaussian<DI::Atom>: MakeNoise<DI, DI::Metric, MO>,
            (DI, DI::Metric): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<DI>()?.clone();
            let input_metric = input_metric.downcast_ref::<DI::Metric>()?.clone();
            make_gaussian::<DI, DI::Metric, MO>(input_domain, input_metric, scale, radius, k)
                .into_any()
        }
        let radius = as_ref(radius as *const T).cloned();
        let DI = input_domain.type_.clone();
        let QI = input_metric.type_.get_atom()?;
        dispatch!(monomorphize2, [
            (DI, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (MO, [MO]),
            (QI, [QI])
        ], (input_domain, input_metric, scale, radius, k))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let k = as_ref(k as *const i32).map(Clone::clone);
    let MO = try_!(Type::try_from(MO));
    let QI_ = try_!(input_metric.type_.get_atom());
    let T_ = try_!(input_domain.type_.get_atom());

    if radius.is_null() {
        dispatch!(monomorphize, [
            (MO, [ZeroConcentratedDivergence]),
            (T_, @numbers),
            (QI_, @numbers)
        ], (input_domain, input_metric, scale, radius, k, MO))
    } else {
        let MO = Type::of::<Approximate<ZeroConcentratedDivergence>>();
        dispatch!(monomorphize, [
            (MO, [Approximate<ZeroConcentratedDivergence>]),
            (T_, @numbers),
            (QI_, @numbers)
        ], (input_domain, input_metric, scale, radius, k, MO))
    }
    .into()
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use super::*;
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;
    use crate::metrics::AbsoluteDistance;

    #[test]
    fn test_make_gaussian_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_gaussian(
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            0.0,
            null(),
            null(),
            "ZeroConcentratedDivergence".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
