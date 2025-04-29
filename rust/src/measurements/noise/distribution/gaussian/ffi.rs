use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{
    Domain, FfiResult, IntoAnyMeasurementFfiResultExt, Measure, Metric, MetricSpace,
};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::{Type, as_ref};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{DiscreteGaussian, MakeNoise, make_gaussian};
use crate::measures::ZeroConcentratedDivergence;
use crate::metrics::{AbsoluteDistance, L2Distance};
use crate::traits::Number;

trait GaussianMetric<T> {
    type Domain: Domain;
}

impl<T: Number, Q: Number> GaussianMetric<T> for AbsoluteDistance<Q> {
    type Domain = AtomDomain<T>;
}
impl<T: Number, Q: Number> GaussianMetric<T> for L2Distance<Q> {
    type Domain = VectorDomain<AtomDomain<T>>;
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_gaussian(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    k: *const i32,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure, T: 'static + Number + Nature, QI: 'static + Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        k: Option<i32>,
        MO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T::RV<2>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
            + MakeNoise<VectorDomain<AtomDomain<T>>, L2Distance<QI>, MO>,
    {
        fn monomorphize2<MI: 'static + Metric, MO: 'static + Measure, T: Number>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: f64,
            k: Option<i32>,
        ) -> Fallible<AnyMeasurement>
        where
            MI: GaussianMetric<T>,
            DiscreteGaussian: MakeNoise<MI::Domain, MI, MO>,
            (MI::Domain, MI): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<MI::Domain>()?.clone();
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_gaussian::<MI::Domain, MI, MO>(input_domain, input_metric, scale, k).into_any()
        }
        let T_ = input_domain.type_.get_atom()?;
        let MI = input_metric.type_.clone();
        dispatch!(monomorphize2, [
            (MI, [AbsoluteDistance<QI>, L2Distance<QI>]),
            (MO, [MO]),
            (T_, [T])
        ], (input_domain, input_metric, scale, k))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let k = as_ref(k as *const i32).map(Clone::clone);
    let MO = try_!(Type::try_from(MO));
    let QI_ = try_!(input_metric.type_.get_atom());
    let T_ = try_!(input_domain.type_.get_atom());

    dispatch!(monomorphize, [
        (MO, [ZeroConcentratedDivergence]),
        (T_, @numbers),
        (QI_, @numbers)
    ], (input_domain, input_metric, scale, k, MO))
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
            "ZeroConcentratedDivergence".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
