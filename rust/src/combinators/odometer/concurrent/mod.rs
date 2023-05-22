use crate::{
    combinators::{assert_components_match, BasicCompositionMeasure, OdometerAnswer},
    core::{Domain, Function, Measurement, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{Answer, Query, Queryable},
};

use super::{OdometerQuery, OdometerQueryable};

#[cfg(feature = "ffi")]
mod ffi;

/// Construct a concurrent odometer that spawns a queryable that interactively composes interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
pub fn make_concurrent_odometer<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<
    Odometer<
        DI,
        OdometerQueryable<Measurement<DI, TO, MI, MO>, TO, MI::Distance, MO::Distance>,
        MI,
        MO,
    >,
>
where
    MI::Distance: 'static + Clone,
    DI::Carrier: Clone,
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    Odometer::new(
        input_domain.clone(),
        Function::new_fallible(enclose!(
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
                    move |_co_qbl: &Queryable<_, _>,
                          query: Query<
                        OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>,
                    >| {
                        Ok(match query {
                            // evaluate external invoke query
                            Query::External(OdometerQuery::Invoke(meas)) => {
                                assert_components_match!(
                                    DomainMismatch,
                                    input_domain,
                                    meas.input_domain
                                );

                                assert_components_match!(
                                    MetricMismatch,
                                    input_metric,
                                    meas.input_metric
                                );

                                assert_components_match!(
                                    MeasureMismatch,
                                    output_measure,
                                    meas.output_measure
                                );
                                let answer = meas.invoke(&arg)?;
                                child_maps.push(meas.privacy_map.clone());

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
                                // handler to see privacy usage after running a query.
                                // Someone is passing in an OdometerQuery internally,
                                // so return the potential privacy map of this odometer after running this query
                                if let Some(OdometerQuery::Invoke(meas)) =
                                    query.downcast_ref::<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>()
                                {
                                    let mut pending_child_maps = child_maps.clone();
                                    pending_child_maps.push(meas.privacy_map.clone());
                                    let output_measure = output_measure.clone();

                                    let pending_map = PrivacyMap::<MI, MO>::new_fallible(move |d_in|
                                        output_measure.compose(
                                            (pending_child_maps.iter())
                                                .map(|pmap| pmap.eval(d_in))
                                                .collect::<Fallible<_>>()?));

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
    )
}

#[cfg(test)]
mod test {

    use std::any::Any;

    use crate::{
        combinators::make_sequential_composition, domains::AtomDomain,
        measurements::make_randomized_response_bool, measures::MaxDivergence,
        metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_privacy_odometer() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_concurrent_odometer::<_, Box<dyn Any>, _, _>(
            AtomDomain::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
        )?;

        // pass dataset in and receive a queryable
        let mut co_qbl = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the odometer queryable
        let _answer1: bool = co_qbl.eval_invoke_poly(rr_poly_query.clone())?;
        let _answer2: bool = co_qbl.eval_invoke_poly(rr_poly_query.clone())?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in AtomDomain<bool>
        let cc_query_3 = make_sequential_composition::<_, bool, _, _>(
            AtomDomain::<bool>::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        println!("\nsubmitting a CC query. This CC compositor is concretely-typed");
        let mut answer3 = co_qbl.eval_invoke_poly::<Queryable<_, bool>>(cc_query_3)?;

        println!("\nsubmitting a RR query to child CC compositor with concrete types");
        let _answer3_1: bool = answer3.eval(&rr_query)?;

        println!("\nsubmitting a second RR query to child CC compositor with concrete types");
        let _answer3_2: bool = answer3.eval(&rr_query)?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are Boxed and type-erased
        let cc_query_4 = make_sequential_composition::<_, Box<dyn Any>, _, _>(
            AtomDomain::<bool>::default(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        println!("\nsubmitting a second CC query to root CC compositor with type erasure");
        let mut answer4 = co_qbl.eval_invoke_poly::<Queryable<_, Box<dyn Any>>>(cc_query_4)?;

        println!("\nsubmitting a RR query to child CC compositor");
        let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;
        let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

        let total_usage = co_qbl.eval_map(1)?;
        println!("total usage: {:?}", total_usage);

        Ok(())
    }
}
