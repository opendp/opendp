use std::any::Any;

use crate::{
    core::{Domain, Function, InteractiveMeasurement, Measure, Measurement, Metric, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{Context, Queryable},
    traits::{InfAdd, TotalOrd},
};

// struct CheckDescendantChange<Q> {
//     index: usize,
//     new_privacy_loss: Q,
//     commit: bool,
// }

pub trait Contextualize {
    // the type to be contextualized
    type Content;

    // a lives at least as long as b, so it's safe to return something with a lifetime of 'a as a lifetime of 'b
    fn contextualize(content: Self::Content, context: Context) -> Fallible<()>;
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Contextualize
    for Measurement<DI, DO, MI, MO>
{
    type Content = DO::Carrier;
    fn contextualize(_content: DO::Carrier, _context: Context) -> Fallible<()> {
        Ok(())
    }
}

impl<DI: Domain, MI: Metric, MO: Measure> Contextualize for InteractiveMeasurement<DI, MI, MO> {
    type Content = Queryable;
    fn contextualize(mut content: Queryable, context: Context) -> Fallible<()> {
        content.eval::<()>(&context as &dyn Any)
    }
}

pub fn make_concurrent_composition<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + 'static,
    MO: Measure + 'static,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<Measurement<DI, QueryableDomain, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd,
    Measurement<DI, DO, MI, MO>: Contextualize<Content = DO::Carrier>,
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
            // STATE
            let mut context = None;
            let mut d_mids = d_mids.clone();

            Queryable::new(enclose!((d_in, arg), move |s: &Queryable, q: &dyn Any| {
                if let Some(q_meas) = q.downcast_ref::<Measurement<DI, DO, MI, MO>>() {
                    let d_mid =
                        (d_mids.pop()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    if !q_meas.check(&d_in, &d_mid)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }
                    let answer = q_meas.invoke(&arg)?;

                    let answer = Measurement::<DI, DO, MI, MO>::contextualize(
                        answer,
                        Context {
                            parent: s.clone(),
                            id: d_mids.len(),
                        },
                    );
                    return Ok(Box::new(answer) as Box<dyn Any>);
                }
                if let Some(q) = q.downcast_ref::<Context>() {
                    if context.is_some() {
                        return fallible!(FailedFunction, "context has already been set");
                    }
                    context.replace(q.clone());
                    return Ok(Box::new(()) as Box<dyn Any>);
                }
                fallible!(FailedFunction, "unrecognized query!")
            }))
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
