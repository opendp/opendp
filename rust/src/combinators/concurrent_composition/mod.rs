use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    error::Fallible,
    interactive::{Queryable, QueryableMap},
    measures::{MaxDivergence, SmoothedMaxDivergence},
    traits::TotalOrd,
};

use super::BasicCompositionMeasure;

#[cfg(feature = "ffi")]
mod ffi;


/// Construct a queryable that interactively composes interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
/// * `d_in` - maximum distance between adjacent input datasets
/// * `d_mids` - maximum privacy expenditure of each query
pub fn make_concurrent_composition<
    DI: Domain + 'static,
    TO: QueryableMap,
    MI: Metric + 'static,
    MO: ConcurrentCompositionMeasure + BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, Queryable<Measurement<DI, TO, MI, MO>, TO>, MI, MO>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_mid");
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

                // below, the queryable closure accepts the query, which is a measurement

                // all state variables are moved into (or captured by) the Queryable closure here
                Queryable::new_external(move |measurement: &Measurement<DI, TO, MI, MO>| {
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
                    let d_mid =
                        (d_mids.last()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    // check that the query doesn't consume too much privacy
                    if !measurement.check(&d_in, d_mid)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }

                    // evaluate the query!
                    let answer = measurement.invoke_mappable(&arg)?;

                    // we've now consumed the last d_mid. This is our only state modification
                    d_mids.pop();

                    Ok(answer)
                })
            }
        )),
        input_metric,
        output_measure,
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than the d_in passed into constructor"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    ))
}

/// A trait that is only implemented for measures that support concurrent composition
#[doc(hidden)]
pub trait ConcurrentCompositionMeasure: Measure {}

// concurrent composition supports at least pure and approximate DP
impl<Q: Clone> ConcurrentCompositionMeasure for MaxDivergence<Q> {}
impl<Q: Clone> ConcurrentCompositionMeasure for SmoothedMaxDivergence<Q> {}


#[cfg(test)]
mod test {

    use crate::{
        domains::AllDomain, interactive::{PolyQueryable, Static}, measurements::make_randomized_response_bool, metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_concurrent_composition() -> Fallible<()> {
        // construct concurrent compositor IM
        //                                      DI, TO,           MI, MO
        let root = make_concurrent_composition::<_, PolyQueryable, _, _>(
            AllDomain::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        println!("preparing query");
        let rr_poly_query = make_randomized_response_bool(0.5, false)?
            .interactive()
            .into_poly_queryable();
        let rr_query = make_randomized_response_bool(0.5, false)?.interactive();

        // pass queries into the CC queryable
        println!("\nsubmitting rr query to CC queryable");
        let mut answer1a = queryable.eval(&rr_poly_query)?;

        println!("\nretrieving value from RR query");
        let _answer1b: bool = answer1a.get_poly::<Static<_>>()?;

        println!("\nsubmitting and retrieving a second RR query");
        let _answer2: bool = queryable.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let cc_query_3 = make_concurrent_composition::<_, Queryable<(), Static<bool>>, _, _>(
            AllDomain::<bool>::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly_queryable();

        println!("\nsubmitting a CC query. This CC compositor is concretely-typed");
        let mut answer3: Queryable<_, Queryable<(), Static<bool>>> = queryable.eval_poly(&cc_query_3)?;

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
        let mut answer4 = queryable.eval_poly::<_, PolyQueryable>(&cc_query_4)?;

        println!("\nsubmitting a RR query to child CC compositor");
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly::<Static<_>>()?;

        Ok(())
    }
}
