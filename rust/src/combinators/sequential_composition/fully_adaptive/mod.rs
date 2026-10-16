use std::{cell::RefCell, rc::Rc};

use opendp_derive::{bootstrap, proven};

use crate::{
    combinators::{Adaptivity, Composability, CompositionMeasure, MapKey, assert_elements_match},
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable,
    },
    error::Fallible,
    interactive::{Answer, Query, Queryable, Wrapper},
    traits::AlertingAdd,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        d_in(rust_type = "$get_distance_type(input_metric)", c_type = "AnyObject *"),
    ),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress))
)]
/// Construct an odometer that can spawn a compositor queryable.
///
/// # Citations
/// * [WRRW23 Fully Adaptive Composition in Differential Privacy](https://arxiv.org/abs/2203.05481)
/// * [VZ23 Concurrent Composition Theorems for Differential Privacy](http://dx.doi.org/10.1145/3564246.3585241)
/// * [HSTVVXZ23 Concurrent Composition for Interactive Differential Privacy with Adaptive Privacy-Loss Parameters](https://arxiv.org/abs/2309.05901)
///
/// # Runtime
/// Constructing the odometer is `O(1)`.
/// Each invocation evaluates the privacy map of the queried measurement once, at `d_in`.
/// A privacy-loss query is `O(1)`, or `O(m)` in the number of distinct privacy maps under Rényi-DP.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
/// * `d_in` - upper bound on the distance between adjacent input datasets, at which privacy loss is reported
pub fn make_fully_adaptive_composition<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
    TO: 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, MI, MO, TO>, TO>>
where
    DI::Carrier: Clone,
    MI::Distance: Clone,
    (DI, MI): MetricSpace,
{
    // check if fully adaptive composition is supported
    output_measure.composability(Adaptivity::FullyAdaptive)?;

    Odometer::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(enclose!(d_in, move |arg: &DI::Carrier| {
            new_fully_adaptive_composition_queryable(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                d_in.clone(),
                arg.clone(),
            )
        })),
        d_in,
    )
}

#[proven(
    proof_path = "combinators/sequential_composition/fully_adaptive/new_fully_adaptive_composition_queryable.tex"
)]
fn new_fully_adaptive_composition_queryable<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    data: DI::Carrier,
) -> Fallible<OdometerQueryable<Measurement<DI, MI, MO, TO>, TO, MO::Distance>>
where
    (DI, MI): MetricSpace,
{
    let require_sequentiality = matches!(
        output_measure.composability(Adaptivity::FullyAdaptive)?,
        Composability::Sequential
    );

    let mut acc = MO::Acc::default();

    let mut num_queries: usize = 0;

    Queryable::new(
        move |self_: &OdometerQueryable<Measurement<DI, MI, MO, TO>, TO, MO::Distance>,
              query: Query<OdometerQuery<Measurement<DI, MI, MO, TO>>>| {
            // this queryable and wrapped children communicate via an AskPermission query
            // defined here, where no-one else can access the type
            struct AskPermission(usize);

            Ok(match query {
                // evaluate external invoke query
                Query::External(OdometerQuery::Invoke(meas)) => {
                    assert_elements_match!(DomainMismatch, &input_domain, &meas.input_domain);
                    assert_elements_match!(MetricMismatch, &input_metric, &meas.input_metric);
                    assert_elements_match!(MeasureMismatch, &output_measure, &meas.output_measure);

                    // check the bookkeeping before spending privacy
                    let next_num_queries = num_queries.alerting_add(&1)?;

                    // charge before the measurement runs, so a failing privacy map releases nothing.
                    // This and num_queries are our only state modifications
                    output_measure.accumulate(&mut acc, MapKey::new(&meas.privacy_map), || {
                        meas.privacy_map.eval(&d_in)
                    })?;

                    let enforce_sequentiality = Rc::new(RefCell::new(false));

                    let seq_wrapper = require_sequentiality.then(|| {
                        // Wrap any spawned queryables with a check that no new queries have been asked.
                        let child_id = num_queries;
                        let mut self_ = self_.clone();

                        Wrapper::new_recursive_pre_hook(enclose!(
                            enforce_sequentiality,
                            move || {
                                if *enforce_sequentiality.borrow() {
                                    self_.eval_internal(&AskPermission(child_id))
                                } else {
                                    Ok(())
                                }
                            }
                        ))
                    });

                    // evaluate the query and wrap the answer
                    let answer = meas.invoke_wrap(&data, seq_wrapper)?;

                    // start enforcing sequentiality
                    *enforce_sequentiality.borrow_mut() = true;

                    num_queries = next_num_queries;

                    // done!
                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external privacy loss query
                Query::External(OdometerQuery::PrivacyLoss) => {
                    let d_out = output_measure.compose_acc(&acc)?;
                    Answer::External(OdometerAnswer::PrivacyLoss(d_out))
                }
                Query::Internal(query) => {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != num_queries {
                            return fallible!(
                                FailedFunction,
                                "sequential odometer has received a new query"
                            );
                        }
                        // otherwise, return Ok to approve the change
                        return Ok(Answer::internal(()));
                    }

                    return fallible!(FailedFunction, "query not recognized");
                }
            })
        },
    )
}
