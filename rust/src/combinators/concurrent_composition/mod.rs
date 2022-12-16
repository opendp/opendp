use std::any::Any;

use num::Zero;

use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::{AllDomain, QueryableDomain},
    error::Fallible,
    interactive::{Queryable, QueryableBase},
    measures::{MaxDivergence, SmoothedMaxDivergence},
    traits::TotalOrd,
};

use super::BasicCompositionMeasure;

pub fn make_concurrent_composition<
    DI: Domain + 'static,
    DQ: Domain + 'static,
    DA: Domain + 'static,
    MI: Metric + Default + 'static,
    MO: ConcurrentCompositionMeasure + BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    query_domain: DQ,
    answer_domain: DA,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<
    Measurement<DI, AllDomain<Measurement<DI, DQ, DA, MI, MO>>, QueryableDomain<DQ, DA>, MI, MO>,
>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + Zero,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_mid");
    }

    // we'll iteratively pop from the end
    d_mids.reverse();

    let d_out = output_measure.compose(d_mids.clone())?;

    Ok(Measurement::new(
        input_domain.clone(),
        AllDomain::new(),
        QueryableDomain::new(query_domain, answer_domain),
        Function::new(enclose!(
            (input_domain, output_measure, d_in),
            move |arg: &DI::Carrier| {
                // a new copy of the state variables is made each time the Function is called:

                // IMMUTABLE STATE VARIABLES
                let input_domain = input_domain.clone();
                let output_measure = output_measure.clone();
                let d_in = d_in.clone();
                let arg = arg.clone();

                // MUTABLE STATE VARIABLES
                let mut d_mids = d_mids.clone();

                // below, the queryable closure's arguments are
                // 1. a reference to itself (which it can use to set context)
                // 2. the query, which is a dynamically typed `&dyn Any`

                // arg, d_mids, d_in and d_out are all moved into (or captured by) the Queryable closure here
                Queryable::new(move |_self: &QueryableBase, query: &dyn Any| {
                    // evaluate the measurement query and return the answer.
                    //     the downcast ref attempts to downcast the &dyn Any to a specific concrete type
                    //     if the query passed in was this type of measurement, the downcast will succeed
                    if let Some(measurement) =
                        query.downcast_ref::<Measurement<DI, DQ, DA, MI, MO>>()
                    {
                        assert_components_match!(
                            DomainMismatch,
                            input_domain,
                            measurement.input_domain
                        );

                        assert_components_match!(
                            MetricMismatch,
                            MI::default(),
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
                        let answer = measurement.invoke(&arg)?;

                        // we've now consumed the trailing d_mid. This is our only state modification
                        d_mids.pop();

                        // The box allows the return value to be dynamically typed, just like query was.
                        // Necessary because different queries have different return types.
                        // All responses are of type `Fallible<Box<dyn Any>>`
                        return Ok(Box::new(answer));
                    }

                    fallible!(FailedFunction, "unrecognized query!")
                })
            }
        )),
        MI::default(),
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

/// A trait that is only implemented for measures that support concurrent composition
#[doc(hidden)]
pub trait ConcurrentCompositionMeasure: Measure {}

// concurrent composition supports at least pure and approximate DP
impl<Q: Clone> ConcurrentCompositionMeasure for MaxDivergence<Q> {}
impl<Q: Clone> ConcurrentCompositionMeasure for SmoothedMaxDivergence<Q> {}

#[cfg(test)]
mod test {

    use crate::{
        domains::{AllDomain, PolyDomain},
        measurements::make_randomized_response_bool,
        metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_concurrent_composition() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_concurrent_composition::<_, _, _, DiscreteDistance, _>(
            AllDomain::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the CC queryable
        let _answer1: bool = queryable.eval(&rr_poly_query)?.get_poly()?;
        let _answer2: bool = queryable.eval(&rr_poly_query)?.get_poly()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let cc_query_3 = make_concurrent_composition(
            AllDomain::<bool>::new(),
            AllDomain::<()>::new(),
            AllDomain::<bool>::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        let mut answer3: Queryable<_, Queryable<(), bool>> = queryable.eval_poly(&cc_query_3)?;
        let _answer3_1: bool = answer3.eval(&rr_query)?.get()?;
        let _answer3_2: bool = answer3.eval(&rr_query)?.get()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in PolyDomain
        let cc_query_4 = make_concurrent_composition(
            AllDomain::<bool>::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<_, Queryable<Box<dyn Any>, Box<dyn Any>>> =
            queryable.eval_poly(&cc_query_4)?;
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly()?;
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly()?;

        Ok(())
    }
}
