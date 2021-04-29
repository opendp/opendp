use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use crate::error::*;
use crate::traits::{FallibleSub, MeasureDistance, MetricDistance};

pub trait Downcast {
    fn downcast<T: 'static>(self) -> Fallible<T>;
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T>;
}

pub struct AnyBox {
    value: Box<dyn Any>,
    eq_glue: Rc<dyn Fn(&Self, &Self) -> bool>,
}

impl AnyBox {
    pub fn new<T: 'static + PartialEq>(value: T) -> Self {
        let eq_ = |self_: &Self, other: &Self| {
            // This downcast of self will always succeed, so equality check is all that's necessary.
            self_.value.downcast_ref::<T>() == other.value.downcast_ref::<T>()
        };
        Self {
            value: Box::new(value),
            eq_glue: Rc::new(eq_),
        }
    }
}

impl PartialEq for AnyBox {
    fn eq(&self, other: &Self) -> bool {
        (self.eq_glue)(self, other)
    }
}

impl Downcast for AnyBox {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value.downcast().map_or_else(|_| fallible!(FailedCast), |v| *v)
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref().ok_or_else(|| err!(FailedCast))
    }
}

pub struct CloneableAnyBox {
    value: Box<dyn Any>,
    eq_glue: Rc<dyn Fn(&Self, &Self) -> bool>,
    clone_glue: Rc<dyn Fn(&Self) -> Self>,
}

impl CloneableAnyBox {
    pub fn new<T: 'static + PartialEq + Clone>(value: T) -> Self {
        let eq_ = |self_: &Self, other: &Self| {
            // This downcast of self will always succeed, so equality check is all that's necessary.
            self_.value.downcast_ref::<T>() == other.value.downcast_ref::<T>()
        };
        let clone_ = |self_: &Self| {
            Self {
                value: Box::new(self_.value.downcast_ref::<T>().unwrap().clone()),
                eq_glue: self_.eq_glue.clone(),
                clone_glue: self_.clone_glue.clone(),
            }
        };
        Self {
            value: Box::new(value),
            eq_glue: Rc::new(eq_),
            clone_glue: Rc::new(clone_),
        }
    }
}

impl PartialEq for CloneableAnyBox {
    fn eq(&self, other: &Self) -> bool {
        (self.eq_glue)(self, other)
    }
}

impl Clone for CloneableAnyBox {
    fn clone(&self) -> Self {
        (self.clone_glue)(self)
    }
}

impl Downcast for CloneableAnyBox {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value.downcast().map_or_else(|_| fallible!(FailedCast), |v| *v)
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref().ok_or_else(|| err!(FailedCast))
    }
}


#[derive(Clone)]
pub struct AnyDomain {
    pub domain: CloneableAnyBox,
    pub member_glue: Rc<dyn Fn(&Self, &dyn Any) -> bool>,
}

impl AnyDomain {
    pub fn new<D: 'static + Domain>(domain: D) -> Self {
        Self {
            domain: CloneableAnyBox::new(domain),
            member_glue: Rc::new(|self_: &Self, val: &dyn Any| {
                let self_ = self_.downcast_ref::<D>().unwrap_assert("downcast of AnyDomain to constructed type will always work");
                let val = val.downcast_ref::<D::Carrier>();
                // FIXME: Return a Fallible here for bad downcast (https://github.com/opendp/opendp/issues/87)
                val.map_or(false, |v| self_.member(v))
            }),
        }
    }
}

impl PartialEq for AnyDomain {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
    }
}

impl Domain for AnyDomain {
    type Carrier = Box<dyn Any>;
    fn member(&self, val: &Self::Carrier) -> bool {
        (self.member_glue)(self, val)
    }
}

impl Downcast for AnyDomain {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.domain.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.domain.downcast_ref()
    }
}

// TODO: If/when we remove the clone of the budget from make_adaptive_composition(), then remove Clone from AnyXXXDistance.
#[derive(Clone)]
pub struct AnyMeasureDistance {
    pub distance: CloneableAnyBox,
    pub partial_cmp_glue: Rc<dyn Fn(&Self, &Self) -> Option<Ordering>>,
    pub sub_glue: Rc<dyn Fn(Self, &Self) -> Fallible<Self>>,
}

impl AnyMeasureDistance {
    pub fn new<Q: 'static + Clone + MeasureDistance>(distance: Q) -> Self {
        Self {
            distance: CloneableAnyBox::new(distance),
            partial_cmp_glue: Rc::new(|self_: &Self, other: &Self| -> Option<Ordering> {
                let self_ = self_.downcast_ref::<Q>().unwrap_assert("downcast of AnyMeasureDistance to constructed type will always work");
                let other = other.downcast_ref::<Q>();
                // FIXME: Do we want to have a FalliblePartialCmp for this?
                other.map_or(None, |o| self_.partial_cmp(o))
            }),
            sub_glue: Rc::new(|self_: Self, rhs: &Self| -> Fallible<Self> {
                let distance = self_.downcast::<Q>()?;
                let rhs = rhs.downcast_ref::<Q>()?;
                let res = distance.sub(rhs);
                res.map(Self::new)
            }),
        }
    }
}

impl PartialEq for AnyMeasureDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl PartialOrd for AnyMeasureDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.partial_cmp_glue)(self, other)
    }
}

impl FallibleSub<&Self> for AnyMeasureDistance {
    type Output = Self;
    fn sub(self, rhs: &Self) -> Fallible<Self::Output> {
        // We have to clone sub_glue, because self is moved into the call.
        self.sub_glue.clone()(self, rhs)
    }
}

impl Downcast for AnyMeasureDistance {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.distance.downcast()
    }

    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.distance.downcast_ref()
    }
}

// TODO: If/when we remove the clone of the budget from make_adaptive_composition(), then remove Clone from AnyXXXDistance.
#[derive(Clone)]
pub struct AnyMetricDistance {
    pub distance: CloneableAnyBox,
    pub partial_cmp_glue: Rc<dyn Fn(&Self, &Self) -> Option<Ordering>>,
}

impl AnyMetricDistance {
    pub fn new<Q: 'static + Clone + MetricDistance>(distance: Q) -> Self {
        Self {
            distance: CloneableAnyBox::new(distance),
            partial_cmp_glue: Rc::new(|self_: &Self, other: &Self| -> Option<Ordering> {
                let self_ = self_.downcast_ref::<Q>().unwrap_assert("downcast of AnyMeasureDistance to constructed type will always work");
                let other = other.downcast_ref::<Q>();
                // FIXME: Do we want to have a FalliblePartialCmp for this?
                other.map_or(None, |o| self_.partial_cmp(o))
            }),
        }
    }
}

impl PartialEq for AnyMetricDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl PartialOrd for AnyMetricDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.partial_cmp_glue)(self, other)
    }
}

impl Downcast for AnyMetricDistance {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.distance.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.distance.downcast_ref()
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyMeasure {
    pub measure: CloneableAnyBox,
}

impl AnyMeasure {
    pub fn new<M: 'static + Measure>(measure: M) -> Self {
        Self { measure: CloneableAnyBox::new(measure) }
    }
}

impl Default for AnyMeasure {
    fn default() -> Self { unimplemented!("called AnyMeasure::default()") }
}

impl Measure for AnyMeasure {
    type Distance = AnyMeasureDistance;
}

#[derive(Clone, PartialEq)]
pub struct AnyMetric {
    pub metric: CloneableAnyBox,
}

impl AnyMetric {
    pub fn new<M: 'static + Metric>(metric: M) -> Self {
        Self { metric: CloneableAnyBox::new(metric) }
    }
}

impl Default for AnyMetric {
    fn default() -> Self { unimplemented!("called AnyMetric::default()") }
}

impl Metric for AnyMetric {
    type Distance = AnyMetricDistance;
}

impl<DI: Domain, DO: Domain> Function<DI, DO> where DI::Carrier: 'static, DO::Carrier: 'static {
    pub fn as_any_out(&self) -> Function<DI, AnyDomain> {
        let function = self.function.clone();
        let function = move |arg: &DI::Carrier| -> Fallible<Box<dyn Any>> {
            let res = function(arg);
            res.map(|o| Box::new(*o) as Box<dyn Any>)
        };
        Function::new_fallible(function)
    }
    pub fn as_any(&self) -> Function<AnyDomain, AnyDomain> {
        let function = self.function.clone();
        let function = move |arg: &Box<dyn Any>| -> Fallible<Box<dyn Any>> {
            let arg = arg.downcast_ref().ok_or_else(|| err!(FailedCast))?;
            let res = function(arg);
            res.map(|o| Box::new(*o) as Box<dyn Any>)
        };
        Function::new_fallible(function)
    }
}

fn make_any_relation<QI: 'static, QO: 'static, AQI: Downcast, AQO: Downcast>(relation: &Rc<dyn Fn(&QI, &QO) -> Fallible<bool>>) -> impl Fn(&AQI, &AQO) -> Fallible<bool> + 'static {
    let relation = relation.clone();
    move |d_in: &AQI, d_out: &AQO| {
        let d_in = d_in.downcast_ref()?;
        let d_out = d_out.downcast_ref()?;
        relation(d_in, d_out)
    }
}

fn make_any_map<QI, QO, AQI>(map: &Option<Rc<dyn Fn(&QI) -> Fallible<Box<QO>>>>) -> Option<impl Fn(&AQI) -> Fallible<Box<AnyMetricDistance>>>
    where QI: 'static + PartialOrd,
          QO: 'static + PartialOrd + Clone,
          AQI: Downcast {
    map.as_ref().map(|map| {
        let map = map.clone();
        move |d_in: &AQI| -> Fallible<Box<AnyMetricDistance>> {
            let d_in = d_in.downcast_ref()?;
            let d_out = map(d_in);
            d_out.map(|d| AnyMetricDistance::new(*d)).map(Box::new)
        }
    })
}

impl<MI: Metric, MO: Measure> PrivacyRelation<MI, MO>
    where MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    pub fn as_any(&self) -> PrivacyRelation<AnyMetric, AnyMeasure> {
        PrivacyRelation::new_all(
            make_any_relation(&self.relation),
            make_any_map(&self.backward_map),
        )
    }
}

impl<MI: Metric, MO: Metric> StabilityRelation<MI, MO>
    where MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    pub fn as_any(&self) -> StabilityRelation<AnyMetric, AnyMetric> {
        StabilityRelation::new_all(
            make_any_relation(&self.relation),
            make_any_map(&self.forward_map),
            make_any_map(&self.backward_map),
        )
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Measure> Measurement<DI, DO, MI, MO>
    where DI::Carrier: 'static,
          DO::Carrier: 'static,
          MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    pub fn into_any_out(self) -> Measurement<DI, AnyDomain, MI, MO> {
        Measurement::new(
            *self.input_domain,
            AnyDomain::new(*self.output_domain),
            self.function.as_any_out(),
            *self.input_metric,
            *self.output_measure,
            self.privacy_relation,
        )
    }
    pub fn into_any(self) -> Measurement<AnyDomain, AnyDomain, AnyMetric, AnyMeasure> {
        Measurement::new(
            AnyDomain::new(*self.input_domain),
            AnyDomain::new(*self.output_domain),
            self.function.as_any(),
            AnyMetric::new(*self.input_metric),
            AnyMeasure::new(*self.output_measure),
            self.privacy_relation.as_any(),
        )
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric> Transformation<DI, DO, MI, MO>
    where DI::Carrier: 'static,
          DO::Carrier: 'static,
          MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    pub fn into_any_out(self) -> Transformation<DI, AnyDomain, MI, MO> {
        Transformation::new(
            *self.input_domain,
            AnyDomain::new(*self.output_domain),
            self.function.as_any_out(),
            *self.input_metric,
            *self.output_metric,
            self.stability_relation,
        )
    }
    pub fn into_any(self) -> Transformation<AnyDomain, AnyDomain, AnyMetric, AnyMetric> {
        Transformation::new(
            AnyDomain::new(*self.input_domain),
            AnyDomain::new(*self.output_domain),
            self.function.as_any(),
            AnyMetric::new(*self.input_metric),
            AnyMetric::new(*self.output_metric),
            self.stability_relation.as_any(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::dist::{HammingDistance, L2Sensitivity};
    use crate::error::*;
    use crate::meas;
    use crate::trans;

    use super::*;

    #[test]
    fn test_any_chain() -> Fallible<()> {
        let t1 = trans::make_split_dataframe::<HammingDistance, _>(None, vec!["a".to_owned(), "b".to_owned()])?;
        let t2 = trans::make_parse_column::<HammingDistance, _, f64>("a".to_owned(), true)?.into_any();
        let t3 = trans::make_select_column::<HammingDistance, _, f64>("a".to_owned())?.into_any();
        let t4 = trans::make_clamp_vec::<HammingDistance, _>(0.0, 10.0)?.into_any();
        let t5 = trans::make_bounded_sum::<HammingDistance, L2Sensitivity<_>>(0.0, 10.0)?.into_any();
        let m1 = meas::make_base_gaussian(0.0)?.into_any();
        let chain = (t1.into_any() >> t2 >> t3 >> t4 >> t5 >> m1)?;
        let arg = Box::new("1.0, 10.0\n2.0, 20.0\n3.0, 30.0\n".to_owned()) as Box<dyn Any>;
        let res = chain.function.eval(&arg);
        let res = *res?.downcast::<f64>().unwrap_test();
        assert_eq!(res, 6.0);

        Ok(())
    }
}
