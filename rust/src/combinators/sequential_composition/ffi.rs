use crate::{
    combinators::Composability,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
};

use super::{Adaptivity, ComposeK, CompositionMeasure};

impl CompositionMeasure for AnyMeasure {
    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            adaptivity: Adaptivity,
        ) -> Fallible<Composability>
        where
            M::Distance: Clone,
        {
            self_.downcast_ref::<M>()?.composability(adaptivity)
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

impl ComposeK for AnyMeasure {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance>
    where
        Self::Distance: Clone,
    {
        fn monomorphize<M: 'static + ComposeK>(
            self_: &AnyMeasure,
            d_mid: AnyObject,
            k: u32,
        ) -> Fallible<AnyObject>
        where
            M::Distance: Clone,
        {
            self_
                .downcast_ref::<M>()?
                .compose_k(d_mid.downcast_ref::<M::Distance>()?.clone(), k)
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, d_mid, k))
    }
}
