use std::{cell::RefCell, rc::Rc};

use opendp_derive::proven;

use crate::{
    combinators::{Adaptivity, Composition, CompositionMeasure, assert_elements_match},
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable,
    },
    error::Fallible,
    interactive::{Answer, Query, Queryable, Wrapper},
};

#[cfg(test)]
mod test;

/// Construct an odometer that can spawn a compositor queryable.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
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
    MI::Distance: Clone + Send + Sync,
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    let require_sequentiality = matches!(
        output_measure.composability(Adaptivity::FullyAdaptive)?,
        Composition::Sequential
    );

    Odometer::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            new_fully_adaptive_composition_queryable(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                d_in.clone(),
                arg.clone(),
                require_sequentiality,
            )
        }),
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
    require_sequentiality: bool,
) -> Fallible<OdometerQueryable<Measurement<DI, MI, MO, TO>, TO, MO::Distance>>
where
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    let mut d_mids: Vec<MO::Distance> = vec![];

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

                    let d_mid = meas.map(&d_in)?;
                    let enforce_sequentiality = Rc::new(RefCell::new(false));

                    let seq_wrapper = require_sequentiality.then(|| {
                        // when the output measure doesn't allow concurrent composition,
                        // wrap any interactive queryables spawned.
                        // This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                        // giving this sequential odometer queryable
                        // a chance to deny the child permission to execute
                        let child_id = d_mids.len();
                        let mut self_ = self_.clone();

                        Wrapper::new_recursive_pre_hook(enclose!(
                            enforce_sequentiality,
                            move || {
                                if *enforce_sequentiality.borrow() {
                                    self_.eval_internal(&AskPermission(child_id))?
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

                    // we've now increased our privacy spend. This is our only state modification
                    d_mids.push(d_mid);

                    // done!
                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external map query
                Query::External(OdometerQuery::PrivacyLoss) => Answer::External(
                    OdometerAnswer::PrivacyLoss(output_measure.compose(d_mids.clone())?),
                ),
                Query::Internal(query) => {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != d_mids.len() {
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
