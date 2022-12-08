use std::any::Any;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{ChildChange, Context, PrivacyUsageAfter, Queryable, QueryableBase},
    measures::{MaxDivergence, SmoothedMaxDivergence},
    traits::{InfAdd, TotalOrd},
};

pub fn make_concurrent_composition<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + 'static,
    MO: ConcurrentCompositionMeasure + 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, QueryableDomain<Measurement<DI, DO, MI, MO>, DO>, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Copy,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Copy + InfAdd,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_out");
    }

    let d_out = (d_mids.iter().cloned().map(Ok))
        .reduce(|a, b| a?.inf_add(&b?))
        .expect("there is always at least one d_out")?;

    // we'll iteratively pop from the end
    d_mids.reverse();

    Ok(Measurement::new(
        input_domain,
        QueryableDomain::new(),
        Function::new(enclose!((d_in, d_out), move |arg: &DI::Carrier| {
            // a new copy of the state variables is made each time the Function is called:

            // IMMUTABLE STATE VARIABLES
            let arg = arg.clone();

            // MUTABLE STATE VARIABLES
            let mut d_mids = d_mids.clone();

            // below, the queryable closure's arguments are
            // 1. a reference to itself (which it can use to set context)
            // 2. the query, which is a dynamically typed `&dyn Any`

            // arg, d_mids, d_in and d_out are all moved into (or captured by) the Queryable closure here
            Queryable::new(move |self_: &QueryableBase, query: &dyn Any| {

                // evaluate the measurement query and return the answer.
                //     the downcast ref attempts to downcast the &dyn Any to a specific concrete type
                //     if the query passed in was this type of measurement, the downcast will succeed
                if let Some(measurement) = query.downcast_ref::<Measurement<DI, DO, MI, MO>>() {

                    // retrieve the last distance from d_mids, or bubble an error if d_mids is empty
                    let d_mid =
                        (d_mids.last()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    // check that the query doesn't consume too much privacy
                    if !measurement.check(&d_in, d_mid)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }

                    // evaluate the query!
                    let answer = measurement.invoke(&arg)?;

                    // we've now consumed the last d_mid. This is our only state modification
                    d_mids.pop();

                    // if the answer is a queryable, 
                    // wrap it so that when the child gets a query it sends a ChildChange query to this parent queryable
                    // it gives this concurrent composition queryable (or any parent of this queryable) 
                    // a chance to deny the child permission to execute
                    let wrapped_answer = DO::wrap_queryable::<MO::Distance>(
                        answer,
                        Context::new(self_.clone(), d_mids.len()),
                    );

                    // The box allows the return value to be dynamically typed, just like query was.
                    // Necessary because different queries have different return types.
                    // All responses are of type `Fallible<Box<dyn Any>>`
                    return Ok(Box::new(wrapped_answer));
                }

                // returns what the privacy usage would be after evaluating the measurement
                if (query.downcast_ref::<PrivacyUsageAfter<Measurement<DI, DO, MI, MO>>>())
                    .is_some()
                {
                    // privacy usage won't change in response to any query
                    // when this queryable is a child, d_out is used to send a ChildChange query to parent
                    return Ok(Box::new(d_out.clone()));
                }

                // update state based on child change
                if query.downcast_ref::<ChildChange<MO::Distance>>().is_some() {
                    // state won't change in response to child,
                    // but return an Ok to approve the change
                    return Ok(Box::new(()));
                }

                fallible!(FailedFunction, "unrecognized query!")
            })
        })),
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
        measures::MaxDivergence,
        metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_concurrent_composition() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_concurrent_composition(
            AllDomain::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the CC queryable
        let _answer1: bool = queryable.eval_poly(&rr_poly_query)?;
        let _answer2: bool = queryable.eval_poly(&rr_poly_query)?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let cc_query_3 = make_concurrent_composition::<_, AllDomain<bool>, _, _>(
            AllDomain::<bool>::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        let mut answer3: Queryable<_, AllDomain<bool>> = queryable.eval_poly(&cc_query_3)?;
        let _answer3_1: bool = answer3.eval(&rr_query)?;
        let _answer3_2: bool = answer3.eval(&rr_query)?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in PolyDomain
        let cc_query_4 = make_concurrent_composition::<_, PolyDomain, _, _>(
            AllDomain::<bool>::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<Measurement<_, PolyDomain, _, _>, _> =
            queryable.eval_poly(&cc_query_4)?;
        let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;
        let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

        Ok(())
    }
}
