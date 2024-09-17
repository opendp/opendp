use opendp_derive::bootstrap;
use std::fmt::Debug;

use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::{Answer, Query, Queryable, WrapFn},
    traits::ProductOrd,
};

#[cfg(feature = "ffi")]
mod ffi;

use super::BasicCompositionMeasure;

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
/// * `TO` - Output Type.
/// * `MI` - Input Metric
/// * `MO` - Output Metric
pub fn make_sequential_composition<
    DI: Domain + 'static,
    TO: 'static,
    MI: Metric + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, Queryable<Measurement<DI, TO, MI, MO>, TO>, MI, MO>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + ProductOrd + Clone + Send + Sync,
    MO::Distance: 'static + ProductOrd + Clone + Send + Sync + Debug,
    (DI, MI): MetricSpace,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "d_mids must have at least one element");
    }

    // we'll iteratively pop from the end
    d_mids.reverse();

    let d_out = output_measure.compose(d_mids.clone())?;

    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(enclose!(
            (d_in, input_metric, output_measure),
            move |arg: &DI::Carrier| {
                // a new copy of the state variables is made each time the Function is called:

                // IMMUTABLE STATE VARIABLES
                let input_domain = input_domain.clone();
                let input_metric = input_metric.clone();
                let output_measure = output_measure.clone();
                let d_in = d_in.clone();
                let arg = arg.clone();

                // MUTABLE STATE VARIABLES
                let mut d_mids = d_mids.clone();

                // below, the queryable closure's arguments are
                // 1. a reference to itself (which it can use to tell child queryables about their parent)
                // 2. the query (a measurement)

                // all state variables are moved into (or captured by) the Queryable closure here
                Queryable::new(move |sc_qbl, query: Query<Measurement<DI, TO, MI, MO>>| {
                    // this queryable and wrapped children communicate via an AskPermission query
                    // defined here, where no-one else can access the type
                    struct AskPermission(pub usize);

                    // if the query is external (passed by the user), then it is a measurement
                    if let Query::External(measurement) = query {
                        assert_components_match!(
                            DomainMismatch,
                            input_domain,
                            measurement.input_domain
                        );

                        assert_components_match!(
                            MetricMismatch,
                            input_metric,
                            measurement.input_metric
                        );

                        assert_components_match!(
                            MeasureMismatch,
                            output_measure,
                            measurement.output_measure
                        );

                        // retrieve the last distance from d_mids, or bubble an error if d_mids is empty
                        let d_mid = (d_mids.last())
                            .ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                        // check that the query doesn't consume too much privacy
                        if !measurement.check(&d_in, d_mid)? {
                            return fallible!(
                                FailedFunction,
                                "insufficient budget for query: {:?} > {:?}",
                                measurement.map(&d_in)?,
                                d_mid
                            );
                        }

                        let answer = if output_measure.concurrent()? {
                            // evaluate the query directly; no wrapping is necessary
                            measurement.invoke(&arg)
                        } else {
                            // if the answer contains a queryable,
                            // wrap it so that when the child gets a query it sends an AskPermission query to this parent queryable
                            // it gives this sequential composition queryable (or any parent of this queryable)
                            // a chance to deny the child permission to execute
                            let child_id = d_mids.len() - 1;

                            let mut sc_qbl = sc_qbl.clone();
                            let wrap_logic = WrapFn::new_pre_hook(move || {
                                sc_qbl.eval_internal(&AskPermission(child_id))
                            });

                            // evaluate the query and wrap the answer
                            measurement.invoke_wrap(&arg, wrap_logic.as_map())
                        }?;

                        // we've now consumed the last d_mid. This is our only state modification
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
            }
        )),
        input_metric,
        output_measure,
        PrivacyMap::new_fallible(move |actual_d_in: &MI::Distance| {
            if actual_d_in.total_gt(&d_in)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than the d_in passed into the constructor"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    )
}

#[cfg(test)]
mod test;
