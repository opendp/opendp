use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::{Answer, Query, Queryable, WrapFn},
    traits::TotalOrd,
};

#[cfg(feature = "ffi")]
mod ffi;

use super::BasicCompositionMeasure;

pub fn make_sequential_composition<
    DI: 'static + Domain + Send + Sync,
    TO: 'static,
    MI: 'static + Metric + Send + Sync,
    MO: 'static + BasicCompositionMeasure + Send + Sync,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, Queryable<Measurement<DI, TO, MI, MO>, TO>, MI, MO>>
where
    DI::Carrier: 'static + Clone + Send + Sync,
    MI::Distance: 'static + TotalOrd + Clone + Send + Sync,
    MO::Distance: 'static + TotalOrd + Clone + Send + Sync,
    (DI, MI): MetricSpace,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_mid");
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
                            return fallible!(FailedFunction, "insufficient budget for query");
                        }

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
                        let answer = measurement.invoke_wrap(&arg, wrap_logic.as_map());

                        // we've now consumed the last d_mid. This is our only state modification
                        d_mids.pop();

                        // done!
                        return answer.map(Answer::External);
                    }

                    // if the query is internal (passed by the framework)
                    if let Query::Internal(query) = query {
                        // check if the query is from a child queryable who is asking for permission to execute
                        if let Some(AskPermission(id)) = query.downcast_ref() {
                            // deny permission if the sequential compositor has moved on
                            if *id != d_mids.len() {
                                return fallible!(
                                    FailedFunction,
                                    "sequential compositor has received a new query"
                                );
                            }
                            // otherwise, return Ok to approve the change
                            return Ok(Answer::internal(()));
                        }
                    }

                    fallible!(FailedFunction, "unrecognized query!")
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
mod test {

    use std::any::Any;

    use super::*;
    use crate::{
        domains::AtomDomain, measurements::make_randomized_response_bool, measures::MaxDivergence,
        metrics::DiscreteDistance,
    };

    #[test]
    fn test_sequential_composition() -> Fallible<()> {
        // construct sequential compositor IM
        let root = make_sequential_composition::<_, Box<dyn Any>, _, _>(
            AtomDomain::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        // construct the leaf-node queries:
        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the SC queryable
        println!("the sequential compositor can be evaluated with rr poly queries, and the answer is downcast to bool");
        let _answer1: bool = queryable.eval_poly(&rr_poly_query)?;
        let _answer2: bool = queryable.eval_poly(&rr_poly_query)?;

        println!("\nbuild a sequential composition IM and then convert to poly, so that it can be passed to the root queryable");
        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are concretely-typed (bool)
        let sc_query_3 = make_sequential_composition::<_, bool, _, _>(
            AtomDomain::<bool>::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        // both approaches are valid
        println!("\ncreate the sequential composition queryable as a child of the root queryable");
        let mut answer3a = queryable.eval_poly::<Queryable<_, bool>>(&sc_query_3)?;

        println!("\npass an RR query to the child sequential compositor queryable");
        let _answer3a_1: bool = answer3a.eval(&rr_query)?;

        println!("\npass a second RR query to the child sequential compositor queryable");
        let _answer3a_2: bool = answer3a.eval(&rr_query)?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in PolyDomain, but operates over dyn domains
        println!("\nbuild a dyn sequential composition IM and then convert to poly");
        let sc_query_4 = make_sequential_composition::<_, Box<dyn Any>, _, _>(
            AtomDomain::<bool>::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        println!("\ncreate the poly sequential composition queryable as a child of the root queryable, and downcast the queryable itself");
        let mut answer4 = queryable.eval_poly::<Queryable<_, Box<dyn Any>>>(&sc_query_4)?;

        println!("\nsend a dyn query");
        let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;

        println!("\nsend another dyn query");
        let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

        println!("\ncan no longer send queries to the first compositor child, because a second query has been passed to its parent");
        assert!(answer3a.eval(&rr_query).is_err());

        Ok(())
    }
}
