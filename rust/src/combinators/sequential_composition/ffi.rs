use crate::{
    combinators::Sequentiality,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{
        Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence, ffi::TypedMeasure,
    },
};

use super::{Adaptivity, CompositionMeasure};

impl CompositionMeasure for AnyMeasure {
    fn theorem(&self, adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            adaptivity: Adaptivity,
        ) -> Fallible<Sequentiality>
        where
            M::Distance: Clone,
        {
            self_.downcast_ref::<M>()?.theorem(adaptivity)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, adaptivity))
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            d_i: Vec<AnyObject>,
        ) -> Fallible<AnyObject>
        where
            M::Distance: Clone,
        {
            self_
                .downcast_ref::<M>()?
                .compose(
                    d_i.iter()
                        .map(|d_i| d_i.downcast_ref::<M::Distance>().map(Clone::clone))
                        .collect::<Fallible<Vec<M::Distance>>>()?,
                )
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, d_i))
    }
}

impl<Q: 'static> CompositionMeasure for TypedMeasure<Q> {
    fn theorem(&self, adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        self.measure.theorem(adaptivity)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        self.measure
            .compose(d_i.into_iter().map(AnyObject::new).collect())?
            .downcast()
    }
}
