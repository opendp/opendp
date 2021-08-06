use std::ops::Fn;
use std::marker::PhantomData;
use num::Float;
use crate::error::Fallible;
use crate::core::{
    Domain,
    Metric,
    Measure,
    Measurement,
};

pub struct BCRelation<MI: Metric, MO: Measure> {
    pub legendre: String,
}

impl<MI: Metric, MO: Measure> Fn(&MI::Distance, &MO::Distance) -> bool for BCRelation<MI, MO> {

}

/// A bounded complexity measurement
pub fn bc_measurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
    measurement: Measurement<DI, DO, MI, MO>
) -> Measurement<DI, DO, MI, MO> {
    return measurement;
}

#[derive(Clone)]
pub struct BCMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for BCMaxDivergence<Q> {
    fn default() -> Self { BCMaxDivergence(PhantomData) }
}

impl<Q> PartialEq for BCMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<Q: Clone> Measure for BCMaxDivergence<Q> {
    type Distance = (Q, Q);
}