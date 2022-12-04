use std::any::Any;

use crate::{
    core::{Domain, Function, InteractiveMeasurement, Measure, Measurement, Metric, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{CheckDescendantChange, Context, EvalIfQueryable, Queryable},
    traits::{InfAdd, TotalOrd},
};

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
) -> Fallible<InteractiveMeasurement<DI, Measurement<DI, DO, MI, MO>, DO::Carrier, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd,
    Measurement<DI, DO, MI, MO>: EvalIfQueryable<OutputCarrier = DO::Carrier>,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_out");
    }

    let d_out = (d_mids.iter().cloned().map(Ok))
        .reduce(|a, b| a?.inf_add(&b?))
        .expect("there is always at least one d_out")?;

    // we'll iteratively pop from the end
    d_mids.reverse();

    Ok(InteractiveMeasurement::new(
        input_domain,
        QueryableDomain::new(),
        Function::new(enclose!((d_in, d_out, d_mids), move |arg: &DI::Carrier| {
            // STATE
            let mut context: Option<Context> = None;
            let mut d_mids = d_mids.clone();

            Queryable::new(enclose!(
                (d_in, d_out, arg),
                move |s: &Queryable<Measurement<DI, DO, MI, MO>, DO::Carrier>, q: &dyn Any| {
                    if let Some(q_meas) = q.downcast_ref::<Measurement<DI, DO, MI, MO>>() {
                        let d_mid =
                            (d_mids.pop()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                        if !q_meas.check(&d_in, &d_mid)? {
                            return fallible!(FailedFunction, "insufficient budget for query");
                        }

                        // pre-check
                        if let Some(context) = &mut context {
                            context.parent.eval::<_, ()>(&CheckDescendantChange {
                                index: context.id,
                                new_privacy_loss: d_out.clone(),
                                commit: false,
                            })?;
                        }

                        let mut answer = q_meas.invoke(&arg)?;

                        // // commit... I don't think this is needed in this case
                        // if let Some(context) = &mut context {
                        //     context.parent.eval::<_, ()>(&CheckDescendantChange {
                        //         index: context.id,
                        //         new_privacy_loss: d_out.clone(),
                        //         commit: true,
                        //     })?;
                        // }

                        // register context with the child if it is a queryable
                        Measurement::<DI, DO, MI, MO>::eval_if_queryable(
                            &mut answer,
                            Context {
                                parent: s.base.clone(),
                                id: d_mids.len(),
                            },
                        )?;
                        return Ok(Box::new(answer) as Box<dyn Any>);
                    }

                    // tell this queryable that it is a child of some other queryable
                    if let Some(q) = q.downcast_ref::<Context>() {
                        if context.is_some() {
                            return fallible!(FailedFunction, "context has already been set");
                        }
                        context.replace(q.clone());
                        return Ok(Box::new(()) as Box<dyn Any>);
                    }

                    // children are always IM's, so new_privacy_loss is bounded by d_mid_i
                    if let Some(query) = q.downcast_ref::<CheckDescendantChange<MO::Distance>>() {
                        return if let Some(context) = &mut context {
                            context.parent.eval_any(&CheckDescendantChange {
                                index: context.id,
                                new_privacy_loss: d_out.clone(),
                                commit: query.commit,
                            })
                        } else {
                            Ok(Box::new(()) as Box<dyn Any>)
                        };
                    }

                    fallible!(FailedFunction, "unrecognized query!")
                }
            ))
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

#[cfg(test)]
mod test {
    use crate::{
        domains::AllDomain, measurements::make_randomized_response_bool, measures::MaxDivergence,
        metrics::DiscreteDistance,
    };

    use super::*;

    #[test]
    fn test_concurrent_composition() -> Fallible<()> {
        // construct concurrent compositor IM
        let comp = make_concurrent_composition(
            AllDomain::new(),
            DiscreteDistance::default(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = comp.invoke(&true)?;

        // pass queries into the CC queryable
        let _answer1 = queryable.eval(&make_randomized_response_bool(0.5, false)?.into_poly())?;
        let _answer2 = queryable.eval(&make_randomized_response_bool(0.5, false)?.into_poly())?;

        // pass a concurrent composition compositor into the original CC compositor
        // TODO: Fails because into_poly doesn't erase the INTERACTIVE const generic
        // let answer3 = queryable.eval(&make_concurrent_composition(
        //     AllDomain::new(),
        //     DiscreteDistance::default(),
        //     MaxDivergence::default(),
        //     1,
        //     vec![0.2, 0.3],
        // )?.into_poly())?;
        // let _answer3_1 = answer3.eval(&make_randomized_response_bool(0.5, false)?)?;
        // let _answer3_2 = answer3.eval(&make_randomized_response_bool(0.5, false)?)?;

        let _answer3 = queryable.eval(&make_randomized_response_bool(0.5, false)?.into_poly())?;
        Ok(())
    }
}
