use std::any::Any;

use num::Zero;

use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measure, Measurement, Metric, Odometer},
    error::Fallible,
    interactive::{ChildChange, PrivacyUsage, PrivacyUsageAfter, Queryable, QueryableBase},
    traits::{InfAdd, TotalOrd}, domains::QueryableDomain,
};

use super::BasicCompositionMeasure;

pub fn make_odometer<
    DI: Domain + 'static,
    DQ: Domain + 'static,
    DAQ: Domain + 'static,
    DAA: Domain + 'static,
    MI: Metric + Default + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    query_domain: DQ,
    query2_domain: DAQ,
    answer2_domain: DAA,
    output_measure: MO,
    d_in: MI::Distance,
) -> Fallible<Odometer<DI, DQ, QueryableDomain<DAQ, DAA>, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
    DQ::Carrier: Invokable<DI, DAQ, DAA, MI, MO>,
{
    Ok(Odometer::new(
        input_domain.clone(),
        query_domain,
        QueryableDomain::new(query2_domain, answer2_domain),
        Function::new(enclose!(
            (input_domain, output_measure, d_in),
            move |arg: &DI::Carrier| {
                // IMMUTABLE STATE VARIABLES
                let input_domain = input_domain.clone();
                let output_measure = output_measure.clone();
                let d_in = d_in.clone();
                let arg = arg.clone();

                // MUTABLE STATE VARIABLES
                let mut d_children: Vec<MO::Distance> = vec![];

                Queryable::new(move |self_: &QueryableBase, query: &dyn Any| {
                    // evaluate query if it is a measurement
                    if let Some(invokable) = query.downcast_ref::<DQ::Carrier>() {
                        assert_components_match!(
                            DomainMismatch,
                            input_domain,
                            invokable.input_domain()
                        );

                        assert_components_match!(
                            MetricMismatch,
                            MI::default(),
                            invokable.input_metric()
                        );

                        assert_components_match!(
                            MeasureMismatch,
                            output_measure,
                            invokable.output_measure()
                        );

                        let d_child = invokable.map(&d_in)?;
                        let answer = invokable
                            .invoke(&arg, Context::new(self_.clone(), d_children.len()))?;

                        d_children.push(d_child);

                        return Ok(Box::new(answer));
                    }

                    // returns what the privacy usage would be after evaluating the measurement
                    if let Some(PrivacyUsageAfter(invokable)) =
                        query.downcast_ref::<PrivacyUsageAfter<DQ::Carrier>>()
                    {
                        let mut pending_d_children = d_children.clone();
                        pending_d_children.push(invokable.map(&d_in)?);

                        return Ok(Box::new(output_measure.compose(pending_d_children)?));
                    }

                    // returns what the privacy usage is
                    if query.downcast_ref::<PrivacyUsage>().is_some() {
                        return Ok(Box::new(output_measure.compose(d_children.clone())?));
                    }

                    // update state based on child change
                    if let Some(change) = query.downcast_ref::<ChildChange<MO::Distance>>() {
                        let mut pending_d_children = d_children.clone();
                        *pending_d_children
                            .get_mut(change.id)
                            .ok_or_else(|| err!(FailedFunction, "child not recognized"))? =
                            change.new_privacy_loss.clone();

                        if change.commit {
                            d_children[change.id] = change.new_privacy_loss.clone();
                        }

                        return output_measure
                            .compose(pending_d_children)
                            .map(|v| Box::new(v) as Box<dyn Any>);
                    }

                    fallible!(FailedFunction, "query not recognized")
                })
            }
        )),
        MI::default(),
        MO::default(),
        d_in,
    ))
}

pub trait Invokable<DI: Domain, DQ: Domain, DA: Domain, MI: Metric, MO: Measure> {
    fn invoke(
        &self,
        value: &DI::Carrier,
        context: Context,
    ) -> Fallible<Queryable<DQ::Carrier, DA>>;
    fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance>;

    fn input_domain(&self) -> DI;
    fn input_metric(&self) -> MI;
    fn output_measure(&self) -> MO;
}

impl<DI: Domain, DQ: Domain, DA: Domain, MI: Metric, MO: Measure> Invokable<DI, DQ, DA, MI, MO>
    for Measurement<DI, DQ, DA, MI, MO>
{
    fn invoke(
        &self,
        value: &DI::Carrier,
        _context: Context,
    ) -> Fallible<Queryable<DQ::Carrier, DA>> {
        self.invoke(value)
    }

    fn map(&self, d_in: &MI::Distance) -> Fallible<<MO as Measure>::Distance> {
        self.map(d_in)
    }

    fn input_domain(&self) -> DI {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> MI {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> MO {
        self.output_measure.clone()
    }
}

impl<DI: Domain, DQ: Domain, DA: Domain, MI: Metric, MO: Measure> Invokable<DI, DQ, DA, MI, MO>
    for Odometer<DI, DQ, DA, MI, MO>
where
    MO::Distance: 'static + Clone + Zero,
    DQ::Carrier: 'static + Clone,
    DA::Carrier: 'static,
{
    fn invoke(
        &self,
        value: &DI::Carrier,
        mut context: Context,
    ) -> Fallible<Queryable<DQ::Carrier, DA>> {
        let mut inner = self.invoke(value)?;

        Ok(Queryable::new(
            move |_self: &QueryableBase, query: &dyn Any| {
                if let Some(query_typed) = query.downcast_ref::<DQ::Carrier>() {
                    let d_mid = inner.eval_privacy_after::<MO::Distance>(&query_typed)?;

                    context.parent.eval(&ChildChange {
                        id: context.id,
                        new_privacy_loss: d_mid.clone(),
                        commit: false,
                    })?;

                    let answer = inner.eval(query_typed)?;

                    context.parent.eval(&ChildChange {
                        id: context.id,
                        new_privacy_loss: d_mid.clone(),
                        commit: true,
                    })?;

                    return Ok(Box::new(answer));
                }

                inner.base.eval_any(query)
            },
        ))
    }

    fn map(&self, _d_in: &MI::Distance) -> Fallible<MO::Distance> {
        Ok(MO::Distance::zero())
    }

    fn input_domain(&self) -> DI {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> MI {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> MO {
        self.output_measure.clone()
    }
}

#[derive(Clone)]
pub struct Context {
    parent: QueryableBase,
    id: usize,
}

impl Context {
    pub(crate) fn new(parent: QueryableBase, id: usize) -> Self {
        Context { parent, id }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        combinators::{make_concurrent_composition, make_sequential_composition},
        domains::{AllDomain, PolyDomain},
        measurements::make_randomized_response_bool,
        measures::MaxDivergence,
    };

    use super::*;

    #[test]
    fn test_privacy_odometer() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_odometer(
            AllDomain::new(),
            AllDomain::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the odometer queryable
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

        let mut answer3: Queryable<_, QueryableDomain<AllDomain<()>, AllDomain<bool>>> = queryable.eval_poly(&cc_query_3)?;
        let _answer3_1: bool = answer3.eval(&rr_query)?.get()?;
        let _answer3_2: bool = answer3.eval(&rr_query)?.get()?;

        // pass a concurrent composition compositor into the original CC compositor
        // This compositor expects all outputs are in PolyDomain
        let sc_query_4 = make_sequential_composition(
            AllDomain::<bool>::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<_, QueryableDomain<PolyDomain, PolyDomain>> =
            queryable.eval_poly(&sc_query_4)?;
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly()?;
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly()?;

        Ok(())
    }
}
