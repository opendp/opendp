use std::collections::Bound;
use std::marker::PhantomData;
use std::ops::{Div, Mul};

use num::One;

use crate::core::{DatasetMetric, Domain, Function, Metric, StabilityRelation, Transformation};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{CastFrom, DistanceCast};
use crate::trans::{MakeTransformation0, MakeTransformation2};


/// Constructs a [`Transformation`] representing the identity function.
pub struct Identity;

impl<D, T, M, Q> MakeTransformation2<D, D, M, M, D, M> for Identity
    where D: Domain<Carrier=T>, T: Clone,
          M: Metric<Distance=Q>, Q: 'static + Clone + Div<Output=Q> + Mul<Output=Q> + PartialOrd + DistanceCast + One {
    fn make2(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>> {
        Ok(Transformation::new(
            domain.clone(),
            domain,
            Function::new(|arg: &T| arg.clone()),
            metric.clone(),
            metric,
            StabilityRelation::new_from_constant(Q::one())))
    }
}

pub struct Clamp<M, T, Q> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
    distance: PhantomData<Q>
}

impl<M, T, Q> MakeTransformation2<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M, T, T> for Clamp<M, Vec<T>, Q>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            Function::new(move |arg: &Vec<T>| arg.into_iter().map(|e| clamp(&lower, &upper, e)).collect()),
            M::new(),
            M::new(),
            // clamping has a c-stability of one, as well as a lipschitz constant of one
            StabilityRelation::new_from_constant(Q::one())))
    }
}

impl<M, T, Q> MakeTransformation2<AllDomain<T>, IntervalDomain<T>, M, M, T, T> for Clamp<M, T, Q>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<AllDomain<T>, IntervalDomain<T>, M, M>> {
        Ok(Transformation::new(
            AllDomain::new(),
            IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone())),
            Function::new(move |arg: &T| clamp(&lower, &upper, arg)),
            M::new(),
            M::new(),
            // clamping has a c-stability of one, as well as a lipschitz constant of one
            StabilityRelation::new_from_constant(Q::one())))
    }
}

fn clamp<T: Clone + PartialOrd>(lower: &T, upper: &T, x: &T) -> T {
    (if x < &lower { lower } else if x > &upper { upper } else { x }).clone()
}

pub struct Unclamp<M, T, Q> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
    distance: PhantomData<Q>
}

impl<M, T, Q> MakeTransformation2<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M, T, T> for Unclamp<M, Vec<T>, Q>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + Default + DistanceCast + One + Div<Output=Q> + Mul<Output=Q> + PartialOrd {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
            VectorDomain::new_all(),
            Function::new(move |arg: &Vec<T>| arg.clone()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(Q::one())
        ))
    }
}

impl<M, T, Q> MakeTransformation2<IntervalDomain<T>, AllDomain<T>, M, M, Bound<T>, Bound<T>> for Unclamp<M, T, Q>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + Default + DistanceCast + One + Div<Output=Q> + Mul<Output=Q> + PartialOrd {
    fn make2(lower: Bound<T>, upper: Bound<T>) -> Fallible<Transformation<IntervalDomain<T>, AllDomain<T>, M, M>> {
        Ok(Transformation::new(
            IntervalDomain::new(lower, upper),
            AllDomain::new(),
            Function::new(move |arg: &T| arg.clone()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(Q::one())
        ))
    }
}


pub struct Cast<MI, MO, TI, TO> {
    metric_input: PhantomData<MI>,
    metric_output: PhantomData<MO>,
    data_input: PhantomData<TI>,
    data_output: PhantomData<TO>,
}

impl<M, TI, TO> MakeTransformation0<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, M, M> for Cast<M, M, Vec<TI>, Vec<TO>>
    where M: DatasetMetric<Distance=u32>,
          TI: Clone, TO: CastFrom<TI> + Default {
    fn make0() -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new(move |arg: &Vec<TI>| arg.into_iter()
                .map(|v| TO::cast(v.clone()).unwrap_or_else(|_| TO::default()))
                .collect()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

// casting primitive types is not exposed over ffi. Need a way to constrain that MI == MO, but MI::Distance may vary from MO::Distance
// impl<MI, MO, TI, TO> MakeTransformation0<AllDomain<TI>, AllDomain<TO>, MI, MO> for Cast<MI, MO, TI, TO>
//     where MI: SensitivityMetric<Distance=TI>,
//           MO: SensitivityMetric<Distance=TO>,
//           TI: Clone + DistanceCast, TO: 'static + CastFrom<TI> + Default + DistanceCast + One + Div<Output=TO> + Mul<Output=TO> + PartialOrd {
//     fn make0() -> Fallible<Transformation<AllDomain<TI>, AllDomain<TO>, MI, MO>> {
//         Ok(Transformation::new(
//             AllDomain::new(),
//             AllDomain::new(),
//             Function::new(move |v: &TI| TO::cast(v.clone()).unwrap_or_else(|_| TO::default())),
//             MI::new(),
//             MO::new(),
//             StabilityRelation::new_from_constant(TO::one())))
//     }
// }
