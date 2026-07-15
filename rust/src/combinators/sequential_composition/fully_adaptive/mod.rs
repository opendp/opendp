use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use opendp_derive::{bootstrap, proven};

use crate::{
    combinators::{Adaptivity, Composability, ComposeK, CompositionMeasure, assert_elements_match},
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
/// # Citations
/// * [WRRW23 Fully Adaptive Composition in Differential Privacy](https://arxiv.org/abs/2203.05481)
/// * [VZ23 Concurrent Composition Theorems for Differential Privacy](http://dx.doi.org/10.1145/3564246.3585241)
/// * [HSTVVXZ23 Concurrent Composition for Interactive Differential Privacy with Adaptive Privacy-Loss Parameters](https://arxiv.org/abs/2309.05901)
///
/// # Runtime
/// Constructing the odometer is `O(1)`.
/// Each invocation adds `O(1)` bookkeeping beyond the cost of the queried measurement,
/// while a privacy-loss query over `m` prior invocations costs `O(m)`.
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
                                    // no ? operator: it would make inference expect a Result answer payload,
                                    // implementation now matches adaptive/mod.rs
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

#[bootstrap(
    features("contrib"),
    arguments(output_measure(c_type = "AnyMeasure *", rust_type = b"null"),),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress))
)]
/// Construct an odometer that can spawn a compositor queryable,
/// aggregating the privacy loss of repeated identical queries.
///
/// Semantically identical to `make_fully_adaptive_composition`,
/// but queries sharing a privacy map are grouped as `(map, k)` in first-submission order,
/// each group is then charged an inf rounded `k`-fold multiple of its privacy loss
///
/// # Citations
/// * [WRRW23 Fully Adaptive Composition in Differential Privacy](https://arxiv.org/abs/2203.05481)
/// * [VZ23 Concurrent Composition Theorems for Differential Privacy](http://dx.doi.org/10.1145/3564246.3585241)
/// * [HSTVVXZ23 Concurrent Composition for Interactive Differential Privacy with Adaptive Privacy-Loss Parameters](https://arxiv.org/abs/2309.05901)
///
/// # Runtime
/// A privacy-loss query evaluates each distinct privacy map once, not once per invocation —
/// this matters when maps are expensive to evaluate, such as Rényi curves.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
pub fn make_fully_adaptive_composition_k<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + ComposeK,
    TO: 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, MI, MO, TO>, TO>>
where
    DI::Carrier: Clone,
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    output_measure.composability(Adaptivity::FullyAdaptive)?;

    Odometer::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            new_fully_adaptive_composition_k_queryable(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                arg.clone(),
            )
        }),
    )
}

/// Delegates to the proven `new_fully_adaptive_composition_queryable` for invocation,
/// mismatch checks, and sequentiality; answers privacy-loss queries from `(map, k)` groups.
fn new_fully_adaptive_composition_k_queryable<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + ComposeK,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI::Carrier,
) -> Fallible<OdometerQueryable<Measurement<DI, MI, MO, TO>, TO, MI::Distance, MO::Distance>>
where
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    let mut inner = new_fully_adaptive_composition_queryable(
        input_domain,
        input_metric,
        output_measure.clone(),
        data,
    )?;

    // queries sharing a privacy map are stored as one (map, k) group, in first-submission order
    let mut privacy_maps: Vec<(PrivacyMap<MI, MO>, u32)> = vec![];

    Queryable::new(
        move |_self: &OdometerQueryable<
            Measurement<DI, MI, MO, TO>,
            TO,
            MI::Distance,
            MO::Distance,
        >,
              query: Query<OdometerQuery<Measurement<DI, MI, MO, TO>, _>>| {
            Ok(match query {
                Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                    let d_mids = (privacy_maps.iter())
                        .map(|(map, k)| output_measure.compose_k(map.eval(d_in)?, *k))
                        .collect::<Fallible<_>>()?;

                    let d_out = output_measure.compose(d_mids)?;
                    Answer::External(OdometerAnswer::PrivacyLoss(d_out))
                }
                Query::External(query) => {
                    let answer = inner.eval_query(Query::External(query))?;
                    if let OdometerQuery::Invoke(meas) = query {
                        // groups hold a clone of the map's Arc, so ptr_eq cannot collide with a freed map
                        let group = (privacy_maps.iter_mut())
                            .find(|(map, _)| Arc::ptr_eq(&map.0, &meas.privacy_map.0));
                        match group {
                            Some((_, k)) => *k += 1,
                            None => privacy_maps.push((meas.privacy_map.clone(), 1)),
                        }
                    }
                    answer
                }
                Query::Internal(query) => inner.eval_query(Query::Internal(query))?,
            })
        },
    )
}
