use crate::{
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{
        Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence, ffi::TypedMeasure,
    },
};

use super::SequentialCompositionMeasure;

impl SequentialCompositionMeasure for AnyMeasure {
    fn concurrent(&self) -> Fallible<bool> {
        fn monomorphize<M: 'static + SequentialCompositionMeasure>(
            self_: &AnyMeasure,
        ) -> Fallible<bool>
        where
            M::Distance: Clone,
        {
            self_.downcast_ref::<M>()?.concurrent()
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>])
        ], (self))
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + SequentialCompositionMeasure>(
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

impl<Q: 'static> SequentialCompositionMeasure for TypedMeasure<Q> {
    fn concurrent(&self) -> Fallible<bool> {
        self.measure.concurrent()
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        self.measure
            .compose(d_i.into_iter().map(AnyObject::new).collect())?
            .downcast()
    }
}
