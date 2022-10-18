use crate::{
    core::{Domain, Measure, Measurement, Metric, PrivacyMap, Function},
    domains::QueryableDomain,
    error::Fallible,
    traits::{TotalOrd, InfSub}, interactive::Queryable
};

pub fn make_privacy_odometer<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    domain_in: DI,
    metric_in: MI,
    measure_out: MO,
    d_in: MI::Distance,
) -> Fallible<Queryable<MO::Distance, Measurement<DI, DO, MI, MO>, DO::Carrier>>
where
    DI::Carrier: 'static + Clone,
    MI::Distance: 'static + TotalOrd + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
{   
    let state = d_out.clone();
    let transition = enclose!((arg, d_in), move |mut s: MO::Distance, q: &Measurement<DI, DO, MI, MO>| {
        let d_mid = q.map(&d_in)?;
        s = s.inf_sub(&d_mid)?;
        let a = q.invoke(&arg)?;
        Ok((s, a))
    });
    Queryable::new(state, transition)
}
