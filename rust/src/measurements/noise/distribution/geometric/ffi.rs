use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure, Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast};
use crate::ffi::util::{Type, as_ref};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{
    ConstantTimeGeometric, DiscreteLaplace, MakeNoise, NoiseDomain, make_geometric,
};
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance};
use crate::traits::{Integer, Number};

trait GeometricDomain<Q>: NoiseDomain {
    type Metric: Metric;
}

impl<T: Number, Q: Number> GeometricDomain<Q> for AtomDomain<T> {
    type Metric = AbsoluteDistance<Q>;
}
impl<T: Number, Q: Number> GeometricDomain<Q> for VectorDomain<AtomDomain<T>> {
    type Metric = L1Distance<Q>;
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_geometric(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    bounds: *const AnyObject,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure, T: 'static + Integer + Nature, QI: 'static + Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        bounds: *const AnyObject,
        MO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T::RV<1>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
            + MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<QI>, MO>,
        ConstantTimeGeometric<T>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
            + MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<QI>, MO>,
    {
        fn monomorphize2<DI: 'static + GeometricDomain<QI>, MO: 'static + Measure, QI: Number>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: f64,
            bounds: Option<(DI::Atom, DI::Atom)>,
        ) -> Fallible<AnyMeasurement>
        where
            DiscreteLaplace<DI::Atom>: MakeNoise<DI, DI::Metric, MO>,
            ConstantTimeGeometric<DI::Atom>: MakeNoise<DI, DI::Metric, MO>,
            (DI, DI::Metric): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<DI>()?.clone();
            let input_metric = input_metric.downcast_ref::<DI::Metric>()?.clone();
            make_geometric::<DI, DI::Metric, MO>(input_domain, input_metric, scale, bounds)
                .into_any()
        }
        let DI = input_domain.type_.clone();
        let QI = input_metric.type_.get_atom()?;
        let bounds = if let Some(bounds) = as_ref(bounds) {
            Some(try_!(bounds.downcast_ref::<(T, T)>()).clone())
        } else {
            None
        };
        dispatch!(monomorphize2, [
            (DI, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (MO, [MO]),
            (QI, [QI])
        ], (input_domain, input_metric, scale, bounds))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MO = try_!(Type::try_from(MO));
    let QI_ = try_!(input_metric.type_.get_atom());
    let T_ = try_!(input_domain.type_.get_atom());

    dispatch!(monomorphize, [
        (MO, [MaxDivergence]),
        (T_, @integers),
        (QI_, @integers)
    ], (input_domain, input_metric, scale, bounds, MO))
    .into()
}
