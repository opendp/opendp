//! A collection of types and utilities for doing type erasure, using Any types and downcasting.
//! This makes it convenient to pass values over FFI, because generic types are erased, and everything
//! has a single concrete type.
//!
//! This is made possible by glue functions which can take the Any representation and downcast to the
//! correct concrete type.

use std::any;
use std::any::Any;
use std::rc::Rc;

use opendp::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use opendp::err;
use opendp::error::*;

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
    // clone_glue: Option<Glue<fn(&Self) -> Self>>,
    // eq_glue: Option<Glue<fn(&Self, &Self) -> bool>>,
    // partial_cmp_glue: Option<Glue<fn(&Self, &Self) -> Option<Ordering>>>,
    // checked_sub_glue: Option<Glue<fn(&Self, &Self) -> Option<Self>>>,
}

impl AnyObject {
    pub fn new<T: 'static>(value: T) -> Self {
        Self {
            type_: Type::of::<T>(),
            value: AnyBox::new(value),
            // clone_glue: None,
            // eq_glue: None,
            // partial_cmp_glue: None,
            // checked_sub_glue: None,
        }
    }

    #[cfg(test)]
    pub fn new_raw<T: 'static>(value: T) -> *mut Self {
        crate::util::into_raw(Self::new(value))
    }
}
// impl AnyObject {
//     pub fn has_clone<T: 'static + Clone>(&mut self) {
//         self.clone_glue = Some(Glue::new(|self_: &Self| -> Self {
//             Self::new(self_.downcast_ref::<T>().unwrap().clone())
//         }))
//     }
// }
// impl Clone for AnyObject {
//     fn clone(&self) -> Self {
//         self.clone_glue.as_ref().expect("missing clone glue")(self)
//     }
// }
//
// impl AnyObject {
//     pub fn has_partial_eq<T: 'static + PartialEq>(&mut self) {
//         self.eq_glue = Some(Glue::new(|self_: &Self, other: &Self| -> bool {
//             let self_ = self_.downcast_ref::<T>().unwrap();
//             let other = other.downcast_ref::<T>().unwrap();
//             self_.eq(other)
//         }));
//     }
// }
// impl PartialEq for AnyObject {
//     fn eq(&self, other: &AnyObject) -> bool {
//         self.eq_glue.as_ref().expect("missing eq glue")(self, other)
//     }
// }
//
// impl AnyObject {
//     pub fn has_partial_ord<T: 'static + PartialOrd>(&mut self) {
//         self.partial_cmp_glue = Some(Glue::new(|self_: &Self, other: &Self| -> Option<Ordering> {
//             // TODO: should this panic instead?
//             let self_ = self_.downcast_ref::<T>().ok()?;
//             let other = other.downcast_ref::<T>().ok()?;
//             self_.partial_cmp(other)
//         }));
//     }
// }
// impl PartialOrd for AnyObject {
//     fn partial_cmp(&self, other: &AnyObject) -> Option<Ordering> {
//         self.partial_cmp_glue.as_ref().expect("missing partial_cmp glue")(self, other)
//     }
// }
//
// impl AnyObject {
//     pub fn has_checked_sub<T: 'static + CheckedSub>(&mut self) {
//         self.checked_sub_glue = Some(Glue::new(|self_: &Self, rhs: &Self| -> Option<Self> {
//             let self_ = self_.downcast_ref::<T>().unwrap();
//             let rhs = rhs.downcast_ref::<T>().unwrap();
//             self_.checked_sub(&rhs).map(Self::new)
//         }))
//     }
// }
// impl CheckedSub for AnyObject {
//     fn checked_sub(&self, other: &AnyObject) -> Option<Self> {
//         self.checked_sub_glue.as_ref().expect("missing checked_sub glue")(self, other)
//     }
// }

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
    pub domain: AnyBoxClonePartialEq,
    member_glue: Glue<fn(&Self, &<Self as Domain>::Carrier) -> Fallible<bool>>,
    get_size_glue: Glue<fn(&Self) -> Fallible<usize>>,
}

impl AnyDomain {
    pub fn new<D: 'static + Domain + Clone + PartialEq>(domain: D) -> Self {
        Self {
            carrier_type: Type::of::<D::Carrier>(),
            domain: AnyBoxClonePartialEq::new_clone_partial_eq(domain),
            member_glue: Glue::new(|self_: &Self, val: &<Self as Domain>::Carrier| {
                let self_ = self_.downcast_ref::<D>()
                    .unwrap_assert("downcast of AnyDomain to constructed type will always work");
                self_.member(val.downcast_ref::<D::Carrier>()?)
            }),
            get_size_glue: Glue::new(|self_: &Self| {
                let self_: &D = self_.downcast_ref::<D>()
                    .unwrap_assert("downcast of AnyDomain to constructed type will always work");
                self_.as_sized_domain()?.get_size()
            })
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

#[derive(Clone, PartialEq)]
pub struct AnyMeasure {
    pub measure: AnyBoxClonePartialEq,
    pub distance_type: Type,
    amplify_glue: Glue<fn(&Self, &AnyObject, usize, usize) -> Fallible<AnyObject>>,
}

impl AnyMeasure {
    pub fn new<M: 'static + Measure + Clone + PartialEq>(measure: M) -> Self {
        Self {
            measure: AnyBoxClonePartialEq::new_clone_partial_eq(measure),
            distance_type: Type::of::<M::Distance>(),
            amplify_glue: Glue::new(|self_: &AnyMeasure, budget: &AnyObject, n_population: usize, n_sample: usize| -> Fallible<AnyObject> {
                let budget = budget.downcast_ref::<M::Distance>()?;
                self_.downcast_ref::<M>()?.to_amplifiable()?.amplify(budget, n_population, n_sample).map(AnyObject::new)
            })
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
    type Distance = AnyObject;
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
    type Distance = AnyObject;
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

fn make_any_map<QI, QO, AQI>(map: &Option<Rc<dyn Fn(&QI) -> Fallible<Box<QO>>>>) -> Option<impl Fn(&AQI) -> Fallible<Box<AnyObject>>>
    where QI: 'static,
          QO: 'static,
          AQI: Downcast {
    map.as_ref().map(|map| {
        let map = map.clone();
        move |d_in: &AQI| -> Fallible<Box<AnyObject>> {
            let d_in = d_in.downcast_ref()?;
            let d_out = map(d_in);
            d_out.map(|d| AnyObject::new(*d)).map(Box::new)
        }
    })
}

pub type AnyPrivacyRelation = PrivacyRelation<AnyMetric, AnyMeasure>;

pub trait IntoAnyPrivacyRelationExt {
    fn into_any(self) -> AnyPrivacyRelation;
}

impl<MI: Metric, MO: Measure> IntoAnyPrivacyRelationExt for PrivacyRelation<MI, MO>
    where MI::Distance: 'static,
          MO::Distance: 'static {
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

impl<DI: 'static + Domain + Clone + PartialEq, DO: 'static + Domain + Clone + PartialEq, MI: 'static + Metric, MO: 'static + Measure + Clone + PartialEq> IntoAnyMeasurementExt for Measurement<DI, DO, MI, MO>
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

impl<DO: 'static + Domain + Clone + PartialEq> IntoAnyMeasurementOutExt for Measurement<AnyDomain, DO, AnyMetric, AnyMeasure>
    where DO::Carrier: 'static {
    fn into_any_out(self) -> AnyMeasurement {
        AnyMeasurement::new(
            self.input_domain,
            AnyDomain::new(self.output_domain),
            self.function.into_any_out(),
            self.input_metric,
            self.output_measure,
            self.privacy_relation,
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

impl<DI: 'static + Domain + Clone + PartialEq, DO: 'static + Domain + Clone + PartialEq, MI: 'static + Metric, MO: 'static + Metric> IntoAnyTransformationExt for Transformation<DI, DO, MI, MO>
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
    use std::ops::Bound;

    use opendp::dist::{MaxDivergence, SmoothedMaxDivergence, SubstituteDistance, SymmetricDistance};
    use opendp::dom::{AllDomain, IntervalDomain};
    use opendp::error::*;
    use opendp::meas;
    use opendp::trans;

    use super::*;

    #[test]
    fn test_any_domain() -> Fallible<()> {
        let domain1 = IntervalDomain::new(Bound::Included(0), Bound::Included(1))?;
        let domain2 = IntervalDomain::new(Bound::Included(0), Bound::Included(1))?;
        // TODO: Add Debug to Domain so we can use assert_eq!.
        assert!(domain1 == domain2);

        let domain1 = AnyDomain::new(IntervalDomain::new(Bound::Included(0), Bound::Included(1))?);
        let domain2 = AnyDomain::new(IntervalDomain::new(Bound::Included(0), Bound::Included(1))?);
        let domain3 = AnyDomain::new(AllDomain::<i32>::new());
        assert!(domain1 == domain2);
        assert!(domain1 != domain3);

        let _domain1: IntervalDomain<i32> = domain1.downcast()?;
        let domain3: Fallible<IntervalDomain<i32>> = domain3.downcast();
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
        let t2 = trans::make_parse_column::<_, f64>("a".to_owned(), true)?.into_any();
        let t3 = trans::make_select_column::<_, f64>("a".to_owned())?.into_any();
        let t4 = trans::make_clamp(0.0, 10.0)?.into_any();
        let t5 = trans::make_bounded_sum(0.0, 10.0)?.into_any();
        let m1 = meas::make_base_gaussian::<AllDomain<_>>(0.0)?.into_any();
        let chain = (t1 >> t2 >> t3 >> t4 >> t5 >> m1)?;
        let arg = AnyObject::new("1.0, 10.0\n2.0, 20.0\n3.0, 30.0\n".to_owned());
        let res = chain.function.eval(&arg);
        let res: f64 = res?.downcast()?;
        assert_eq!(6.0, res);

        Ok(())
    }
}
