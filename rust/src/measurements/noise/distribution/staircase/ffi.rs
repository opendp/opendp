use crate::core::{Domain, FfiResult, IntoAnyMeasurementFfiResultExt, Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::measurements::{
    L1Staircase, LinfStaircase, MakeNoise, make_l1_staircase, make_linf_staircase,
};
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance, LInfDistance};
use crate::traits::Number;

trait L1StaircaseMetric<T> {
    type Domain: Domain;
}

impl<T: Number, Q: Number> L1StaircaseMetric<T> for AbsoluteDistance<Q> {
    type Domain = AtomDomain<T>;
}

impl<T: Number, Q: Number> L1StaircaseMetric<T> for L1Distance<Q> {
    type Domain = VectorDomain<AtomDomain<T>>;
}

trait LinfStaircaseMetric<T> {
    type Domain: Domain;
}

impl<T: Number, Q: Number> LinfStaircaseMetric<T> for AbsoluteDistance<Q> {
    type Domain = AtomDomain<T>;
}

impl<T: Number> LinfStaircaseMetric<T> for LInfDistance<T> {
    type Domain = VectorDomain<AtomDomain<T>>;
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_l1_staircase(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    delta: f64,
    r: f64,
    epsilon: f64,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T: 'static + Number, QI: 'static + Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        delta: f64,
        r: f64,
        epsilon: f64,
    ) -> Fallible<AnyMeasurement>
    where
        L1Staircase: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MaxDivergence>
            + MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<QI>, MaxDivergence>,
    {
        fn monomorphize2<MI: 'static + Metric, T: Number>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            delta: f64,
            r: f64,
            epsilon: f64,
        ) -> Fallible<AnyMeasurement>
        where
            MI: L1StaircaseMetric<T>,
            L1Staircase: MakeNoise<MI::Domain, MI, MaxDivergence>,
            (MI::Domain, MI): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<MI::Domain>()?.clone();
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_l1_staircase::<MI::Domain, MI>(input_domain, input_metric, delta, r, epsilon)
                .into_any()
        }

        let T_ = input_domain.type_.get_atom()?;
        let MI = input_metric.type_.clone();
        dispatch!(
            monomorphize2,
            [(MI, [AbsoluteDistance<QI>, L1Distance<QI>]), (T_, [T])],
            (input_domain, input_metric, delta, r, epsilon)
        )
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T_ = try_!(input_domain.type_.get_atom());
    let QI_ = try_!(input_metric.type_.get_atom());

    dispatch!(
        monomorphize,
        [(T_, @numbers), (QI_, @numbers)],
        (input_domain, input_metric, delta, r, epsilon)
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_linf_staircase(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    delta: f64,
    r: f64,
    epsilon: f64,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T: 'static + Number, QI: 'static + Number>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        delta: f64,
        r: f64,
        epsilon: f64,
    ) -> Fallible<AnyMeasurement>
    where
        LinfStaircase: MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MaxDivergence>
            + MakeNoise<VectorDomain<AtomDomain<T>>, LInfDistance<T>, MaxDivergence>,
    {
        fn monomorphize2<MI: 'static + Metric, T: Number>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            delta: f64,
            r: f64,
            epsilon: f64,
        ) -> Fallible<AnyMeasurement>
        where
            MI: LinfStaircaseMetric<T>,
            LinfStaircase: MakeNoise<MI::Domain, MI, MaxDivergence>,
            (MI::Domain, MI): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<MI::Domain>()?.clone();
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_linf_staircase::<MI::Domain, MI>(input_domain, input_metric, delta, r, epsilon)
                .into_any()
        }

        let T_ = input_domain.type_.get_atom()?;
        let MI = input_metric.type_.clone();
        dispatch!(
            monomorphize2,
            [(MI, [AbsoluteDistance<QI>, LInfDistance<T>]), (T_, [T])],
            (input_domain, input_metric, delta, r, epsilon)
        )
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T_ = try_!(input_domain.type_.get_atom());
    let QI_ = try_!(input_metric.type_.get_atom());
    let QI_ = if QI_.id == input_metric.type_.id {
        T_.clone()
    } else {
        QI_
    };

    dispatch!(
        monomorphize,
        [(T_, @numbers), (QI_, @numbers)],
        (input_domain, input_metric, delta, r, epsilon)
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core;
    use crate::ffi::any::AnyObject;
    use crate::ffi::util;

    #[test]
    fn test_make_l1_staircase_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_l1_staircase(
            util::into_raw(AnyDomain::new(VectorDomain::new(
                AtomDomain::<f64>::new_non_nan(),
            ))),
            util::into_raw(AnyMetric::new(L1Distance::<f64>::default())),
            1.0,
            1.0,
            1.0,
        ))?;
        let arg = AnyObject::new_raw(vec![0.0, 1.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res.len(), 2);
        Ok(())
    }

    #[test]
    fn test_make_linf_staircase_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_linf_staircase(
            util::into_raw(AnyDomain::new(VectorDomain::new(
                AtomDomain::<i32>::default(),
            ))),
            util::into_raw(AnyMetric::new(LInfDistance::<i32>::default())),
            1.0,
            1.0,
            1.0,
        ))?;
        let arg = AnyObject::new_raw(vec![0, 1]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res.len(), 2);
        Ok(())
    }
}
