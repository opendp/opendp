use std::{cell::RefCell, rc::Rc};

use opendp_derive::{bootstrap, proven};

use crate::{
    combinators::{Adaptivity, Composability, CompositionMeasure, assert_elements_match},
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable, PrivacyMap,
    },
    error::Fallible,
    interactive::{Answer, Query, Queryable, Wrapper},
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(output_measure(c_type = "AnyMeasure *", rust_type = b"null"),),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress))
)]
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
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, MI, MO, TO>, TO>>
where
    DI::Carrier: Clone,
    (DI, MI): MetricSpace,
{
    // check if fully adaptive composition is supported
    output_measure.composability(Adaptivity::FullyAdaptive)?;

    Odometer::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            new_fully_adaptive_composition_queryable(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                arg.clone(),
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
    data: DI::Carrier,
) -> Fallible<OdometerQueryable<Measurement<DI, MI, MO, TO>, TO, MI::Distance, MO::Distance>>
where
    (DI, MI): MetricSpace,
{
    let require_sequentiality = matches!(
        output_measure.composability(Adaptivity::FullyAdaptive)?,
        Composability::Sequential
    );

    let mut privacy_maps: Vec<PrivacyMap<MI, MO>> = vec![];

    Queryable::new(
        move |self_: &OdometerQueryable<
            Measurement<DI, MI, MO, TO>,
            TO,
            MI::Distance,
            MO::Distance,
        >,
              query: Query<OdometerQuery<Measurement<DI, MI, MO, TO>, _>>| {
            // this queryable and wrapped children communicate via an AskPermission query
            // defined here, where no-one else can access the type
            struct AskPermission(usize);

            Ok(match query {
                // evaluate external invoke query
                Query::External(OdometerQuery::Invoke(meas)) => {
                    assert_elements_match!(DomainMismatch, &input_domain, &meas.input_domain);
                    assert_elements_match!(MetricMismatch, &input_metric, &meas.input_metric);
                    assert_elements_match!(MeasureMismatch, &output_measure, &meas.output_measure);

                    let enforce_sequentiality = Rc::new(RefCell::new(false));

                    let seq_wrapper = require_sequentiality.then(|| {
                        // Wrap any spawned queryables with a check that no new queries have been asked.
                        let child_id = privacy_maps.len();
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
                    privacy_maps.push(meas.privacy_map.clone());

                    // done!
                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external map query
                Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                    let d_mids = (privacy_maps.iter())
                        .map(|m| m.eval(d_in))
                        .collect::<Fallible<_>>()?;

                    let d_out = output_measure.compose(d_mids)?;
                    Answer::External(OdometerAnswer::PrivacyLoss(d_out))
                }
                Query::Internal(query) => {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != privacy_maps.len() {
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
