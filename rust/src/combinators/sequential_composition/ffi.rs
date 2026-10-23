use crate::{
    combinators::Composability,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
};

use super::{Adaptivity, CompositionMeasure, MapKey};

impl CompositionMeasure for AnyMeasure {
    type Acc = Option<AnyObject>;

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

    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            acc: &Option<AnyObject>,
        ) -> Fallible<AnyObject> {
            let self_ = self_.downcast_ref::<M>()?;
            match acc {
                Some(acc) => self_.compose_acc(acc.downcast_ref::<M::Acc>()?),
                None => self_.compose_acc(&M::Acc::default()),
            }
            .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, acc))
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            acc: &mut Option<AnyObject>,
            key: MapKey,
            eval: Box<dyn FnOnce() -> Fallible<AnyObject> + '_>,
        ) -> Fallible<()> {
            let acc = acc.get_or_insert_with(|| AnyObject::new(M::Acc::default()));
            self_
                .downcast_ref::<M>()?
                .accumulate(acc.downcast_mut::<M::Acc>()?, key, || {
                    eval()?.downcast::<M::Distance>()
                })
        }
        let eval: Box<dyn FnOnce() -> Fallible<AnyObject> + '_> = Box::new(eval);
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence])
        ], (self, acc, key, eval))
    }
}
