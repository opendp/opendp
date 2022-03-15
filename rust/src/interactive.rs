use std::any::Any;
use std::rc::Rc;

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation};
use crate::dom::AllDomain;
use crate::error::*;
use crate::traits::{CheckNull, InfSub};

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

    /// Evaluates a query.
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
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(query)?.downcast().map_err(|_| err!(FailedCast)).map(|b| *b)
    }
}

impl<S, Q, A> CheckNull for Queryable<S, Q, A> { fn is_null(&self) -> bool { false } }

pub type InteractiveMeasurement<DI, DO, MI, MO, S, Q> = Measurement<DI, AllDomain<Queryable<S, Q, <DO as Domain>::Carrier>>, MI, MO>;

/// The state of an adaptive composition Queryable.
pub struct AcState<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    input_domain: DI,
    output_domain: DO,
    input_metric: MI,
    output_measure: MO,
    d_in_budget: MI::Distance,
    d_out_budget: MO::Distance,
    data: DI::Carrier,
}
impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> AcState<DI, DO, MI, MO> where MO::Distance: PartialOrd + InfSub {
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        input_metric: MI,
        output_measure: MO,
        data: DI::Carrier,
        d_in_budget: MI::Distance,
        d_out_budget: MO::Distance,
    ) -> Self {
        Self {
            input_domain,
            output_domain,
            input_metric,
            output_measure,
            data,
            d_in_budget,
            d_out_budget,
        }
    }

    /// Checks that a measurement (of a query) is compatible with this Queryable state.
    fn check_types(&self, measurement: &Measurement<DI, DO, MI, MO>) -> Fallible<()> {
        if measurement.input_domain != self.input_domain {
            return fallible!(DomainMismatch, "wrong query input domain")
        } else if measurement.output_domain != self.output_domain {
            return fallible!(DomainMismatch, "wrong query output domain")
        } else if measurement.input_metric != self.input_metric {
            return fallible!(MetricMismatch, "wrong query input metric")
        } else if measurement.output_measure != self.output_measure {
            return fallible!(MeasureMismatch, "wrong query output measure")
        }
        Ok(())
    }

    /// Checks that there is adequate budget in this Queryable state.
    fn check_budget(&self, privacy_relation: &PrivacyRelation<MI, MO>, d_out_query: &MO::Distance) -> Fallible<()> {
        privacy_relation.eval(&self.d_in_budget, d_out_query)?;
        if d_out_query > &self.d_out_budget {
            return fallible!(FailedRelation, "not enough budget")
        }
        Ok(())
    }

    /// Updates this Queryable state by consuming the given amount of budget.
    fn update(self, d_out_query: &MO::Distance) -> Fallible<Self> {
        Ok(Self { d_out_budget: self.d_out_budget.neg_inf_sub(d_out_query)?, ..self })
    }

    /// Processes a query, generating a new Queryable state.
    fn transition(self, (measurement, d_out_query): &AcQuery<DI, DO, MI, MO>) -> Fallible<(Self, DO::Carrier)>
        where MO::Distance: Clone + PartialOrd + InfSub {
        self.check_types(measurement)?;
        self.check_budget(&measurement.privacy_relation, d_out_query)?;
        let res = measurement.invoke(&self.data)?;
        let new = self.update(d_out_query)?;
        Ok((new, res))
    }
}
type AcQuery<DI, DO, MI, MO> = (Measurement<DI, DO, MI, MO>, <MO as Measure>::Distance);
type AcQueryable<DI, DO, MI, MO> = Queryable<AcState<DI, DO, MI, MO>, AcQuery<DI, DO, MI, MO>, <DO as Domain>::Carrier>;
type AcMeasurement<DI, DO, MI, MO> = InteractiveMeasurement<DI, DO, MI, MO, AcState<DI, DO, MI, MO>, AcQuery<DI, DO, MI, MO>>;

pub fn make_adaptive_composition<DI, DO, MI, MO>(
    input_domain: DI,
    output_domain: DO,
    input_metric: MI,
    output_measure: MO,
    d_in_budget: MI::Distance,
    d_out_budget: MO::Distance,
) -> AcMeasurement<DI, DO, MI, MO>
    where DI: 'static + Domain,
          DI::Carrier: Clone,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MI::Distance: 'static + Clone + PartialOrd,
          MO: 'static + Measure,
          MO::Distance: 'static + Clone + PartialOrd + InfSub {
    AcMeasurement::new(
        input_domain.clone(),
        AllDomain::new(),
        Function::new(enclose!((input_domain, input_metric, output_measure, d_in_budget, d_out_budget), move |arg: &DI::Carrier| -> AcQueryable<DI, DO, MI, MO> {
            AcQueryable::new(
                // TODO: Remove these clones and have the Queryable use refs? (Also remove Clone trait bounds.)
                AcState::new(input_domain.clone(), output_domain.clone(), input_metric.clone(), output_measure.clone(), arg.clone(), d_in_budget.clone(), d_out_budget.clone()),
                |s, q| s.transition(q))
        })),
        input_metric,
        output_measure,
        PrivacyRelation::new(move |d_in, d_out| d_in <= &d_in_budget && d_out <= &d_out_budget),
    )
}


#[cfg(test)]
mod tests {
    use crate::dist::{MaxDivergence, AbsoluteDistance, SymmetricDistance};
    use crate::dom::VectorDomain;
    use crate::error::*;
    use crate::meas::*;
    use crate::poly::PolyDomain;
    use crate::trans::*;

    use super::*;

    fn make_dummy_meas<TO: From<i32> + CheckNull>() -> Measurement<AllDomain<i32>, AllDomain<TO>, AbsoluteDistance<f64>, MaxDivergence<f64>> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|a: &i32| TO::from(a.clone())),
            AbsoluteDistance::<f64>::default(),
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
        let adaptive = make_adaptive_composition(meas1.input_domain.clone(), meas1.output_domain.clone(), meas1.input_metric.clone(), meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.invoke(&data)?;
        let res1 = queryable.eval(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999);
        let res2 = queryable.eval(&(meas2, d_out_budget / 2.0))?;
        assert_eq!(res2, 999);

        Ok(())
    }

    #[test]
    fn test_adaptive_composition_poly() -> Fallible<()> {
        let meas1 = make_dummy_meas::<i32>().into_poly();
        let meas2 = make_dummy_meas::<i64>().into_poly();

        let data = 999;
        let d_in_budget = 1.0;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(meas1.input_domain.clone(), PolyDomain::new(), meas1.input_metric.clone(), meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.invoke(&data)?;
        let res1: i32 = queryable.eval_poly(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999_i32);
        let res2: i64 = queryable.eval_poly(&(meas2, d_out_budget / 2.0))?;
        assert_eq!(res2, 999_i64);

        Ok(())
    }

    #[test]
    fn test_adaptive_composition_no_budget() -> Fallible<()> {
        let meas1 = make_dummy_meas::<i32>();
        let meas2 = make_dummy_meas::<i32>();

        let data = 999;
        let d_in_budget = 1.0;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(meas1.input_domain.clone(), meas1.output_domain.clone(), meas1.input_metric.clone(), meas1.output_measure.clone(), d_in_budget, d_out_budget);
        let mut queryable = adaptive.invoke(&data)?;
        let res1 = queryable.eval(&(meas1, d_out_budget / 2.0))?;
        assert_eq!(res1, 999);
        let res2 = queryable.eval(&(meas2, d_out_budget));
        // TODO: Would be handy to have a way of comparing Errors for this assertion.
        assert!(res2.is_err());

        Ok(())
    }

    #[test]
    fn test_adaptive_composition_chain() -> Fallible<()> {
        // Definitions
        let input_domain = VectorDomain::new(AllDomain::new());
        let output_domain = PolyDomain::new();
        let input_metric = SymmetricDistance::default();
        let output_measure = MaxDivergence::default();

        // Build queryable
        let d_in = 1;
        let d_out_budget = 1.0;
        let adaptive = make_adaptive_composition(input_domain, output_domain, input_metric, output_measure, d_in, d_out_budget);
        let data = vec![0.6, 2.8, 6.0, 9.4, 8.9, 7.7, 5.9, 3.4, 8.0, 2.4, 4.4, 7.1, 6.0, 3.2, 7.1];
        let mut queryable = adaptive.invoke(&data)?;
        // NO FURTHER ACCESS TO DATA AFTER THIS POINT.

        // Set parameters for queries
        let count_bounds = (0, 20);
        let val_bounds = (0.0, 10.0);
        let d_out_query = 0.5 * d_out_budget;

        // Noisy count
        let measurement1 = (
            make_count()? >>
            make_base_geometric(1.0 / d_out_query, Some(count_bounds))?
        )?.into_poly();
        let query1 = (measurement1, d_out_query);
        let _result1: i32 = queryable.eval_poly(&query1)?;
        // println!("_result = {}", result1);

        // Noisy sum
        let measurement2 = (
            make_clamp(val_bounds)? >>
            make_bounded_sum(val_bounds)? >>
            make_base_laplace(1.0 / d_out_query)?
        )?.into_poly();
        let query2 = (measurement2, d_out_query);
        let _result2: f64 = queryable.eval_poly(&query2)?;
        // println!("_result = {}", result2);

        Ok(())
    }
}
