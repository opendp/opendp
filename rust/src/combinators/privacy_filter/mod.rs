use opendp_derive::{bootstrap, proven};

use crate::{
    core::{
        Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable, PrivacyMap,
    },
    error::Fallible,
    interactive::{Queryable, Wrapper, wrap},
    traits::ProductOrd,
};
use std::fmt::Debug;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        odometer(rust_type = b"null"),
        d_in(
            c_type = "AnyObject *",
            rust_type = "$get_distance_type(odometer_input_metric(odometer))"
        ),
        d_out(
            c_type = "AnyObject *",
            rust_type = "$get_distance_type(odometer_output_measure(odometer))"
        )
    ),
    generics(DI(suppress), MI(suppress), MO(suppress), Q(suppress), A(suppress))
)]
/// Combinator that limits the privacy loss of an odometer.
///
/// Adjusts the queryable returned by the odometer
/// to reject any query that would increase the total privacy loss
/// above the privacy guarantee of the mechanism.
///
/// # Arguments
/// * `odometer` - A privacy odometer
/// * `d_in` - Upper bound on the distance between adjacent datasets
/// * `d_out` - Upper bound on the privacy loss
pub fn make_privacy_filter<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    Q: 'static,
    A: 'static,
>(
    odometer: Odometer<DI, MI, MO, Q, A>,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, MI, MO, OdometerQueryable<Q, A, MI::Distance, MO::Distance>>>
where
    MI::Distance: Clone + ProductOrd,
    MO::Distance: Clone + ProductOrd + Debug,
    (DI, MI): MetricSpace,
{
    let continuation_rule = new_continuation_rule::<Q, A, _, _>(d_in.clone(), d_out.clone());
    Measurement::new(
        odometer.input_domain.clone(),
        odometer.input_metric.clone(),
        odometer.output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            wrap(continuation_rule.clone(), || odometer.invoke(arg))
        }),
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than d_in"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    )
}

#[proven]
/// # Proof Definition
/// Returns a function that wraps a queryable.
/// The wrapped queryable refuses to release any query
/// that would cause the privacy loss to exceed `d_out`.
fn new_continuation_rule<Q, A, QB, AB>(d_in: QB, d_out: AB) -> Wrapper
where
    Q: 'static,
    A: 'static,
    QB: 'static + Clone,
    AB: 'static + Clone + Debug + ProductOrd,
{
    Wrapper::new(move |queryable| {
        let d_in = d_in.clone();
        let d_out = d_out.clone();

        let mut state = Some(queryable);

        Ok(Queryable::new_raw(move |_, query| {
            let Some(queryable) = &mut state else {
                return fallible!(
                    FailedFunction,
                    "filter is exhausted: no more queries can be answered"
                );
            };

            let answer = queryable.eval_query(query)?;

            let OdometerAnswer::<A, AB>::PrivacyLoss(pending_d_out) =
                queryable.eval_poly(&OdometerQuery::<Q, QB>::PrivacyLoss(d_in.clone()))?
            else {
                return fallible!(FailedFunction, "expected privacy loss");
            };

            if pending_d_out.total_gt(&d_out)? {
                state.take();
                return fallible!(
                    FailedFunction,
                    "filter is now exhausted: pending privacy loss ({:?}) would exceed privacy budget ({:?})",
                    pending_d_out,
                    d_out
                );
            }

            Ok(answer)
        }))
    })
}
