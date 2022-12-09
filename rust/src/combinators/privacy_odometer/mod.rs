use std::any::Any;

use num::Zero;

use crate::{
    combinators::assert_components_match,
    core::{Domain, Function, Measurement, Metric, Odometer},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{
        ChildChange, Context, PrivacyUsage, PrivacyUsageAfter, Queryable, QueryableBase,
    },
    traits::{InfAdd, TotalOrd},
};

use super::BasicCompositionMeasure;

pub fn make_odometer<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + Default + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    output_measure: MO,
    d_in: MI::Distance,
) -> Fallible<Odometer<DI, QueryableDomain<Measurement<DI, DO, MI, MO>, DO>, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
{
    Ok(Odometer::new(
        input_domain.clone(),
        QueryableDomain::new(),
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
                    if let Some(measurement) = query.downcast_ref::<Measurement<DI, DO, MI, MO>>() {
                        assert_components_match!(
                            DomainMismatch,
                            input_domain,
                            measurement.input_domain
                        );

                        assert_components_match!(
                            MetricMismatch,
                            MI::default(),
                            measurement.input_metric
                        );

                        assert_components_match!(
                            MeasureMismatch,
                            output_measure,
                            measurement.output_measure
                        );

                        let d_child = measurement.map(&d_in)?;

                        let mut answer = measurement.invoke(&arg)?;

                        DO::inject_context(
                            &mut answer,
                            Context::new(self_.clone(), d_children.len()),
                            Some(d_child.clone()),
                        );

                        d_children.push(d_child);

                        return Ok(Box::new(answer));
                    }

                    if let Some(odometer) = query.downcast_ref::<Odometer<DI, DO, MI, MO>>() {
                        assert_components_match!(
                            DomainMismatch,
                            input_domain,
                            odometer.input_domain
                        );

                        assert_components_match!(
                            MetricMismatch,
                            MI::default(),
                            odometer.input_metric
                        );

                        assert_components_match!(
                            MeasureMismatch,
                            output_measure,
                            odometer.output_measure
                        );

                        let mut answer = odometer.invoke(&arg)?;

                        DO::inject_context(
                            &mut answer,
                            Context::new(self_.clone(), d_children.len()),
                            None::<MO::Distance>,
                        );

                        d_children.push(MO::Distance::zero());

                        return Ok(Box::new(answer));
                    }

                    // returns what the privacy usage would be after evaluating the measurement
                    if let Some(PrivacyUsageAfter(measurement)) =
                        query.downcast_ref::<PrivacyUsageAfter<Measurement<DI, DO, MI, MO>>>()
                    {
                        let mut pending_d_children = d_children.clone();
                        pending_d_children.push(measurement.map(&d_in)?);

                        return Ok(Box::new(output_measure.compose(pending_d_children)?));
                    }

                    // returns what the privacy usage is, or would be after adding an odometer
                    if query.downcast_ref::<PrivacyUsage>().is_some()
                        || (query.downcast_ref::<PrivacyUsageAfter<Odometer<DI, DO, MI, MO>>>())
                            .is_some()
                    {
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
    fn test_concurrent_composition() -> Fallible<()> {
        // construct concurrent compositor IM
        let root = make_odometer(AllDomain::<bool>::new(), MaxDivergence::default(), 1)?;

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
        let cc_query_4 = make_sequential_composition::<_, PolyDomain, _, _>(
            AllDomain::<bool>::new(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<Measurement<_, PolyDomain, _, _>, PolyDomain> =
            queryable.eval_poly(&cc_query_4)?;
        let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;
        let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

        Ok(())
    }
}
