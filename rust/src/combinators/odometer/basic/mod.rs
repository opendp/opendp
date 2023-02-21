use crate::{
    combinators::{assert_components_match, BasicCompositionMeasure, OdometerAnswer},
    core::{Domain, Function, Metric, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{Answer, IntoPolyQueryable, Query, Queryable},
};

use super::{Invokable, OdometerQuery, OdometerQueryable, ChildChange};

pub fn make_basic_odometer<
    DI: 'static + Domain,
    Q: 'static + Invokable<DI, MI, MO>,
    MI: 'static + Metric + Default,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    output_measure: MO,
) -> Fallible<Odometer<DI, OdometerQueryable<Q, Q::Output, MI::Distance, MO::Distance>, MI, MO>>
where
    MI::Distance: 'static + Clone,
    DI::Carrier: Clone,
    MO::Distance: Clone,
{
    let input_metric = MI::default();

    Ok(Odometer::new(
        input_domain.clone(),
        Function::new(enclose!(
            (input_domain, input_metric, output_measure),
            move |arg: &DI::Carrier| {
                // IMMUTABLE STATE VARIABLES
                let input_domain = input_domain.clone();
                let input_metric = input_metric.clone();
                let output_measure = output_measure.clone();
                let arg = arg.clone();

                // MUTABLE STATE VARIABLES
                let mut child_maps: Vec<PrivacyMap<MI, MO>> = vec![];

                Queryable::new(
                    move |self_: &Queryable<_, _>, query: Query<OdometerQuery<Q, MI::Distance>>| {
                        Ok(match query {
                            // evaluate external invoke query
                            Query::External(OdometerQuery::Invoke(invokable)) => {
                                assert_components_match!(
                                    DomainMismatch,
                                    input_domain,
                                    invokable.input_domain()
                                );

                                assert_components_match!(
                                    MetricMismatch,
                                    input_metric,
                                    invokable.input_metric()
                                );

                                assert_components_match!(
                                    MeasureMismatch,
                                    output_measure,
                                    invokable.output_measure()
                                );

                                let answer = invokable.invoke(
                                    &arg,
                                    self_.clone().into_poly(),
                                    child_maps.len(),
                                )?;

                                child_maps.push(invokable.privacy_map());

                                Answer::External(OdometerAnswer::Invoke(answer))
                            }
                            // evaluate external map query
                            Query::External(OdometerQuery::Map(d_in)) => {
                                let d_out = output_measure.compose(
                                    child_maps
                                        .iter()
                                        .map(|pmap| pmap.eval(&d_in))
                                        .collect::<Fallible<_>>()?,
                                )?;
                                Answer::External(OdometerAnswer::Map(d_out))
                            }
                            Query::Internal(query) => {
                                
                                if let Some(ChildChange {
                                    id,
                                    new_privacy_map,
                                    commit
                                }) = query.downcast_ref()
                                {
                                    let mut pending_child_maps = child_maps.clone();
                                    if let Some(id) = id {
                                        *pending_child_maps.get_mut(*id).ok_or_else(|| {
                                            err!(FailedFunction, "child not recognized")
                                        })? = new_privacy_map.clone();
                                    } else {
                                        pending_child_maps.push(new_privacy_map.clone());
                                    }
                                    
                                    let pending_map: PrivacyMap<MI, MO> = PrivacyMap::new_fallible(
                                        enclose!((output_measure, pending_child_maps), move |d_in| {
                                            output_measure.compose(
                                                pending_child_maps
                                                    .iter()
                                                    .map(|pmap| pmap.eval(d_in))
                                                    .collect::<Fallible<_>>()?,
                                            )
                                        }),
                                    );

                                    if *commit {
                                        child_maps = pending_child_maps;
                                    }

                                    return Ok(Answer::internal(pending_map));
                                }
                                
                                return fallible!(FailedFunction, "query not recognized");
                            }
                        })
                    },
                )
            }
        )),
        input_metric,
        output_measure,
    ))
}

#[cfg(test)]
mod test {

    use crate::{
        combinators::make_concurrent_composition, core::Measurement, domains::AllDomain,
        interactive::PolyQueryable, measurements::make_randomized_response_bool,
        measures::MaxDivergence, metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_privacy_odometer() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_basic_odometer::<_, Measurement<_, _, _, _>, _, _>(
            AllDomain::new(),
            MaxDivergence::default(),
        )?;

        // pass dataset in and receive a queryable
        let mut odometer = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?
            .interactive()
            .into_poly_queryable();
        let rr_query = make_randomized_response_bool(0.5, false)?.interactive();

        // pass queries into the odometer queryable
        let _answer1: bool = odometer.eval_invoke(rr_poly_query.clone())?.get_poly()?;
        let _answer2: bool = odometer.eval_invoke(rr_poly_query.clone())?.get_poly()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let cc_query_3 = make_concurrent_composition::<_, Queryable<(), bool>, _, _>(
            AllDomain::<bool>::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly_queryable();

        println!("\nsubmitting a CC query. This CC compositor is concretely-typed");
        let mut answer3: Queryable<_, Queryable<(), bool>> =
            odometer.eval_invoke(cc_query_3)?.into_downcast();

        println!("\nsubmitting a RR query to child CC compositor with concrete types");
        let _answer3_1: bool = answer3.eval(&rr_query)?.get()?;

        println!("\nsubmitting a second RR query to child CC compositor with concrete types");
        let _answer3_2: bool = answer3.eval(&rr_query)?.get()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are Boxed and type-erased
        let cc_query_4 = make_concurrent_composition::<_, PolyQueryable, _, _>(
            AllDomain::<bool>::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly_queryable();

        println!("\nsubmitting a second CC query to root CC compositor with type erasure");
        let mut answer4: Queryable<_, PolyQueryable> =
            odometer.eval_invoke(cc_query_4)?.into_downcast();

        println!("\nsubmitting a RR query to child CC compositor");
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly()?;
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly()?;

        let total_usage = odometer.eval_map(1)?;
        println!("total usage: {:?}", total_usage);

        Ok(())
    }
}
