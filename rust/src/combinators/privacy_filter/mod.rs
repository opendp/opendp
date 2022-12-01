use crate::{
    core::{Domain, Measure, Measurement, Metric, PrivacyMap, Function},
    domains::QueryableDomain,
    error::Fallible,
    traits::{TotalOrd, InfSub}, interactive::Queryable
};

pub fn make_privacy_filter<'a, DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    domain_in: DI,
    metric_in: MI,
    measure_out: MO,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, QueryableDomain<'a>, MI, MO>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfSub,
{   
    Ok(Measurement::new(
        domain_in,
        QueryableDomain::new(),
        Function::new(enclose!((d_in, d_out), move |arg: &DI::Carrier| {
            let state = d_out.clone();
            let transition = enclose!((arg, d_in), move |mut s: MO::Distance, q: &Measurement<DI, DO, MI, MO>| {
                let d_mid = q.map(&d_in)?;
                s = s.inf_sub(&d_mid)?;
                let a = q.invoke(&arg)?;
                Ok((s, a))
            });
            Queryable::new(state, transition)
        })),
        metric_in,
        measure_out,
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
