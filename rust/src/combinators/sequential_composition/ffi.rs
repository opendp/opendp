use crate::{
    combinators::Composability,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{Approximate, MultiDP, PureDP, RenyiDP, zCDP},
};

use super::{Adaptivity, CompositionMeasure};

impl CompositionMeasure for AnyMeasure {
    fn compose_measure(measures: &[Self]) -> Fallible<Self> {
        let Some(first) = measures.first() else {
            return fallible!(MakeMeasurement, "Must have at least one measurement");
        };

        fn monomorphize<M: 'static + CompositionMeasure>(
            _first: &AnyMeasure,
            measures: &[AnyMeasure],
        ) -> Fallible<AnyMeasure>
        where
            M::Distance: Clone,
        {
            let measures = measures
                .iter()
                .map(|measure| measure.downcast_ref::<M>().map(Clone::clone))
                .collect::<Fallible<Vec<M>>>()?;
            M::compose_measure(&measures).map(AnyMeasure::new)
        }

        dispatch!(monomorphize, [
            (first.type_, [PureDP, Approximate<PureDP>, zCDP, Approximate<zCDP>, RenyiDP, MultiDP])
        ], (first, measures))
    }

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
            (self.type_, [PureDP, Approximate<PureDP>, zCDP, Approximate<zCDP>, RenyiDP, MultiDP])
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
            (self.type_, [PureDP, Approximate<PureDP>, zCDP, Approximate<zCDP>, RenyiDP, MultiDP])
        ], (self, d_i))
    }
}
