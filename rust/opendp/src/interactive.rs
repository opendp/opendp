use std::any::Any;
use std::rc::Rc;

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation};
use crate::dom::AllDomain;
use crate::error::*;
use crate::traits::{FallibleSub, MeasureDistance, MetricDistance};

/// A structure tracking the state of an interactive measurement queryable.
/// It's generic over state (S), query (Q), answer (A), so it can be used for any
/// interactive measurement expressible as a transition function.
pub struct Queryable<S, Q, A> {
    /// The state of the Queryable. It is wrapped in an option so that ownership can be moved out
    /// temporarily, during transitions.
    state: Option<S>,
    /// The transition function of the Queryable. Takes the current state and a query, returns
    /// the new state and the answer.
    transition: Rc<dyn Fn(S, &Q) -> Fallible<(S, A)>>,
}

impl<S, Q, A> Queryable<S, Q, A> {
    /// Constructs a Queryable with initial state and transition function.
    pub fn new(initial: S, transition: impl Fn(S, &Q) -> Fallible<(S, A)> + 'static) -> Self {
        Queryable {
            state: Some(initial),
            transition: Rc::new(transition),
        }
    }

    /// Evaluates the given query.
    pub fn eval(&mut self, query: &Q) -> Fallible<A> {
        // Take temporary ownership of the state from this struct.
        let state = self.state.take().unwrap_assert("Queryable state is only accessed in this method, always replaced.");
        // Obtain then new state and answer.
        let new_state_answer = (self.transition)(state, query)?;
        // Restore ownership of the state into this struct.
        self.state.replace(new_state_answer.0);
        Ok(new_state_answer.1)
    }
}

impl<S, Q> Queryable<S, Q, Box<dyn Any>> {
    pub fn eval_downcast<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(query)?.downcast().map_err(|_| err!(FailedCast)).map(|b| *b)
    }
}

type AcState<DI, MI, MO> = (<DI as Domain>::Carrier, <MI as Metric>::Distance, <MO as Measure>::Distance);
type AcQuery<DI, DO, MI, MO> = (Measurement<DI, DO, MI, MO>, <MO as Measure>::Distance);
type AcQueryable<DI, DO, MI, MO> = Queryable<AcState<DI, MI, MO>, AcQuery<DI, DO, MI, MO>, <DO as Domain>::Carrier>;
type AcMeasurement<DI, DO, MI, MO> = Measurement<DI, AllDomain<AcQueryable<DI, DO, MI, MO>>, MI, MO>;

fn ac_transition<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    (arg, d_in_budget, d_out_budget): AcState<DI, MI, MO>,
    (measurement, d_out_query): &AcQuery<DI, DO, MI, MO>,
) -> Fallible<(AcState<DI, MI, MO>, DO::Carrier)>
    where MO::Distance: MeasureDistance {
    measurement.privacy_relation.eval(&d_in_budget, d_out_query)?;
    if d_out_query > &d_out_budget {
        return fallible!(FailedRelation, "not enough budget")
    }
    let new_d_out_budget = d_out_budget.sub(d_out_query)?;
    // we want the res to be fallible, so that failing eval does not consume the queryable
    let res = measurement.function.eval(&arg)?;
    Ok(((arg, d_in_budget, new_d_out_budget), res))
}

pub fn make_adaptive_composition<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    d_in_budget: MI::Distance,
    d_out_budget: MO::Distance,
) -> AcMeasurement<DI, DO, MI, MO>
    where DI::Carrier: Clone,
          MI::Distance: 'static + MetricDistance + Clone,
          MO::Distance: 'static + MeasureDistance + Clone {
    AcMeasurement::new(
        input_domain,
        AllDomain::new(),
        Function::new(enclose!((d_in_budget, d_out_budget), move |arg: &DI::Carrier| -> AcQueryable<DI, DO, MI, MO> {
            AcQueryable::new(
                // TODO: Remove these clones and have the Queryable use refs. (Also remove Clone trait bounds.)
                (arg.clone(), d_in_budget.clone(), d_out_budget.clone()),
                |s, q| ac_transition(s, q))
        })),
        input_metric,
        output_measure,
        PrivacyRelation::new(move |d_in, d_out| d_in <= &d_in_budget && d_out <= &d_out_budget),
    )
}


#[cfg(test)]
mod tests {
    use crate::dist::{HammingDistance, L1Sensitivity, MaxDivergence};
    use crate::dom::VectorDomain;
    use crate::error::*;
    use crate::meas::*;
    use crate::trans::*;

    use super::*;

    fn make_dummy_meas<TO: From<i32>>() -> Measurement<AllDomain<i32>, AllDomain<TO>, L1Sensitivity<f64>, MaxDivergence<f64>> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|a: &i32| TO::from(a.clone())),
            L1Sensitivity::<f64>::default(),
            MaxDivergence::<f64>::default(),
            PrivacyRelation::new(|d_in, d_out| d_out <= d_in),
        )
    }

    #[test]
    fn test_adaptive_comp_simple() -> Fallible<()> {
        let meas1 = make_dummy_meas::<i32>();
        let meas2 = make_dummy_meas::<i32>();

        let data = 999;
        let d_in_budget = 1.0;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(*meas1.input_domain.clone(), *meas1.input_metric.clone(), *meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.function.eval(&data)?;
        let res1 = queryable.eval(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999);
        let res2 = queryable.eval(&(meas2, d_out_budget / 2.0))?;
        assert_eq!(res2, 999);

        Ok(())
    }

    #[test]
    fn test_adaptive_comp_heterogeneous() -> Fallible<()> {
        let meas1 = make_dummy_meas::<i32>().into_any_out();
        let meas2 = make_dummy_meas::<i64>().into_any_out();

        let data = 999;
        let d_in_budget = 1.0;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(*meas1.input_domain.clone(), *meas1.input_metric.clone(), *meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.function.eval(&data)?;
        let res1: i32 = queryable.eval_downcast(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999_i32);
        let res2: i64 = queryable.eval_downcast(&(meas2, d_out_budget / 2.0))?;
        assert_eq!(res2, 999_i64);

        Ok(())
    }

    #[test]
    fn test_adaptive_comp_budget() -> Fallible<()> {
        let meas1 = make_dummy_meas::<i32>();
        let meas2 = make_dummy_meas::<i32>();

        let data = 999;
        let d_in_budget = 1.0;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(*meas1.input_domain.clone(), *meas1.input_metric.clone(), *meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.function.eval(&data)?;
        let res1 = queryable.eval(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999);
        let res2 = queryable.eval(&(meas2, d_out_budget));
        // TODO: Would be handy to have a way of comparing Errors for this assertion.
        assert!(res2.is_err());

        Ok(())
    }

    #[test]
    fn test_adaptive_comp_chain() -> Fallible<()> {
        // Definitions
        let input_domain = VectorDomain::new(AllDomain::new());
        // Output domain is implied (AnyDomain).
        let input_metric = HammingDistance::default();
        let output_measure = MaxDivergence::default();

        // Build queryable
        let d_in = 1;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(input_domain, input_metric, output_measure, d_in, d_out_budget);
        let data = vec![0.6, 2.8, 6.0, 9.4, 8.9, 7.7, 5.9, 3.4, 8.0, 2.4, 4.4, 7.1, 6.0, 3.2, 7.1];
        let mut queryable = adaptive.function.eval(&data)?;
        // NO FURTHER ACCESS TO DATA AFTER THIS POINT.

        // Set parameters for queries
        let count_bounds = (0, 20);
        let val_bounds = (0.0, 10.0);
        let d_out_query = 0.5 * d_out_budget;

        // Noisy count
        let measurement1 = (
            make_count()? >>
                make_base_geometric(1.0 / d_out_query, count_bounds.0, count_bounds.1)?
        )?.into_any_out();
        let query1 = (measurement1, d_out_query);
        let result1: u32 = queryable.eval_downcast(&query1)?;
        println!("result = {}", result1);

        // Noisy sum
        let measurement2 = (
            make_clamp_vec(val_bounds.0, val_bounds.1)? >>
                make_bounded_sum(val_bounds.0, val_bounds.1)? >>
                make_base_laplace(1.0 / d_out_query)?
        )?.into_any_out();
        let query2 = (measurement2, d_out_query);
        let result2: f64 = queryable.eval_downcast(&query2)?;
        println!("result = {}", result2);

        Ok(())
    }
}
