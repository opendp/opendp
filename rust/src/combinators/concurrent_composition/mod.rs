use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::{QueryableDomain, Hookable, AllDomain},
    error::Fallible,
    interactive::Queryable,
    traits::{InfAdd, TotalOrd},
};

pub fn make_concurrent_composition_naive<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<
    Measurement<
        DI,
        QueryableDomain<Vec<MO::Distance>, Measurement<DI, DO, MI, MO>, DO::Carrier>,
        MI,
        MO,
    >,
>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd,
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
        Function::new(enclose!((d_in, d_mids), move |arg: &DI::Carrier| {
            let state = d_mids.clone();
            let transition = enclose!(
                (d_in, arg),
                move |mut s: Vec<MO::Distance>, q: &Measurement<DI, DO, MI, MO>| {
                    let d_out = (s.pop()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    if !q.check(&d_in, &d_out)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }

                    Ok((s, q.invoke(&arg)?))
                }
            );
            Queryable::new(state, transition)
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

type CCQuery<DI, S, Q, A, MI, MO> = Measurement<DI, AllDomain<Queryable<Hookable<S, <MO as Measure>::Distance>, Q, A>>, MI, MO>;

pub fn make_concurrent_composition<DI: Domain, S, Q, A, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<
    Measurement<
        DI,
        QueryableDomain<
            Vec<MO::Distance>, 
            CCQuery<DI, S, Q, A, MI, MO>, 
            Queryable<Hookable<S, MO::Distance>, Q, A>
        >,
        MI,
        MO,
    >,
>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd,
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
        Function::new(enclose!((d_in, d_mids), move |arg: &DI::Carrier| {

            let state = d_mids.clone();
            let transition = enclose!(
                (d_in, arg),
                move |mut s: Vec<MO::Distance>, q: &CCQuery<DI, S, Q, A, MI, MO>| {
                    let d_out = (s.pop()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    if !q.check(&d_in, &d_out)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }

                    let a = q.invoke(&arg)?;

                    Ok((s, a))
                }
            );
            Queryable::new(state, transition)
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