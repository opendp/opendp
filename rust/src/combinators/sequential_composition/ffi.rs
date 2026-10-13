use crate::{
    combinators::Composability,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
};

use super::{Adaptivity, CompositionMeasure, MapKey};

impl CompositionMeasure for AnyMeasure {
    type Acc = AnyObject;

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
    fn compose(&self, d_i: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            d_i: Vec<(AnyObject, u32)>,
        ) -> Fallible<AnyObject>
        where
            M::Distance: Clone,
        {
            self_
                .downcast_ref::<M>()?
                .compose(
                    d_i.iter()
                        .map(|(d_i, k_i)| Ok((d_i.downcast_ref::<M::Distance>()?.clone(), *k_i)))
                        .collect::<Fallible<Vec<(M::Distance, u32)>>>()?,
                )
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, d_i))
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
        ) -> Fallible<AnyObject> {
            self_.downcast_ref::<M>()?.new_acc().map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self))
    }

    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            acc: &AnyObject,
        ) -> Fallible<AnyObject> {
            self_
                .downcast_ref::<M>()?
                .compose_acc(acc.downcast_ref::<M::Acc>()?)
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, acc))
    }

    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            acc: &mut AnyObject,
            key: MapKey,
            eval: Box<dyn FnOnce() -> Fallible<AnyObject> + '_>,
        ) -> Fallible<()> {
            self_
                .downcast_ref::<M>()?
                .accumulate_keyed(acc.downcast_mut::<M::Acc>()?, key, || {
                    eval()?.downcast::<M::Distance>()
                })
        }
        let eval: Box<dyn FnOnce() -> Fallible<AnyObject> + '_> = Box::new(eval);
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, acc, key, eval))
    }
}
