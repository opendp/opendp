use opendp_derive::bootstrap;

use crate::{
    core::{
        Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, OdometerQueryable,
        PrivacyMap,
    },
    error::Fallible,
    interactive::{Query, Queryable, Wrapper, wrap},
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
/// * `d_out` - Upper bound on the privacy loss of the odometer
pub fn make_privacy_filter<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    Q: 'static,
    A: 'static,
>(
    odometer: Odometer<DI, MI, MO, Q, A>,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, OdometerQueryable<Q, A, MO::Distance>, MI, MO>>
where
    DI::Carrier: Clone,
    MI::Distance: Clone + Debug + ProductOrd + Send + Sync,
    MO::Distance: Clone + Debug + ProductOrd + Send + Sync,
    (DI, MI): MetricSpace,
{
    let odo_function = odometer.function.clone();
    let (d_in_, d_out_) = (odometer.d_in.clone(), d_out.clone());

    Measurement::new(
        odometer.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let continuation_rule = new_continuation_rule::<MO::Distance>(d_out.clone());
            wrap(continuation_rule, || odo_function.eval(arg))
        }),
        odometer.input_metric.clone(),
        odometer.output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in_)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than d_in"
                )
            } else {
                Ok(d_out_.clone())
            }
        }),
    )
}

/// Denotes the prospective privacy loss of after releasing a query.
pub enum PendingLoss<U> {
    /// The pending loss is the same as the current loss
    Same,
    /// What the pending loss would be if the query were released
    New(U),
}

/// # Proof Definition
/// Returns a function that wraps a queryable.
/// The wrapped queryable refuses to release any query
/// that would cause the privacy loss to exceed `d_out`.
fn new_continuation_rule<U: 'static + Clone + Debug + ProductOrd>(d_out: U) -> Wrapper {
    Wrapper::new(move |mut queryable| {
        Ok(Queryable::new_raw(enclose!(d_out, move |_, query| {
            // Retrieve the pending privacy loss of all external queries
            // and check if it would exceed the privacy budget
            if let Query::External(external) = query {
                let pending_loss: PendingLoss<U> = queryable.eval_internal(external)?;

                if let PendingLoss::New(pending_d_out) = pending_loss {
                    if pending_d_out.total_gt(&d_out)? {
                        return fallible!(
                            FailedFunction,
                            "insufficient privacy budget: {:?} > {:?}",
                            pending_d_out,
                            d_out
                        );
                    }
                }
            }

            queryable.eval_query(query)
        })))
    })
}
