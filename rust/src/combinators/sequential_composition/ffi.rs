use crate::{
    combinators::Composability,
    error::Fallible,
    ffi::any::{AnyMeasure, AnyObject, Downcast},
    measures::{Approximate, MaxDivergence, MultiDP, RenyiDivergence, ZeroConcentratedDivergence},
};

use super::{Adaptivity, CompositionMeasure};

impl CompositionMeasure for AnyMeasure {
    type Budget = AnyObject;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            measure: &AnyMeasure,
            budget: &AnyObject,
        ) -> Fallible<AnyObject>
        where
            M::Distance: 'static,
        {
            measure
                .downcast_ref::<M>()?
                .budget_guarantee(budget.downcast_ref::<M::Budget>()?)
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence, MultiDP])
        ], (self, budget))
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            measure: &AnyMeasure,
            distance: &AnyObject,
            budget: &AnyObject,
        ) -> Fallible<bool>
        where
            M::Distance: 'static,
        {
            measure.downcast_ref::<M>()?.check_budget(
                distance.downcast_ref::<M::Distance>()?,
                budget.downcast_ref::<M::Budget>()?,
            )
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence, MultiDP])
        ], (self, distance, budget))
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
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence, MultiDP])
        ], (self, adaptivity))
    }
    fn compose(
        &self,
        d_i: Vec<Self::Distance>,
        composability: Composability,
    ) -> Fallible<Self::Distance> {
        fn monomorphize<M: 'static + CompositionMeasure>(
            self_: &AnyMeasure,
            d_i: Vec<AnyObject>,
            composability: Composability,
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
                    composability,
                )
                .map(AnyObject::new)
        }
        dispatch!(monomorphize, [
            (self.type_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>, RenyiDivergence, MultiDP])
        ], (self, d_i, composability))
    }
}
