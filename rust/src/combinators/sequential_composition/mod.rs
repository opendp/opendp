use std::{any::Any, rc::Rc};

use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measurement, Metric, PrivacyMap},
    error::Fallible,
    interactive::{Answer, PolyQueryable, Query, Queryable, QueryableMap},
    traits::TotalOrd,
};

use super::BasicCompositionMeasure;

pub fn make_sequential_composition<
    DI: Domain + 'static,
    TO: QueryableMap,
    MI: Metric + Default + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, Queryable<Measurement<DI, TO, MI, MO>, TO>, MI, MO>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone,
{
    let input_metric = MI::default();

    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_out");
    }

    // we'll iteratively pop from the end
    d_mids.reverse();

    let d_out = output_measure.compose(d_mids.clone())?;

    Ok(Measurement::new(
        input_domain.clone(),
        Function::new(enclose!(
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

                        // evaluate the query!
                        let answer = measurement.invoke_mappable(&arg)?;

                        // we've now consumed the last d_mid. This is our only state modification
                        d_mids.pop();

                        // if the answer contains a queryable,
                        // wrap it so that when the child gets a query it sends an AskPermission query to this parent queryable
                        // it gives this sequential composition queryable (or any parent of this queryable)
                        // a chance to deny the child permission to execute
                        let child_id = d_mids.len();

                        #[derive(Clone)]
                        struct WrapFn(Rc<dyn Fn(WrapFn, PolyQueryable) -> PolyQueryable>);
                        impl WrapFn {
                            // constructs a closure that wraps a PolyQueryable
                            fn as_map(&self) -> impl Fn(PolyQueryable) -> PolyQueryable {
                                let wrap_logic = self.clone();
                                move |qbl| (wrap_logic.0)(wrap_logic.clone(), qbl)
                            }
                        }

                        let sc_qbl = sc_qbl.clone();
                        let wrap_logic = WrapFn(Rc::new(move |wrap_logic, mut inner_qbl| {
                            
                            let mut sc_qbl = sc_qbl.clone();
                            Queryable::new(move |_wrapper_qbl, query: Query<dyn Any>| {
                                // ask the sequential compositor for permission to execute
                                sc_qbl.eval_internal(&AskPermission(child_id))?;

                                // evaluate the query
                                Ok(match inner_qbl.eval_query(query)? {
                                    // if the answer is external, then wrap it:
                                    Answer::External(answer) => Answer::External(
                                        answer.queryable_map(&wrap_logic.as_map()),
                                    ),
                                    // otherwise, just return the answer:
                                    answer => answer,
                                })
                            })
                        }));

                        // wrap the answer
                        return Ok(Answer::External(
                            answer.queryable_map(&wrap_logic.as_map()),
                        ));
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
    ))
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        domains::AllDomain, measurements::make_randomized_response_bool, measures::MaxDivergence, interactive::Static,
    };

    #[test]
    fn test_sequential_composition() -> Fallible<()> {
        // construct sequential compositor IM
        let root = make_sequential_composition::<_, PolyQueryable, _, _>(
            AllDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        // construct the leaf-node queries:
        let rr_poly_query = make_randomized_response_bool(0.5, false)?
            .interactive()
            .into_poly_queryable();
        let rr_query = make_randomized_response_bool(0.5, false)?.interactive();

        // pass queries into the SC queryable
        println!("the sequential compositor emits an un-typed null queryable");
        let mut answer1: PolyQueryable = queryable.eval(&rr_poly_query)?;

        println!("\nsubmit a second query, which should freeze the first. Retrieve the value");
        let _answer2: bool = queryable.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;

        println!("\ncan no longer execute the first queryable, because a second query has been passed to its parent");
        assert!(answer1.get_poly::<Static<bool>>().is_err());

        println!("\nbuild a sequential composition IM and then convert to poly, so that it can be passed to the root queryable");
        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are concretely-typed (bool)
        let sc_query_3 = make_sequential_composition::<_, Queryable<(), Static<bool>>, _, _>(
            AllDomain::<bool>::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly_queryable();

        // both approaches are valid
        println!("\nAPPROACH A: submit poly queries to a poly queryable");
        println!("\ncreate the sequential composition queryable as a child of the root queryable");
        let mut answer3a = queryable.eval(&sc_query_3)?;

        println!("\npass an RR query to the child sequential compositor queryable and get a null queryable");
        let mut rr_null_qbl = answer3a.eval_poly::<Queryable<(), Static<bool>>>(&rr_query)?;

        println!("\nget the value from the null queryable");
        let _answer3a_1: bool = rr_null_qbl.get()?;

        println!("\npass a second RR query to the child sequential compositor queryable");
        let _answer3a_2: bool = answer3a
            .eval_poly::<Queryable<(), Static<bool>>>(&rr_query)?
            .get()?;

        println!("\nAPPROACH B: downcast the poly queryable and then send normal queries");
        println!("create the sequential composition queryable as a child of the root queryable, but downcast the queryable itself");
        let mut answer3b: Queryable<_, Queryable<(), Static<bool>>> = queryable.eval_poly(&sc_query_3)?;

        println!("\nsend a normal query without any dyn or poly type-erasure");
        let _answer3b_1: bool = answer3b.eval(&rr_query)?.get()?;
        println!("\nsend a second normal query without any dyn or poly type-erasure");
        let _answer3b_2: bool = answer3b.eval(&rr_query)?.get()?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in PolyDomain, but operates over dyn domains
        println!("\nbuild a dyn sequential composition IM and then convert to poly");
        let sc_query_4 = make_sequential_composition::<_, PolyQueryable, _, _>(
            AllDomain::<bool>::new(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly_queryable();

        println!("\ncreate the poly sequential composition queryable as a child of the root queryable, and downcast the queryable itself");
        let mut answer4: Queryable<_, PolyQueryable> = queryable.eval(&sc_query_4)?.into_downcast();

        println!("\nsend a dyn query");
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;

        println!("\nsend another dyn query");
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;

        Ok(())
    }
}
