use std::rc::Rc;

use crate::{
    core::{Domain, Function, Measure, Measurement, MeasurementBase, Metric, PrivacyMap},
    domains::{AllDomain, Hook, QueryableDomain},
    error::Fallible,
    interactive::{Queryable, Query},
    traits::{InfAdd, TotalOrd},
};

struct CheckDescendantChange<Q> {
    index: usize,
    new_privacy_loss: Q,
    commit: bool,
}

trait Hookable<'a, L> {
    type Carrier;
    type Hooked;
    fn hook(v: Self::Carrier) -> Self::Hooked;
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Hookable<'static, ()>
    for Measurement<DI, DO, MI, MO>
{
    type Carrier = DO::Carrier;
    type Hooked = DO::Carrier;
    fn hook(v: Self::Carrier) -> Self::Hooked {
        v
    }
}

impl<'a, DI: Domain, S, Q, A, MI: Metric, MO: Measure, L> Hookable<'a, L>
    for MeasurementBase<DI, QueryableDomain<S, Q, A>, MI, MO, true>
{
    type Carrier = Queryable<S, Q, A>;
    type Hooked = Queryable<Hook<'a, S, L>, Q, A>;
    fn hook(v: Self::Carrier) -> Self::Hooked {
        let transition = v.transition;
        Queryable {
            state: Some(Hook {
                inner: v.state.unwrap(),
                listener: Box::new(|v, b| Ok(true)),
            }),
            transition: Rc::new(move |s: Hook<'a, S, L>, q: &dyn Query<Q>| {
                let (s_prime, a) = transition(s.inner, q)?;
                Ok((
                    Hook {
                        inner: s_prime,
                        listener: s.listener,
                    },
                    a,
                ))
            }),
        }
    }
}

pub fn make_concurrent_composition<
    'a,
    DI: Domain,
    DO: Domain,
    MI: Metric,
    MO: Measure,
    const I: bool,
>(
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
            MeasurementBase<DI, DO, MI, MO, I>,
            <MeasurementBase<DI, DO, MI, MO, I> as Hookable<'a, MO::Distance>>::Hooked,
        >,
        MI,
        MO,
    >,
>
where
    MeasurementBase<DI, DO, MI, MO, I>: Hookable<'a, MO::Distance>,
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
                move |mut s: Vec<MO::Distance>, q: &dyn Query<Measurement<DI, DO, MI, MO>>| {
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
