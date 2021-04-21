use std::ops::Shr;
use std::rc::Rc;

use crate::core::{Domain, Function, HintMt, HintTt, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use crate::dom::{BoxDomain, PairDomain};
use crate::error::Fallible;

// GLUE FOR FFI USE OF COMBINATORS
fn new_clone<T: Clone>() -> Rc<dyn Fn(&T) -> T> {
    let clone = |t: &T| t.clone();
    Rc::new(clone)
}

fn new_domain_glue<D: Domain>() -> (Rc<dyn Fn(&D, &D) -> bool>, Rc<dyn Fn(&D) -> D>) {
    let eq = |d0: &D, d1: &D| d0 == d1;
    let eq = Rc::new(eq);
    let clone = new_clone();
    (eq, clone)
}

/// Public only for access from FFI.
#[derive(Clone)]
pub struct MeasureGlue<D: Domain, M: Measure> {
    pub domain_eq: Rc<dyn Fn(&D, &D) -> bool>,
    pub domain_clone: Rc<dyn Fn(&D) -> D>,
    pub measure_clone: Rc<dyn Fn(&M) -> M>,
}

impl<D: 'static + Domain, M: 'static + Measure> Default for MeasureGlue<D, M> {
    fn default() -> Self {
        let (domain_eq, domain_clone) = new_domain_glue();
        let measure_clone = new_clone();
        MeasureGlue { domain_eq, domain_clone, measure_clone }
    }
}

/// Public only for access from FFI.
#[derive(Clone)]
pub struct MetricGlue<D: Domain, M: Metric> {
    pub domain_eq: Rc<dyn Fn(&D, &D) -> bool>,
    pub domain_clone: Rc<dyn Fn(&D) -> D>,
    pub metric_clone: Rc<dyn Fn(&M) -> M>,
}

impl<D: 'static + Domain, M: 'static + Metric> Default for MetricGlue<D, M> {
    fn default() -> Self {
        let (domain_eq, domain_clone) = new_domain_glue();
        let metric_clone = new_clone();
        MetricGlue { domain_eq, domain_clone, metric_clone }
    }
}


// CHAINING & COMPOSITION
pub fn make_chain_mt_glue<DI, DX, DO, MI, MX, MO>(
    measurement1: &Measurement<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
    hint: Option<&HintMt<MI, MO, MX>>,
    input_glue: &MetricGlue<DI, MI>,
    x_glue: &MetricGlue<DX, MX>,
    output_glue: &MeasureGlue<DO, MO>,
) -> Fallible<Measurement<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    if !(x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain) {
        return fallible!(DomainMismatch);
    }
    Ok(Measurement {
        input_domain: Box::new((input_glue.domain_clone)(&transformation0.input_domain)),
        output_domain: Box::new((output_glue.domain_clone)(&measurement1.output_domain)),
        function: Function::make_chain(&measurement1.function, &transformation0.function),
        input_metric: Box::new((input_glue.metric_clone)(&transformation0.input_metric)),
        output_measure: Box::new((output_glue.measure_clone)(&measurement1.output_measure)),
        privacy_relation: PrivacyRelation::make_chain(
            &measurement1.privacy_relation,
            &transformation0.stability_relation, hint),
    })
}

pub fn make_chain_mt<DI, DX, DO, MI, MX, MO>(
    measurement1: &Measurement<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Measurement<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    make_chain_mt_glue(
        measurement1, transformation0, None,
        &MetricGlue::<DI, MI>::default(),
        &MetricGlue::<DX, MX>::default(),
        &MeasureGlue::<DO, MO>::default())
}

pub fn make_chain_tt_glue<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
    hint: Option<&HintTt<MI, MO, MX>>,
    input_glue: &MetricGlue<DI, MI>,
    x_glue: &MetricGlue<DX, MX>,
    output_glue: &MetricGlue<DO, MO>,
) -> Fallible<Transformation<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    if !(x_glue.domain_eq)(&transformation0.output_domain, &transformation1.input_domain) {
        return fallible!(DomainMismatch)
    }
    Ok(Transformation {
        input_domain: Box::new((input_glue.domain_clone)(&transformation0.input_domain)),
        output_domain: Box::new((output_glue.domain_clone)(&transformation1.output_domain)),
        function: Function::make_chain(&transformation1.function, &transformation0.function),
        input_metric: Box::new((input_glue.metric_clone)(&transformation0.input_metric)),
        output_metric: Box::new((output_glue.metric_clone)(&transformation1.output_metric)),
        stability_relation: StabilityRelation::make_chain(
            &transformation1.stability_relation,
            &transformation0.stability_relation, hint),
    })
}

pub fn make_chain_tt<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Transformation<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    make_chain_tt_glue(
        transformation1, transformation0, None,
        &MetricGlue::<DI, MI>::default(),
        &MetricGlue::<DX, MX>::default(),
        &MetricGlue::<DO, MO>::default())
}

pub fn make_composition<DI, DO0, DO1, MI, MO>(measurement0: &Measurement<DI, DO0, MI, MO>, measurement1: &Measurement<DI, DO1, MI, MO>) -> Fallible<Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    make_composition_glue(
        measurement0, measurement1,
        &MetricGlue::<DI, MI>::default(),
        &MeasureGlue::<DO0, MO>::default(),
        &MeasureGlue::<DO1, MO>::default())
}

pub fn make_composition_glue<DI, DO0, DO1, MI, MO>(
    measurement0: &Measurement<DI, DO0, MI, MO>,
    measurement1: &Measurement<DI, DO1, MI, MO>,
    input_glue: &MetricGlue<DI, MI>,
    output_glue0: &MeasureGlue<DO0, MO>,
    output_glue1: &MeasureGlue<DO1, MO>,
) -> Fallible<Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    if !(input_glue.domain_eq)(&measurement0.input_domain, &measurement1.input_domain) {
        return fallible!(DomainMismatch);
    }

    Ok(Measurement {
        input_domain: Box::new((input_glue.domain_clone)(&measurement0.input_domain)),
        output_domain: Box::new(PairDomain::new(
            BoxDomain::new(Box::new((output_glue0.domain_clone)(&measurement0.output_domain))),
            BoxDomain::new(Box::new((output_glue1.domain_clone)(&measurement1.output_domain))))),
        function: Function::make_composition(&measurement0.function, &measurement1.function),
        // TODO: Figure out input_metric for composition.
        input_metric: Box::new((input_glue.metric_clone)(&measurement0.input_metric)),
        // TODO: Figure out output_measure for composition.
        output_measure: Box::new((output_glue0.measure_clone)(&measurement0.output_measure)),
        // TODO: PrivacyRelation for make_composition
        privacy_relation: PrivacyRelation::new(|_i, _o| false),
    })
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::dist::{L1Sensitivity, MaxDivergence};
    use crate::dom::AllDomain;
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let function = Function::new(|arg: &i32| arg.clone());
        let input_metric = L1Sensitivity::<i32>::default();
        let output_metric = L1Sensitivity::<i32>::default();
        let stability_relation = StabilityRelation::new_from_constant(1);
        let identity = Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation);
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_chain_mt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Sensitivity::<i32>::default();
        let output_metric0 = L1Sensitivity::<i32>::default();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = make_chain_mt(&measurement1, &transformation0).unwrap_test();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Sensitivity::<i32>::default();
        let output_metric0 = L1Sensitivity::<i32>::default();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::default();
        let output_metric1 = L1Sensitivity::<i32>::default();
        let stability_relation1 = StabilityRelation::new_from_constant(1);
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = make_chain_tt(&transformation1, &transformation0).unwrap_test();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f32);
        let input_metric0 = L1Sensitivity::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_relation0 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = make_composition(&measurement0, &measurement1).unwrap_test();
        let arg = 99;
        let ret = composition.function.eval(&arg).unwrap_test();
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }
}


impl<DI, DX, DO, MI, MX, MO> Shr<Measurement<DX, DO, MX, MO>> for Transformation<DI, DX, MI, MX>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    type Output = Fallible<Measurement<DI, DO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, DO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Measurement<DX, DO, MX, MO>> for Fallible<Transformation<DI, DX, MI, MX>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    type Output = Fallible<Measurement<DI, DO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, DO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self?)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>> for Transformation<DI, DX, MI, MX>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>> for Fallible<Transformation<DI, DX, MI, MX>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self?)
    }
}


#[cfg(test)]
mod test_shift_op {
    use crate::dist::HammingDistance;
    use crate::meas::geometric::make_base_geometric;
    use crate::trans::{make_bounded_sum, make_parse_series, make_split_lines, make_clamp_vec};

    use super::*;

    #[test]
    fn test_shr() -> Fallible<()> {
        let meas = make_split_lines::<HammingDistance>()?
            >> make_parse_series(true)?
            >> make_clamp_vec(0, 1)?
            >> make_bounded_sum(0, 1)?
            >> make_base_geometric(1., 0, 10)?;
        meas?;
        Ok(())
    }
}
