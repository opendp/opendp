use std::{any::Any, rc::Rc};

use crate::{
    core::{Domain, Function, InteractiveMeasurement, Measure, Measurement, Metric, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{Context, Node, Queryable, QueryableNode},
    traits::{InfAdd, TotalOrd},
};

enum InnerQuery<'a, 'b> {
    Context(Context<'a>),
    Query(&'b dyn Any),
}

struct CheckDescendantChange<Q> {
    index: usize,
    new_privacy_loss: Q,
    commit: bool,
}

trait Contextualize {
    // the type to be contextualized
    type Content<'a>;

    // a lives at least as long as b, so it's safe to return something with a lifetime of 'a as a lifetime of 'b
    fn contextualize<'a>(
        content: Self::Content<'static>,
        context: Context<'a>,
    ) -> Self::Content<'a>;
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Contextualize for Measurement<DI, DO, MI, MO> {
    type Content<'a> = DO::Carrier;
    fn contextualize<'a>(content: DO::Carrier, _context: Context<'a>) -> DO::Carrier {
        content
    }
}

impl<DI: Domain, MI: Metric, MO: Measure> Contextualize for InteractiveMeasurement<DI, MI, MO> {
    type Content<'a> = QueryableNode<'a>;
    fn contextualize<'a>(
        mut content: QueryableNode<'static>,
        context: Context<'a>,
    ) -> QueryableNode<'a> {
        content.state.context.replace(context);
        content
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
    Measurement<DI, DO, MI, MO>: Contextualize<Content<'static> = DO::Carrier>,
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
            let state = Node {
                value: Rc::new(d_mids.clone()),
                context: None,
            };

            let transition = enclose!((d_in, arg), move |s: &QueryableNode<'_>, q: &dyn Any| {
                let mut d_mids = s
                    .state
                    .value
                    .downcast_ref::<Vec<MO::Distance>>()
                    .unwrap()
                    .clone();

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
                            parent: s,
                            id: d_mids.len(),
                        },
                    )?;
                    let state = Node {
                        value: Rc::new(d_mids),
                        context: s.state.context.clone(),
                    };
                    return Ok((state, Box::new(answer) as Box<dyn Any>));
                }
                if let Some(q) = q.downcast_ref::<Context<'_>>() {
                    if s.state.context.is_some() {
                        return fallible!(FailedFunction, "already joined a tree!");
                    }
                    let state = Node {
                        value: s.state.value.clone(),
                        context: Some(q.clone()),
                    };
                    return Ok((state, Box::new(()) as Box<dyn Any>));
                }
                fallible!(FailedFunction, "unrecognized query!")
            });
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
