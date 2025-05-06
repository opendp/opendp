use opendp_derive::bootstrap;
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    combinators::{Adaptivity, Composability, assert_elements_match},
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::{Answer, Query, Queryable, Wrapper},
    traits::ProductOrd,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

use super::CompositionMeasure;

#[bootstrap(
    features("contrib"),
    arguments(
        d_in(rust_type = "$get_distance_type(input_metric)", c_type = "AnyObject *"),
        d_mids(rust_type = "Vec<QO>", c_type = "AnyObject *"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null")
    ),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress)),
    derived_types(QO = "$get_distance_type(output_measure)")
)]
/// Construct a Measurement that when invoked,
/// returns a queryable that interactively composes measurements.
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * interactive: mechanisms can be specified based on answers to previous queries
/// * compositor: all privacy parameters specified up-front
///
/// If the privacy measure supports concurrency,
/// this compositor allows you to spawn multiple interactive mechanisms
/// and interleave your queries amongst them.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
/// * `d_in` - maximum distance between adjacent input datasets
/// * `d_mids` - maximum privacy expenditure of each query
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric
/// * `MO` - Output Metric
/// * `TO` - Output Type.
pub fn make_adaptive_composition<
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: CompositionMeasure + 'static,
    TO: 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, MI, MO, Queryable<Measurement<DI, MI, MO, TO>, TO>>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + ProductOrd + Clone,
    MO::Distance: 'static + ProductOrd + Clone + Debug,
    (DI, MI): MetricSpace,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "d_mids must have at least one element");
    }

    // we'll iteratively pop from the end
    d_mids.reverse();

    let d_out = output_measure.compose(d_mids.clone())?;

    let require_sequentiality = matches!(
        output_measure.composability(Adaptivity::Adaptive)?,
        Composability::Sequential
    );

    // an upper bound on the privacy unit in the privacy map
    let d_in_constructor = d_in.clone();

    Measurement::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(move |data: &DI::Carrier| {
            // a new copy of the state variables is made each time the Function is called:

            // IMMUTABLE STATE VARIABLES
            let input_domain = input_domain.clone();
            let input_metric = input_metric.clone();
            let output_measure = output_measure.clone();
            let d_in = d_in.clone();
            let data = data.clone();

            // MUTABLE STATE VARIABLES
            let mut d_mids = d_mids.clone();

            // below, the queryable closure's arguments are
            // 1. a reference to itself (which it can use to tell child queryables about their parent)
            // 2. the query (a measurement)

            // all state variables are moved into (or captured by) the Queryable closure here
            Queryable::new(move |self_, query: Query<Measurement<DI, MI, MO, TO>>| {
                // this queryable and wrapped children communicate via an AskPermission query
                // defined here, where no-one else can access the type
                struct AskPermission(usize);

                // if the query is external (passed by the user), then it is a measurement
                if let Query::External(meas) = query {
                    assert_elements_match!(DomainMismatch, input_domain, meas.input_domain);
                    assert_elements_match!(MetricMismatch, input_metric, meas.input_metric);
                    assert_elements_match!(MeasureMismatch, output_measure, meas.output_measure);

                    // retrieve the last distance from d_mids, or bubble an error if d_mids is empty
                    let d_mid =
                        (d_mids.last()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    // check that the query doesn't consume too much privacy
                    if !meas.check(&d_in, d_mid)? {
                        return fallible!(
                            FailedFunction,
                            "insufficient budget for query: {:?} > {:?}",
                            meas.map(&d_in)?,
                            d_mid
                        );
                    }

                    let enforce_sequentiality = Rc::new(RefCell::new(false));
                    let seq_wrapper = require_sequentiality.then(|| {
                        // Wrap any spawned queryables with a check that no new queries have been asked.
                        let child_id = d_mids.len() - 1;
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

                    // we've now increased our privacy spend. This is our only state modification
                    d_mids.pop();

                    // done!
                    return Ok(Answer::External(answer));
                }

                // if the query is internal (passed by the framework)
                if let Query::Internal(query) = query {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential compositor has moved on
                        if *id != d_mids.len() {
                            return fallible!(
                                FailedFunction,
                                "Adaptive compositor has received a new query. To satisfy the sequentiality constraint of adaptive composition, only the most recent release from the parent compositor may be interacted with."
                            );
                        }
                        // otherwise, return Ok to approve the change
                        return Ok(Answer::internal(()));
                    }
                }

                fallible!(FailedFunction, "unrecognized query: {:?}", query)
            })
        }),
        PrivacyMap::new_fallible(move |d_in_map: &MI::Distance| {
            if d_in_map.total_gt(&d_in_constructor)? {
                fallible!(
                    RelationDebug,
                    "d_in from the privacy map must be no greater than the d_in passed into the constructor"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        d_in(rust_type = "$get_distance_type(input_metric)", c_type = "AnyObject *"),
        d_mids(rust_type = "Vec<QO>", c_type = "AnyObject *"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null")
    ),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress)),
    derived_types(QO = "$get_distance_type(output_measure)")
)]
/// Construct a Measurement that when invoked,
/// returns a queryable that interactively composes measurements.
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * interactive: mechanisms can be specified based on answers to previous queries
/// * compositor: all privacy parameters specified up-front
///
/// If the privacy measure supports concurrency,
/// this compositor allows you to spawn multiple interactive mechanisms
/// and interleave your queries amongst them.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
/// * `d_in` - maximum distance between adjacent input datasets
/// * `d_mids` - maximum privacy expenditure of each query
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric
/// * `MO` - Output Metric
/// * `TO` - Output Type.
#[deprecated(
    since = "0.14.0",
    note = "This function has been renamed, use `make_adaptive_composition` instead."
)]
pub fn make_sequential_composition<
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: CompositionMeasure + 'static,
    TO: 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, MI, MO, Queryable<Measurement<DI, MI, MO, TO>, TO>>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + ProductOrd + Clone + Send + Sync,
    MO::Distance: 'static + ProductOrd + Clone + Send + Sync + Debug,
    (DI, MI): MetricSpace,
{
    make_adaptive_composition(input_domain, input_metric, output_measure, d_in, d_mids)
}
