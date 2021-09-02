//! A collection of types and utilities for doing type erasure, using Any types and downcasting.
//! This makes it convenient to pass values over FFI, because generic types are erased, and everything
//! has a single concrete type.
//!
//! This is made possible by glue functions which can take the Any representation and downcast to the
//! correct concrete type.

use std::any;
use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;

use opendp::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use opendp::err;
use opendp::error::*;
use opendp::traits::{FallibleSub, MeasureDistance, MetricDistance};

use crate::glue::Glue;
use crate::util::Type;

/// A trait for something that can be downcast to a concrete type.
pub trait Downcast {
    fn downcast<T: 'static>(self) -> Fallible<T>;
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T>;
}

/// A struct wrapping a Box<dyn Any>, optionally implementing Clone and/or PartialEq.
pub struct AnyBoxBase<const CLONE: bool, const PARTIALEQ: bool> {
    pub value: Box<dyn Any>,
    clone_glue: Option<Glue<fn(&Self) -> Self>>,
    eq_glue: Option<Glue<fn(&Self, &Self) -> bool>>,
}

impl<const CLONE: bool, const PARTIALEQ: bool> AnyBoxBase<CLONE, PARTIALEQ> {
    fn new_base<T: 'static>(value: T, clone_glue: Option<Glue<fn(&Self) -> Self>>, eq_glue: Option<Glue<fn(&Self, &Self) -> bool>>) -> Self {
        Self { value: Box::new(value), clone_glue, eq_glue }
    }
    fn make_clone_glue<T: 'static + Clone>() -> Option<Glue<fn(&Self) -> Self>> {
        Some(Glue::new(|self_: &Self| {
            Self::new_base(
                self_.value.downcast_ref::<T>().unwrap_assert("Failed downcast of AnyBox value").clone(),
                self_.clone_glue.clone(),
                self_.eq_glue.clone(),
            )
        }))
    }
    fn make_eq_glue<T: 'static + PartialEq>() -> Option<Glue<fn(&Self, &Self) -> bool>> {
        Some(Glue::new(|self_: &Self, other: &Self| {
            // The first downcast will always succeed, so equality check is all that's necessary.
            self_.value.downcast_ref::<T>() == other.value.downcast_ref::<T>()
        }))
    }
}

impl<const CLONE: bool, const PARTIALEQ: bool> Downcast for AnyBoxBase<CLONE, PARTIALEQ> {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value.downcast().map_err(|_| err!(FailedCast, "Failed downcast of AnyBox to {}", any::type_name::<T>())).map(|x| *x)
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref().ok_or_else(|| err!(FailedCast, "Failed downcast_ref of AnyBox to {}", any::type_name::<T>()))
    }
}

impl<const PARTIALEQ: bool> Clone for AnyBoxBase<true, PARTIALEQ> {
    fn clone(&self) -> Self {
        (self.clone_glue.as_ref().unwrap_assert("No clone_glue for AnyBox"))(&self)
    }
}

impl<const CLONE: bool> PartialEq for AnyBoxBase<CLONE, true> {
    fn eq(&self, other: &Self) -> bool {
        (self.eq_glue.as_ref().unwrap_assert("No eq_glue for AnyBox"))(self, other)
    }
}

/// An AnyBox not implementing optional traits.
pub type AnyBox = AnyBoxBase<false, false>;

impl AnyBox {
    pub fn new<T: 'static>(value: T) -> Self {
        Self::new_base(value, None, None)
    }
}

/// An AnyBox implementing Clone.
pub type AnyBoxClone = AnyBoxBase<true, false>;

impl AnyBoxClone {
    pub fn new_clone<T: 'static + Clone>(value: T) -> Self {
        Self::new_base(value, Self::make_clone_glue::<T>(), None)
    }
}

/// An AnyBox implementing PartialEq.
pub type AnyBoxPartialEq = AnyBoxBase<false, true>;

impl AnyBoxPartialEq {
    pub fn new_partial_eq<T: 'static + PartialEq>(value: T) -> Self {
        Self::new_base(value, None, Self::make_eq_glue::<T>())
    }
}

/// An AnyBox implementing Clone + PartialEq.
pub type AnyBoxClonePartialEq = AnyBoxBase<true, true>;

impl AnyBoxClonePartialEq {
    pub fn new_clone_partial_eq<T: 'static + Clone + PartialEq>(value: T) -> Self {
        Self::new_base(value, Self::make_clone_glue::<T>(), Self::make_eq_glue::<T>())
    }
}

/// A struct that can wrap any object.
pub struct AnyObject {
    pub type_: Type,
    value: AnyBox,
}

impl AnyObject {
    pub fn new<T: 'static>(value: T) -> Self {
        Self { type_: Type::of::<T>(), value: AnyBox::new(value) }
    }

    #[cfg(test)]
    pub fn new_raw<T: 'static>(value: T) -> *mut Self {
        crate::util::into_raw(Self::new(value))
    }
}

impl Downcast for AnyObject {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref()
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyDomain {
    pub carrier_type: Type,
    domain: AnyBoxClonePartialEq,
    member_glue: Glue<fn(&Self, &<Self as Domain>::Carrier) -> Fallible<bool>>,
}

impl AnyDomain {
    pub fn new<D: 'static + Domain>(domain: D) -> Self {
        Self {
            carrier_type: Type::of::<D::Carrier>(),
            domain: AnyBoxClonePartialEq::new_clone_partial_eq(domain),
            member_glue: Glue::new(|self_: &Self, val: &<Self as Domain>::Carrier| {
                let self_ = self_.downcast_ref::<D>()
                    .unwrap_assert("downcast of AnyDomain to constructed type will always work");
                self_.member(val.downcast_ref::<D::Carrier>()?)
            }),
        }
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

impl Domain for AnyDomain {
    type Carrier = AnyObject;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        (self.member_glue)(self, val)
    }
}

// TODO: If/when we remove the clone of the budget from make_adaptive_composition(), then remove Clone from AnyXXXDistance.
#[derive(Clone, PartialEq)]
pub struct AnyMeasureDistance {
    distance: AnyBoxClonePartialEq,
    partial_cmp_glue: Glue<fn(&Self, &Self) -> Option<Ordering>>,
    sub_glue: Glue<fn(Self, &Self) -> Fallible<Self>>,
}

impl AnyMeasureDistance {
    pub fn new<Q: 'static + Clone + MeasureDistance>(distance: Q) -> Self {
        Self {
            distance: AnyBoxClonePartialEq::new_clone_partial_eq(distance),
            partial_cmp_glue: Glue::new(|self_: &Self, other: &Self| -> Option<Ordering> {
                let self_ = self_.downcast_ref::<Q>().unwrap_assert("downcast of AnyMeasureDistance to constructed type will always work");
                let other = other.downcast_ref::<Q>().ok()?;
                // FIXME: Do we want to have a FalliblePartialCmp for this?
                self_.partial_cmp(other)
            }),
            sub_glue: Glue::new(|self_: Self, rhs: &Self| -> Fallible<Self> {
                let distance = self_.downcast::<Q>()?;
                let rhs = rhs.downcast_ref::<Q>()?;
                let res = distance.sub(rhs);
                res.map(Self::new)
            }),
        }
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

// TODO: If/when we remove the clone of the budget from make_adaptive_composition(), then remove Clone from AnyXXXDistance.
#[derive(Clone, PartialEq)]
pub struct AnyMetricDistance {
    distance: AnyBoxClonePartialEq,
    partial_cmp_glue: Glue<fn(&Self, &Self) -> Option<Ordering>>,
}

impl AnyMetricDistance {
    pub fn new<Q: 'static + Clone + MetricDistance>(distance: Q) -> Self {
        Self {
            distance: AnyBoxClonePartialEq::new_clone_partial_eq(distance),
            partial_cmp_glue: Glue::new(|self_: &Self, other: &Self| -> Option<Ordering> {
                let self_ = self_.downcast_ref::<Q>().unwrap_assert("downcast of AnyMeasureDistance to constructed type will always work");
                let other = other.downcast_ref::<Q>();
                // FIXME: Do we want to have a FalliblePartialCmp for this?
                other.map_or(None, |o| self_.partial_cmp(o))
            }),
        }
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

impl PartialOrd for AnyMetricDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.partial_cmp_glue)(self, other)
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyMeasure {
    pub measure: AnyBoxClonePartialEq,
    pub distance_type: Type
}

impl AnyMeasure {
    pub fn new<M: 'static + Measure>(measure: M) -> Self {
        Self {
            measure: AnyBoxClonePartialEq::new_clone_partial_eq(measure),
            distance_type: Type::of::<M::Distance>()
        }
    }
}

impl Downcast for AnyMeasure {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.measure.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.measure.downcast_ref()
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
    pub metric: AnyBoxClonePartialEq,
    pub distance_type: Type
}

impl AnyMetric {
    pub fn new<M: 'static + Metric>(metric: M) -> Self {
        Self {
            metric: AnyBoxClonePartialEq::new_clone_partial_eq(metric),
            distance_type: Type::of::<M::Distance>()
        }
    }
}

impl Downcast for AnyMetric {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.metric.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.metric.downcast_ref()
    }
}

impl Default for AnyMetric {
    fn default() -> Self { unimplemented!("called AnyMetric::default()") }
}

impl Metric for AnyMetric {
    type Distance = AnyMetricDistance;
}

type AnyFunction = Function<AnyDomain, AnyDomain>;

pub trait IntoAnyFunctionExt {
    fn into_any(self) -> AnyFunction;
}

impl<DI, DO> IntoAnyFunctionExt for Function<DI, DO>
    where DI: 'static + Domain,
          DI::Carrier: 'static,
          DO: 'static + Domain,
          DO::Carrier: 'static {
    fn into_any(self) -> AnyFunction {
        let function = move |arg: &<AnyDomain as Domain>::Carrier| -> Fallible<<AnyDomain as Domain>::Carrier> {
            let arg = arg.downcast_ref()?;
            let res = self.eval(arg);
            res.map(|o| AnyObject::new(o))
        };
        Function::new_fallible(function)
    }
}

pub trait IntoAnyFunctionOutExt {
    fn into_any_out(self) -> AnyFunction;
}

impl<DO> IntoAnyFunctionOutExt for Function<AnyDomain, DO>
    where DO: 'static + Domain,
          DO::Carrier: 'static {
    fn into_any_out(self) -> AnyFunction {
        let function = move |arg: &<AnyDomain as Domain>::Carrier| -> Fallible<<AnyDomain as Domain>::Carrier> {
            let res = self.eval(arg);
            res.map(|o| AnyObject::new(o))
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

pub type AnyPrivacyRelation = PrivacyRelation<AnyMetric, AnyMeasure>;

pub trait IntoAnyPrivacyRelationExt {
    fn into_any(self) -> AnyPrivacyRelation;
}

impl<MI: Metric, MO: Measure> IntoAnyPrivacyRelationExt for PrivacyRelation<MI, MO>
    where MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    fn into_any(self) -> AnyPrivacyRelation {
        AnyPrivacyRelation::new_all(
            make_any_relation(&self.relation),
            make_any_map(&self.backward_map),
        )
    }
}

pub type AnyStabilityRelation = StabilityRelation<AnyMetric, AnyMetric>;

pub trait IntoAnyStabilityRelationExt {
    fn into_any(self) -> AnyStabilityRelation;
}

impl<MI: Metric, MO: Metric> IntoAnyStabilityRelationExt for StabilityRelation<MI, MO>
    where MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    fn into_any(self) -> AnyStabilityRelation {
        AnyStabilityRelation::new_all(
            make_any_relation(&self.relation),
            make_any_map(&self.forward_map),
            make_any_map(&self.backward_map),
        )
    }
}

/// A Measurement with all generic types filled by Any types. This is the type of Measurements
/// passed back and forth over FFI.
pub type AnyMeasurement = Measurement<AnyDomain, AnyDomain, AnyMetric, AnyMeasure>;

/// A trait for turning a Measurement into an AnyMeasurement. We can't used From because it'd conflict
/// with blanket implementation, and we need an extension trait to add methods to Measurement.
pub trait IntoAnyMeasurementExt {
    fn into_any(self) -> AnyMeasurement;
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Measure> IntoAnyMeasurementExt for Measurement<DI, DO, MI, MO>
    where DI::Carrier: 'static,
          DO::Carrier: 'static,
          MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    fn into_any(self) -> AnyMeasurement {
        AnyMeasurement::new(
            AnyDomain::new(self.input_domain),
            AnyDomain::new(self.output_domain),
            self.function.into_any(),
            AnyMetric::new(self.input_metric),
            AnyMeasure::new(self.output_measure),
            self.privacy_relation.into_any(),
        )
    }
}

/// A trait for turning a Measurement into an AnyMeasurement, when only the output side needs to be wrapped.
/// Used for composition.
pub trait IntoAnyMeasurementOutExt {
    fn into_any_out(self) -> AnyMeasurement;
}

impl<DO: 'static + Domain> IntoAnyMeasurementOutExt for Measurement<AnyDomain, DO, AnyMetric, AnyMeasure>
    where DO::Carrier: 'static {
    fn into_any_out(self) -> AnyMeasurement {
        AnyMeasurement::new(
            self.input_domain,
            AnyDomain::new(self.output_domain),
            self.function.into_any_out(),
            self.input_metric,
            self.output_measure,
            self.privacy_relation.into_any(),
        )
    }
}

/// A Transformation with all generic types filled by Any types. This is the type of Transformation
/// passed back and forth over FFI.
pub type AnyTransformation = Transformation<AnyDomain, AnyDomain, AnyMetric, AnyMetric>;

/// A trait for turning a Transformation into an AnyTransformation. We can't used From because it'd conflict
/// with blanket implementation, and we need an extension trait to add methods to Measurement.
pub trait IntoAnyTransformationExt {
    fn into_any(self) -> AnyTransformation;
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric> IntoAnyTransformationExt for Transformation<DI, DO, MI, MO>
    where DI::Carrier: 'static,
          DO::Carrier: 'static,
          MI::Distance: 'static + Clone + PartialOrd,
          MO::Distance: 'static + Clone + PartialOrd {
    fn into_any(self) -> AnyTransformation {
        AnyTransformation::new(
            AnyDomain::new(self.input_domain),
            AnyDomain::new(self.output_domain),
            self.function.into_any(),
            AnyMetric::new(self.input_metric),
            AnyMetric::new(self.output_metric),
            self.stability_relation.into_any(),
        )
    }
}

#[cfg(test)]
mod tests {
    use opendp::dist::{SubstituteDistance, MaxDivergence, SmoothedMaxDivergence, SymmetricDistance};
    use opendp::dom::{AllDomain, BoundedDomain};
    use opendp::error::*;
    use opendp::meas;
    use opendp::trans;

    use super::*;

    #[test]
    fn test_any_domain() -> Fallible<()> {
        let domain1 = BoundedDomain::new_closed((0, 1))?;
        let domain2 = BoundedDomain::new_closed((0, 1))?;
        // TODO: Add Debug to Domain so we can use assert_eq!.
        assert!(domain1 == domain2);

        let domain1 = AnyDomain::new(BoundedDomain::new_closed((0, 1))?);
        let domain2 = AnyDomain::new(BoundedDomain::new_closed((0, 1))?);
        let domain3 = AnyDomain::new(AllDomain::<i32>::new());
        assert!(domain1 == domain2);
        assert!(domain1 != domain3);

        let _domain1: BoundedDomain<i32> = domain1.downcast()?;
        let domain3: Fallible<BoundedDomain<i32>> = domain3.downcast();
        assert_eq!(domain3.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_any_metric() -> Fallible<()> {
        let metric1 = SymmetricDistance::default();
        let metric2 = SymmetricDistance::default();
        // TODO: Add Debug to Metric so we can use assert_eq!.
        assert!(metric1 == metric2);

        let metric1 = AnyMetric::new(SymmetricDistance::default());
        let metric2 = AnyMetric::new(SymmetricDistance::default());
        let metric3 = AnyMetric::new(SubstituteDistance::default());
        assert!(metric1 == metric2);
        assert!(metric1 != metric3);

        let _metric1: SymmetricDistance = metric1.downcast()?;
        let metric3: Fallible<SymmetricDistance> = metric3.downcast();
        assert_eq!(metric3.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_any_measure() -> Fallible<()> {
        let measure1 = MaxDivergence::<f64>::default();
        let measure2 = MaxDivergence::<f64>::default();
        // TODO: Add Debug to Measure so we can use assert_eq!.
        assert!(measure1 == measure2);

        let measure1 = AnyMeasure::new(MaxDivergence::<f64>::default());
        let measure2 = AnyMeasure::new(MaxDivergence::<f64>::default());
        let measure3 = AnyMeasure::new(SmoothedMaxDivergence::<f64>::default());
        assert!(measure1 == measure2);
        assert!(measure1 != measure3);

        let _measure1: MaxDivergence<f64> = measure1.downcast()?;
        let measure3: Fallible<MaxDivergence<f64>> = measure3.downcast();
        assert_eq!(measure3.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_any_chain() -> Fallible<()> {
        let t1 = trans::make_split_dataframe(None, vec!["a".to_owned(), "b".to_owned()])?.into_any();
        let t2 = trans::make_select_column::<_, String>("a".to_owned())?.into_any();
        let t3 = trans::make_cast_default::<String, f64>()?.into_any();
        let t4 = trans::make_clamp((0.0, 10.0))?.into_any();
        let t5 = trans::make_bounded_sum((0.0, 10.0))?.into_any();
        let m1 = meas::make_base_gaussian::<AllDomain<_>>(0.0)?.into_any();
        let chain = (t1 >> t2 >> t3 >> t4 >> t5 >> m1)?;
        let arg = AnyObject::new("1.0, 10.0\n2.0, 20.0\n3.0, 30.0\n".to_owned());
        let res = chain.function.eval(&arg);
        let res: f64 = res?.downcast()?;
        assert_eq!(6.0, res);

        Ok(())
    }
}
