use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    rc::Rc,
};

use crate::error::Fallible;

use super::{Measure, type_name};

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for MaxDivergence<Q> {
    fn default() -> Self {
        MaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for MaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<Q> Debug for MaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MaxDivergence({})", type_name!(Q))
    }
}

impl<Q: Clone> Measure for MaxDivergence<Q> {
    type Distance = Q;
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for SmoothedMaxDivergence<Q> {
    fn default() -> Self {
        SmoothedMaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for SmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<Q> Debug for SmoothedMaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SmoothedMaxDivergence({})", type_name!(Q))
    }
}

impl<Q: Clone> Measure for SmoothedMaxDivergence<Q> {
    type Distance = SMDCurve<Q>;
}

// a curve mapping from delta to epsilon
pub struct SMDCurve<Q>(Rc<dyn Fn(&Q) -> Fallible<Q>>);

impl<Q> Clone for SMDCurve<Q> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Q> SMDCurve<Q> {
    pub fn new(epsilon: impl Fn(&Q) -> Fallible<Q> + 'static) -> Self {
        SMDCurve(Rc::new(epsilon))
    }

    // these functions allow direct invocation as a method, making parens unnecessary
    pub fn epsilon(&self, delta: &Q) -> Fallible<Q> {
        (self.0)(delta)
    }
}

#[derive(Clone)]
pub struct FixedSmoothedMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for FixedSmoothedMaxDivergence<Q> {
    fn default() -> Self {
        FixedSmoothedMaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for FixedSmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Q> Debug for FixedSmoothedMaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FixedSmoothedMaxDivergence({})", type_name!(Q))
    }
}

impl<Q: Clone> Measure for FixedSmoothedMaxDivergence<Q> {
    type Distance = (Q, Q);
}


#[derive(Clone)]
pub struct ZeroConcentratedDivergence<Q>(PhantomData<Q>);
impl<Q> Default for ZeroConcentratedDivergence<Q> {
    fn default() -> Self {
        ZeroConcentratedDivergence(PhantomData)
    }
}

impl<Q> PartialEq for ZeroConcentratedDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Q> Debug for ZeroConcentratedDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ZeroConcentratedDivergence({})", type_name!(Q))
    }
}

impl<Q: Clone> Measure for ZeroConcentratedDivergence<Q> {
    type Distance = Q;
}
