use std::any::Any;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{ChildChange, Context, PrivacyUsageAfter, Queryable, QueryableBase},
    traits::{InfAdd, TotalOrd},
};

pub fn make_sequential_composition<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + Default + 'static,
    MO: Measure + Default + 'static,
>(
    input_domain: DI,
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
                    let mut answer = measurement.invoke(&arg)?;

                    // we've now consumed the last d_mid. This is our only state modification
                    d_mids.pop();

                    // if the answer is a queryable, 
                    // wrap it so that when the child gets a query it sends a ChildChange query to this parent queryable
                    // it gives this sequential composition queryable (or any parent of this queryable) 
                    // a chance to deny the child permission to execute
                    DO::inject_context::<MO::Distance>(
                        &mut answer,
                        Context::new(self_.clone(), d_mids.len()),
                    );

                    // The box allows the return value to be dynamically typed, just like query was.
                    // Necessary because different queries have different return types.
                    // All responses are of type `Fallible<Box<dyn Any>>`
                    return Ok(Box::new(answer));
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
                if let Some(change) = query.downcast_ref::<ChildChange<MO::Distance>>() {
                    if change.id != d_mids.len()  {
                        return fallible!(FailedFunction, "sequential compositor has received a new query")
                    }
                    // state won't change in response to child,
                    // but return an Ok to approve the change
                    return Ok(Box::new(()));
                }

                fallible!(FailedFunction, "unrecognized query!")
            })
        })),
        MI::default(),
        MO::default(),
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

    use crate::{
        domains::{AllDomain, PolyDomain},
        measurements::make_randomized_response_bool,
    };

    use super::*;

    #[test]
    fn test_sequential_composition() -> Fallible<()> {
        // construct sequential compositor IM
        let root = make_sequential_composition(
            AllDomain::new(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the SC queryable
        let _answer1: bool = queryable.eval_poly(&rr_poly_query)?;
        let _answer2: bool = queryable.eval_poly(&rr_poly_query)?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let sc_query_3 = make_sequential_composition::<_, AllDomain<bool>, _, _>(
            AllDomain::<bool>::new(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        let mut answer3: Queryable<_, AllDomain<bool>> = queryable.eval_poly(&sc_query_3)?;
        let _answer3_1: bool = answer3.eval(&rr_query)?;
        let _answer3_2: bool = answer3.eval(&rr_query)?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in PolyDomain
        let sc_query_4 = make_sequential_composition::<_, PolyDomain, _, _>(
            AllDomain::<bool>::new(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<Measurement<_, PolyDomain, _, _>, _> =
            queryable.eval_poly(&sc_query_4)?;
        let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;
        let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

        Ok(())
    }
}
