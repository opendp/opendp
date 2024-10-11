use std::fmt::Debug;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
    interactive::{Answer, Query, Queryable},
    measures::ffi::TypedMeasure,
    metrics::ffi::TypedMetric,
    traits::ProductOrd,
};

fn make_sequential_composition(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
    d_in: AnyObject,
    d_mids: Vec<AnyObject>,
) -> Fallible<Measurement<AnyDomain, AnyObject, AnyMetric, AnyMeasure>> {
    fn monomorphize<
        QI: 'static + ProductOrd + Clone + Send + Sync,
        QO: 'static + ProductOrd + Clone + Send + Sync + Debug,
    >(
        input_domain: AnyDomain,
        input_metric: AnyMetric,
        output_measure: AnyMeasure,
        d_in: AnyObject,
        d_mids: Vec<AnyObject>,
    ) -> Fallible<AnyMeasurement> {
        let meas = super::make_sequential_composition::<
            AnyDomain,
            AnyObject,
            TypedMetric<QI>,
            TypedMeasure<QO>,
        >(
            input_domain,
            TypedMetric::<QI>::new(input_metric.clone())?,
            TypedMeasure::<QO>::new(output_measure.clone())?,
            d_in.downcast::<QI>()?,
            d_mids
                .into_iter()
                .map(|d| d.downcast::<QO>())
                .collect::<Fallible<Vec<_>>>()?,
        )?;
        let privacy_map = meas.privacy_map.clone();

        Ok(meas
            .with_map(
                input_metric,
                output_measure,
                PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                    Ok(AnyObject::new(
                        privacy_map.eval(d_in.downcast_ref::<QI>()?)?,
                    ))
                }),
            )?
            .into_any_queryable_map()?
            .into_any_Q()
            .into_any_out())
    }

    let QI = input_metric.distance_type.clone();
    let QO = output_measure.distance_type.clone();

    dispatch!(monomorphize, [
        (QI, @numbers),
        (QO, [f64, (f64, f64)])
    ], (input_domain, input_metric, output_measure, d_in, d_mids))
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_sequential_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
    d_mids: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();
    let d_in = try_as_ref!(d_in).clone();
    let d_mids = try_as_ref!(d_mids);

    fn repack_vec<T: 'static + Clone>(obj: &AnyObject) -> Fallible<Vec<AnyObject>> {
        Ok(obj
            .downcast_ref::<Vec<T>>()?
            .iter()
            .map(Clone::clone)
            .map(AnyObject::new)
            .collect())
    }

    let QO = output_measure.distance_type.clone();
    let d_mids = try_!(dispatch!(
        repack_vec,
        [(QO, [f32, f64, (f32, f32), (f64, f64)])],
        (d_mids)
    ));

    make_sequential_composition(input_domain, input_metric, output_measure, d_in, d_mids).into()
}

impl<
        QI: 'static + ProductOrd + Clone + Send + Sync,
        QO: 'static + ProductOrd + Clone + Send + Sync,
    >
    Measurement<
        AnyDomain,
        Queryable<Measurement<AnyDomain, AnyObject, TypedMetric<QI>, TypedMeasure<QO>>, AnyObject>,
        AnyMetric,
        AnyMeasure,
    >
{
    pub fn into_any_queryable_map(
        self,
    ) -> Fallible<Measurement<AnyDomain, Queryable<AnyMeasurement, AnyObject>, AnyMetric, AnyMeasure>>
    {
        let function = self.function.clone();

        Measurement::new(
            self.input_domain.clone(),
            Function::new_fallible(
                move |arg: &AnyObject| -> Fallible<Queryable<AnyMeasurement, AnyObject>> {
                    let mut inner_qbl = function.eval(arg)?;

                    Queryable::new(move |_self, query: Query<AnyMeasurement>| match query {
                        Query::External(query) => {
                            let privacy_map = query.privacy_map.clone();
                            let meas = Measurement::new(
                                query.input_domain.clone(),
                                query.function.clone(),
                                TypedMetric::<QI>::new(query.input_metric.clone())?,
                                TypedMeasure::<QO>::new(query.output_measure.clone())?,
                                PrivacyMap::new_fallible(move |d_in: &QI| {
                                    privacy_map.eval(&AnyObject::new(d_in.clone()))?.downcast()
                                }),
                            )?;
                            inner_qbl.eval(&meas).map(Answer::External)
                        }
                        Query::Internal(query) => {
                            let Answer::Internal(a) =
                                inner_qbl.eval_query(Query::Internal(query))?
                            else {
                                return fallible!(
                                    FailedFunction,
                                    "internal query returned external answer"
                                );
                            };
                            Ok(Answer::Internal(a))
                        }
                    })
                },
            ),
            self.input_metric.clone(),
            self.output_measure.clone(),
            self.privacy_map.clone(),
        )
    }
}
