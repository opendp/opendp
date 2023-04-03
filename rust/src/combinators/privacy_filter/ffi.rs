#[cfg(feature = "polars")]
use crate::{ffi::util::Type, metrics::Bounds};

use std::fmt::Debug;

use crate::{
    core::{
        AnyOdometerQuery, FfiResult, Function, Measurement, Odometer, OdometerAnswer,
        OdometerQuery, PrivacyMap,
    },
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasurement, AnyObject, AnyOdometer, Downcast},
    interactive::{Answer, Query, Queryable},
    measures::ffi::TypedMeasure,
    metrics::ffi::TypedMetric,
    traits::ProductOrd,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_privacy_filter(
    odometer: *const AnyOdometer,
    d_in: *const AnyObject,
    d_out: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let odometer = try_as_ref!(odometer).clone();
    let d_in = try_as_ref!(d_in).clone();
    let d_out = try_as_ref!(d_out).clone();

    fn monomorphize<QI, QO>(
        odometer: AnyOdometer,
        d_in: AnyObject,
        d_out: AnyObject,
    ) -> Fallible<AnyMeasurement>
    where
        QI: 'static + Clone + Debug + ProductOrd + Send + Sync,
        QO: 'static + Clone + Debug + ProductOrd + Send + Sync,
    {
        let function = odometer.function.clone();
        // Since the interior filter measurement has typed distances,
        // privacy loss answers from the odometer should be typed as well
        let odometer = Odometer::new(
            odometer.input_domain.clone(),
            TypedMetric::<QI>::new(odometer.input_metric.clone())?,
            TypedMeasure::<QO>::new(odometer.output_measure.clone())?,
            Function::new_fallible(move |arg: &AnyObject| {
                let mut qbl = function.eval(arg)?;
                Ok(Queryable::new_raw(
                    move |_, query: Query<OdometerQuery<AnyObject, QI>>| {
                        let answer = match query {
                            Query::External(OdometerQuery::Invoke(arg)) => qbl
                                .eval_query(Query::External(&OdometerQuery::Invoke(arg.clone())))?,
                            Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                                qbl.eval_query(Query::External(&OdometerQuery::PrivacyLoss(
                                    AnyObject::new(d_in.clone()),
                                )))?
                            }
                            Query::Internal(any) => qbl.eval_query(Query::Internal(any))?,
                        };
                        Ok(match answer {
                            Answer::External(OdometerAnswer::Invoke(answer)) => {
                                Answer::External(OdometerAnswer::Invoke(answer))
                            }
                            Answer::External(OdometerAnswer::PrivacyLoss(loss)) => {
                                Answer::External(OdometerAnswer::PrivacyLoss(
                                    loss.downcast::<QO>()?,
                                ))
                            }
                            Answer::Internal(answer) => Answer::Internal(answer),
                        })
                    },
                ))
            }),
        )?;
        let filter = super::make_privacy_filter::<
            AnyDomain,
            TypedMetric<QI>,
            TypedMeasure<QO>,
            AnyObject,
            AnyObject,
        >(odometer, d_in.downcast::<QI>()?, d_out.downcast::<QO>()?)?;

        // while this is a valid filter measurement,
        // it has typed distances that needs to be erased for FFI

        // 1. Measurement<AnyDomain, Queryable<AnyOdometerQuery, OdometerAnswer<AnyObject, QO>>, TypedMetric<QI>, TypedMeasure<QO>>
        //    -> is converted to ->
        // 2. Measurement<AnyDomain, AnyObject, AnyMetric, AnyMeasure>
        //    -> which is by definition ->
        // 3. AnyMeasurement

        let function = filter.function.clone();
        let privacy_map = filter.privacy_map.clone();
        Measurement::new(
            filter.input_domain.clone(),
            Function::new_fallible(move |arg: &AnyObject| {
                let mut qbl = function.eval(arg)?;
                Ok(AnyObject::new(Queryable::new_raw(
                    move |_, query: Query<AnyOdometerQuery>| {
                        let answer = match query {
                            Query::External(OdometerQuery::Invoke(arg)) => qbl
                                .eval_query(Query::External(&OdometerQuery::Invoke(arg.clone())))?,
                            Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                                qbl.eval_query(Query::External(&OdometerQuery::PrivacyLoss(
                                    d_in.clone().downcast::<QI>()?,
                                )))?
                            }
                            Query::Internal(any) => qbl.eval_query(Query::Internal(any))?,
                        };
                        Ok(match answer {
                            Answer::External(OdometerAnswer::Invoke(answer)) => {
                                Answer::External(OdometerAnswer::Invoke(answer))
                            }
                            Answer::External(OdometerAnswer::PrivacyLoss(loss)) => {
                                Answer::External(OdometerAnswer::PrivacyLoss(AnyObject::new(loss)))
                            }
                            Answer::Internal(answer) => Answer::Internal(answer),
                        })
                    },
                )))
            }),
            filter.input_metric.metric.clone(),
            filter.output_measure.measure.clone(),
            PrivacyMap::new_fallible(move |arg: &AnyObject| {
                Ok(AnyObject::new(privacy_map.eval(arg.downcast_ref::<QI>()?)?))
            }),
        )
    }

    let QI = odometer.input_metric.distance_type.clone();
    let QO = odometer.output_measure.distance_type.clone();

    #[cfg(feature = "polars")]
    if QI == Type::of::<Bounds>() {
        return dispatch!(
            monomorphize,
            [(QI, [Bounds]), (QO, [f64, (f64, f64)])],
            (odometer, d_in, d_out)
        )
        .into();
    }
    dispatch!(monomorphize, [
        (QI, @numbers),
        (QO, [f64, (f64, f64)])
    ], (odometer, d_in, d_out))
    .into()
}
