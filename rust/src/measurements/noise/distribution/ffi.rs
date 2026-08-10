use crate::core::{Domain, FfiResult, IntoAnyMeasurementFfiResultExt, Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::measurements::noise::nature::Nature;
use crate::measurements::{
    DiscreteGaussian, DiscreteLaplace, MakeNoise, make_gaussian, make_laplace,
};
use crate::measures::{MultiDP, PureDP, zCDP};
use crate::metrics::{AbsoluteDistance, L1Distance, L2Distance};
use crate::traits::Number;

trait LaplaceMetric<T> {
    type Domain: Domain;
}

impl<T: Number, Q: Number> LaplaceMetric<T> for AbsoluteDistance<Q> {
    type Domain = AtomDomain<T>;
}
impl<T: Number, Q: Number> LaplaceMetric<T> for L1Distance<Q> {
    type Domain = VectorDomain<AtomDomain<T>>;
}

trait GaussianMetric<T> {
    type Domain: Domain;
}

impl<T: Number, Q: Number> GaussianMetric<T> for AbsoluteDistance<Q> {
    type Domain = AtomDomain<T>;
}
impl<T: Number, Q: Number> GaussianMetric<T> for L2Distance<Q> {
    type Domain = VectorDomain<AtomDomain<T>>;
}

fn monomorphize_laplace<T: 'static + Number + Nature, QI: 'static + Number>(
    input_domain: &AnyDomain,
    input_metric: &AnyMetric,
    scale: f64,
    k: Option<i32>,
) -> Fallible<AnyMeasurement>
where
    T::RV<1>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MultiDP>
        + MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<QI>, MultiDP>,
{
    fn monomorphize_metric<MI: 'static + Metric, T: Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<AnyMeasurement>
    where
        MI: LaplaceMetric<T>,
        DiscreteLaplace: MakeNoise<MI::Domain, MI, MultiDP>,
        (MI::Domain, MI): MetricSpace,
    {
        let input_domain = input_domain.downcast_ref::<MI::Domain>()?.clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        make_laplace::<MI::Domain, MI, MultiDP>(input_domain, input_metric, scale, k).into_any()
    }

    let T_ = input_domain.type_.get_atom()?;
    let MI = input_metric.type_.clone();
    dispatch!(
        monomorphize_metric,
        [(MI, [AbsoluteDistance<QI>, L1Distance<QI>]), (T_, [T])],
        (input_domain, input_metric, scale, k)
    )
}

fn monomorphize_gaussian<T: 'static + Number + Nature, QI: 'static + Number>(
    input_domain: &AnyDomain,
    input_metric: &AnyMetric,
    scale: f64,
    k: Option<i32>,
) -> Fallible<AnyMeasurement>
where
    T::RV<2>: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MultiDP>
        + MakeNoise<VectorDomain<AtomDomain<T>>, L2Distance<QI>, MultiDP>,
{
    fn monomorphize_metric<MI: 'static + Metric, T: Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<AnyMeasurement>
    where
        MI: GaussianMetric<T>,
        DiscreteGaussian: MakeNoise<MI::Domain, MI, MultiDP>,
        (MI::Domain, MI): MetricSpace,
    {
        let input_domain = input_domain.downcast_ref::<MI::Domain>()?.clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        make_gaussian::<MI::Domain, MI, MultiDP>(input_domain, input_metric, scale, k).into_any()
    }

    let T_ = input_domain.type_.get_atom()?;
    let MI = input_metric.type_.clone();
    dispatch!(
        monomorphize_metric,
        [(MI, [AbsoluteDistance<QI>, L2Distance<QI>]), (T_, [T])],
        (input_domain, input_metric, scale, k)
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    privacy_measure: *const AnyMeasure,
    scale: f64,
    k: *const i32,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let privacy_measure = try_as_ref!(privacy_measure);
    let k = crate::ffi::util::as_ref(k as *const i32).map(Clone::clone);
    let T_ = try_!(input_domain.type_.get_atom());
    let QI_ = try_!(input_metric.type_.get_atom());

    if privacy_measure.type_ == Type::of::<PureDP>() {
        dispatch!(
            monomorphize_laplace,
            [(T_, @numbers), (QI_, @numbers)],
            (input_domain, input_metric, scale, k)
        )
        .into()
    } else if privacy_measure.type_ == Type::of::<zCDP>() {
        dispatch!(
            monomorphize_gaussian,
            [(T_, @numbers), (QI_, @numbers)],
            (input_domain, input_metric, scale, k)
        )
        .into()
    } else {
        err!(FFI, "privacy_measure must be PureDP or zCDP").into()
    }
}
