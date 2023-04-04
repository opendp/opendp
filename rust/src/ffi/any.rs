//! A collection of types and utilities for doing type erasure, using Any types and downcasting.
//! This makes it convenient to pass values over FFI, because generic types are erased, and everything
//! has a single concrete type.
//!
//! This is made possible by glue functions which can take the Any representation and downcast to the
//! correct concrete type.

use std::any;
use std::any::Any;
use std::fmt::{Debug, Formatter};

use crate::core::{
    Domain, Function, Measure, Measurement, Metric, PrivacyMap, StabilityMap, Transformation,
};
use crate::err;
use crate::error::*;

use super::glue::Glue;
use super::util::Type;

/// A trait for something that can be downcast to a concrete type.
pub trait Downcast {
    fn downcast<T: 'static>(self) -> Fallible<T>;
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T>;
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T>;
}

/// A struct wrapping a Box<dyn Any>, optionally implementing Clone and/or PartialEq.
pub struct AnyBoxBase<const CLONE: bool, const PARTIALEQ: bool, const DEBUG: bool> {
    pub value: Box<dyn Any>,
    clone_glue: Option<Glue<fn(&Self) -> Self>>,
    eq_glue: Option<Glue<fn(&Self, &Self) -> bool>>,
    debug_glue: Option<Glue<fn(&Self) -> String>>,
}

impl<const CLONE: bool, const PARTIALEQ: bool, const DEBUG: bool>
    AnyBoxBase<CLONE, PARTIALEQ, DEBUG>
{
    fn new_base<T: 'static>(
        value: T,
        clone_glue: Option<Glue<fn(&Self) -> Self>>,
        eq_glue: Option<Glue<fn(&Self, &Self) -> bool>>,
        debug_glue: Option<Glue<fn(&Self) -> String>>,
    ) -> Self {
        Self {
            value: Box::new(value),
            clone_glue,
            eq_glue,
            debug_glue,
        }
    }
    fn make_clone_glue<T: 'static + Clone>() -> Glue<fn(&Self) -> Self> {
        Glue::new(|self_: &Self| {
            Self::new_base(
                self_
                    .value
                    .downcast_ref::<T>()
                    .unwrap_assert("Failed downcast of AnyBox value")
                    .clone(),
                self_.clone_glue.clone(),
                self_.eq_glue.clone(),
                self_.debug_glue.clone(),
            )
        })
    }
    fn make_eq_glue<T: 'static + PartialEq>() -> Glue<fn(&Self, &Self) -> bool> {
        Glue::new(|self_: &Self, other: &Self| {
            // The first downcast will always succeed, so equality check is all that's necessary.
            self_.value.downcast_ref::<T>() == other.value.downcast_ref::<T>()
        })
    }
    fn make_debug_glue<T: 'static + Debug>() -> Glue<fn(&Self) -> String> {
        Glue::new(|self_: &Self| {
            format!(
                "{:?}",
                self_
                    .value
                    .downcast_ref::<T>()
                    .unwrap_assert("Failed downcast of AnyBox value")
            )
        })
    }
}

impl<const CLONE: bool, const PARTIALEQ: bool, const DEBUG: bool> Downcast
    for AnyBoxBase<CLONE, PARTIALEQ, DEBUG>
{
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "Failed downcast of AnyBox to {}",
                    any::type_name::<T>()
                )
            })
            .map(|x| *x)
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref().ok_or_else(|| {
            let other_type = Type::of_id(&self.value.type_id())
                .map(|t| format!(" AnyBox contains {:?}.", t))
                .unwrap_or_default();
            err!(
                FailedCast,
                "Failed downcast_ref of AnyBox to {}.{}",
                any::type_name::<T>(),
                other_type
            )
        })
    }
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T> {
        let type_id = self.value.type_id();
        self.value.downcast_mut().ok_or_else(|| {
            let other_type = Type::of_id(&type_id)
                .map(|t| format!(" AnyBox contains {:?}.", t))
                .unwrap_or_default();
            err!(
                FailedCast,
                "Failed downcast_mut of AnyBox to {}.{}",
                any::type_name::<T>(),
                other_type
            )
        })
    }
}

impl<const PARTIALEQ: bool, const DEBUG: bool> Clone for AnyBoxBase<true, PARTIALEQ, DEBUG> {
    fn clone(&self) -> Self {
        (self
            .clone_glue
            .as_ref()
            .unwrap_assert("clone_glue always exists for CLONE=true AnyBoxBase"))(self)
    }
}

impl<const CLONE: bool, const DEBUG: bool> PartialEq for AnyBoxBase<CLONE, true, DEBUG> {
    fn eq(&self, other: &Self) -> bool {
        (self
            .eq_glue
            .as_ref()
            .unwrap_assert("eq_glue always exists for PARTIALEQ=true AnyBoxBase"))(
            self, other
        )
    }
}

impl<const CLONE: bool, const PARTIALEQ: bool> Debug for AnyBoxBase<CLONE, PARTIALEQ, true> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            (self
                .debug_glue
                .as_ref()
                .unwrap_assert("debug_glue always exists for DEBUG=true AnyBoxBase"))(
                self
            )
        )
    }
}

/// An AnyBox not implementing optional traits.
pub type AnyBox = AnyBoxBase<false, false, false>;

impl AnyBox {
    pub fn new<T: 'static>(value: T) -> Self {
        Self::new_base(value, None, None, None)
    }
}

pub type AnyBoxClonePartialEqDebug = AnyBoxBase<true, true, true>;

impl AnyBoxClonePartialEqDebug {
    pub fn new_clone_partial_eq_debug<T: 'static + Clone + PartialEq + Debug>(value: T) -> Self {
        Self::new_base(
            value,
            Some(Self::make_clone_glue::<T>()),
            Some(Self::make_eq_glue::<T>()),
            Some(Self::make_debug_glue::<T>()),
        )
    }
}

/// A struct that can wrap any object.
pub struct AnyObject {
    pub type_: Type,
    value: AnyBox,
}

impl AnyObject {
    pub fn new<T: 'static>(value: T) -> Self {
        AnyObject {
            type_: Type::of::<T>(),
            value: AnyBox::new(value),
        }
    }

    #[cfg(test)]
    pub fn new_raw<T: 'static>(value: T) -> *mut Self {
        crate::ffi::util::into_raw(Self::new(value))
    }
}

impl Downcast for AnyObject {
    fn downcast<T: 'static>(self) -> Fallible<T> {
        self.value.downcast()
    }
    fn downcast_ref<T: 'static>(&self) -> Fallible<&T> {
        self.value.downcast_ref()
    }
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T> {
        self.value.downcast_mut()
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyDomain {
    pub type_: Type,
    pub carrier_type: Type,
    pub domain: AnyBoxClonePartialEqDebug,
    member_glue: Glue<fn(&Self, &<Self as Domain>::Carrier) -> Fallible<bool>>,
}

impl AnyDomain {
    pub fn new<D: 'static + Domain>(domain: D) -> Self {
        Self {
            type_: Type::of::<D>(),
            carrier_type: Type::of::<D::Carrier>(),
            domain: AnyBoxClonePartialEqDebug::new_clone_partial_eq_debug(domain),
            member_glue: Glue::new(|self_: &Self, val: &<Self as Domain>::Carrier| {
                let self_ = self_
                    .downcast_ref::<D>()
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
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T> {
        self.domain.downcast_mut()
    }
}

impl Domain for AnyDomain {
    type Carrier = AnyObject;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        (self.member_glue)(self, val)
    }
}

impl Debug for AnyDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.domain)
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyMeasure {
    pub measure: AnyBoxClonePartialEqDebug,
    pub type_: Type,
    pub distance_type: Type,
}

impl AnyMeasure {
    pub fn new<M: 'static + Measure>(measure: M) -> Self {
        Self {
            measure: AnyBoxClonePartialEqDebug::new_clone_partial_eq_debug(measure),
            type_: Type::of::<M>(),
            distance_type: Type::of::<M::Distance>(),
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
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T> {
        self.measure.downcast_mut()
    }
}

impl Default for AnyMeasure {
    fn default() -> Self {
        unimplemented!("called AnyMeasure::default()")
    }
}

impl Measure for AnyMeasure {
    type Distance = AnyObject;
}

impl Debug for AnyMeasure {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.measure.fmt(f)
    }
}

#[derive(Clone, PartialEq)]
pub struct AnyMetric {
    pub type_: Type,
    pub distance_type: Type,
    pub metric: AnyBoxClonePartialEqDebug,
}

impl AnyMetric {
    pub fn new<M: 'static + Metric>(metric: M) -> Self {
        Self {
            type_: Type::of::<M>(),
            distance_type: Type::of::<M::Distance>(),
            metric: AnyBoxClonePartialEqDebug::new_clone_partial_eq_debug(metric),
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
    fn downcast_mut<T: 'static>(&mut self) -> Fallible<&mut T> {
        self.metric.downcast_mut()
    }
}

impl Default for AnyMetric {
    fn default() -> Self {
        unimplemented!("called AnyMetric::default()")
    }
}

impl Metric for AnyMetric {
    type Distance = AnyObject;
}

impl Debug for AnyMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.metric.fmt(f)
    }
}

pub(crate) type AnyFunction = Function<AnyObject, AnyObject>;

pub trait IntoAnyFunctionExt {
    fn into_any(self) -> AnyFunction;
}

impl<TI, TO> IntoAnyFunctionExt for Function<TI, TO>
where
    TI: 'static,
    TO: 'static,
{
    fn into_any(self) -> AnyFunction {
        let function = move |arg: &AnyObject| -> Fallible<AnyObject> {
            let arg = arg.downcast_ref()?;
            let res = self.eval(arg);
            res.map(AnyObject::new)
        };
        Function::new_fallible(function)
    }
}

pub trait IntoAnyFunctionOutExt {
    fn into_any_out(self) -> AnyFunction;
}

impl<TO: 'static> IntoAnyFunctionOutExt for Function<AnyObject, TO> {
    fn into_any_out(self) -> AnyFunction {
        let function = move |arg: &AnyObject| -> Fallible<AnyObject> {
            let res = self.eval(arg);
            res.map(AnyObject::new)
        };
        Function::new_fallible(function)
    }
}

pub type AnyPrivacyMap = PrivacyMap<AnyMetric, AnyMeasure>;

pub trait IntoAnyPrivacyMapExt {
    fn into_any(self) -> AnyPrivacyMap;
}

impl<MI: Metric, MO: Measure> IntoAnyPrivacyMapExt for PrivacyMap<MI, MO>
where
    MI::Distance: 'static,
    MO::Distance: 'static,
{
    fn into_any(self) -> AnyPrivacyMap {
        let map = self.0;
        AnyPrivacyMap::new_fallible(move |d_in| map(d_in.downcast_ref()?).map(AnyObject::new))
    }
}

pub type AnyStabilityMap = StabilityMap<AnyMetric, AnyMetric>;

pub trait IntoAnyStabilityMapExt {
    fn into_any(self) -> AnyStabilityMap;
}

impl<MI: Metric, MO: Metric> IntoAnyStabilityMapExt for StabilityMap<MI, MO>
where
    MI::Distance: 'static,
    MO::Distance: 'static,
{
    fn into_any(self) -> AnyStabilityMap {
        let map = self.0;
        AnyStabilityMap::new_fallible(move |d_in| map(d_in.downcast_ref()?).map(AnyObject::new))
    }
}

/// A Measurement with all generic types filled by Any types. This is the type of Measurements
/// passed back and forth over FFI.
pub type AnyMeasurement = Measurement<AnyDomain, AnyObject, AnyMetric, AnyMeasure>;

/// A trait for turning a Measurement into an AnyMeasurement. We can't used From because it'd conflict
/// with blanket implementation, and we need an extension trait to add methods to Measurement.
pub trait IntoAnyMeasurementExt {
    fn into_any(self) -> AnyMeasurement;
}

impl<DI: 'static + Domain, TO: 'static, MI: 'static + Metric, MO: 'static + Measure>
    IntoAnyMeasurementExt for Measurement<DI, TO, MI, MO>
where
    DI::Carrier: 'static,
    MI::Distance: 'static,
    MO::Distance: 'static,
{
    fn into_any(self) -> AnyMeasurement {
        AnyMeasurement::new(
            AnyDomain::new(self.input_domain),
            self.function.into_any(),
            AnyMetric::new(self.input_metric),
            AnyMeasure::new(self.output_measure),
            self.privacy_map.into_any(),
        )
    }
}

/// A trait for turning a Measurement into an AnyMeasurement, when only the output side needs to be wrapped.
/// Used for composition.
pub trait IntoAnyMeasurementOutExt {
    fn into_any_out(self) -> AnyMeasurement;
}

impl<TO: 'static> IntoAnyMeasurementOutExt for Measurement<AnyDomain, TO, AnyMetric, AnyMeasure> {
    fn into_any_out(self) -> AnyMeasurement {
        AnyMeasurement::new(
            self.input_domain,
            self.function.into_any_out(),
            self.input_metric,
            self.output_measure,
            self.privacy_map,
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

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric>
    IntoAnyTransformationExt for Transformation<DI, DO, MI, MO>
where
    DI::Carrier: 'static,
    DO::Carrier: 'static,
    MI::Distance: 'static,
    MO::Distance: 'static,
{
    fn into_any(self) -> AnyTransformation {
        AnyTransformation::new(
            AnyDomain::new(self.input_domain),
            AnyDomain::new(self.output_domain),
            self.function.into_any(),
            AnyMetric::new(self.input_metric),
            AnyMetric::new(self.output_metric),
            self.stability_map.into_any(),
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::domains::AtomDomain;
    use crate::error::*;
    use crate::measurements;
    use crate::measures::{MaxDivergence, SmoothedMaxDivergence};
    use crate::metrics::{ChangeOneDistance, SymmetricDistance};
    use crate::transformations;

    use super::*;

    #[test]
    fn test_any_domain() -> Fallible<()> {
        let domain1 = AtomDomain::new_closed((0, 1))?;
        let domain2 = AtomDomain::new_closed((0, 1))?;
        assert_eq!(domain1, domain2);

        let domain1 = AnyDomain::new(AtomDomain::new_closed((0, 1))?);
        let domain2 = AnyDomain::new(AtomDomain::new_closed((0, 1))?);
        let domain3 = AnyDomain::new(AtomDomain::<i32>::default());
        assert_eq!(domain1, domain2);
        assert_ne!(domain1, domain3);

        let _domain1: AtomDomain<i32> = domain1.downcast()?;
        let domain3: Fallible<AtomDomain<i64>> = domain3.downcast();
        assert_eq!(
            domain3.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }

    #[test]
    fn test_any_metric() -> Fallible<()> {
        let metric1 = SymmetricDistance::default();
        let metric2 = SymmetricDistance::default();
        assert_eq!(metric1, metric2);

        let metric1 = AnyMetric::new(SymmetricDistance::default());
        let metric2 = AnyMetric::new(SymmetricDistance::default());
        let metric3 = AnyMetric::new(ChangeOneDistance::default());
        assert_eq!(metric1, metric2);
        assert_ne!(metric1, metric3);

        let _metric1: SymmetricDistance = metric1.downcast()?;
        let metric3: Fallible<SymmetricDistance> = metric3.downcast();
        assert_eq!(
            metric3.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }

    #[test]
    fn test_any_measure() -> Fallible<()> {
        let measure1 = MaxDivergence::<f64>::default();
        let measure2 = MaxDivergence::<f64>::default();
        assert_eq!(measure1, measure2);

        let measure1 = AnyMeasure::new(MaxDivergence::<f64>::default());
        let measure2 = AnyMeasure::new(MaxDivergence::<f64>::default());
        let measure3 = AnyMeasure::new(SmoothedMaxDivergence::<f64>::default());
        assert_eq!(measure1, measure2);
        assert_ne!(measure1, measure3);

        let _measure1: MaxDivergence<f64> = measure1.downcast()?;
        let measure3: Fallible<MaxDivergence<f64>> = measure3.downcast();
        assert_eq!(
            measure3.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }

    #[test]
    fn test_any_chain() -> Fallible<()> {
        let t1 = transformations::make_split_dataframe(None, vec!["a".to_owned(), "b".to_owned()])?
            .into_any();
        let t2 = transformations::make_select_column::<_, String>("a".to_owned())?.into_any();
        let t3 = transformations::make_cast_default::<String, f64>()?.into_any();
        let t4 = transformations::make_clamp((0.0f64, 10.0))?.into_any();
        let t5 = transformations::make_bounded_sum::<SymmetricDistance, _>((0.0, 10.0))?.into_any();
        let m1 = measurements::make_base_laplace::<AtomDomain<_>>(
            0.0, None,
        )?
        .into_any();
        let chain = (t1 >> t2 >> t3 >> t4 >> t5 >> m1)?;
        let arg = AnyObject::new("1.0, 10.0\n2.0, 20.0\n3.0, 30.0\n".to_owned());
        let res = chain.invoke(&arg);
        let res: f64 = res?.downcast()?;
        assert_eq!(6.0, res);

        Ok(())
    }
}
