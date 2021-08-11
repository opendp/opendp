use std::{convert::TryFrom, marker::PhantomData};

use rug::Rational;

use crate::{core::Domain, error::Fallible};

use super::PLDistribution;

/// A Domain that comes with the privacy loss distribution.
#[derive(Clone, PartialEq)]
pub struct PLDomain<D, Q> where D: Domain<Carrier=Q>, Q: Clone + PartialEq, Rational: TryFrom<Q> {
    element_domain: D,
    privacy_loss_distribution: PLDistribution,
}

impl<D,Q> PLDomain<D,Q> where D: Domain<Carrier=Q>, Q: Clone + PartialEq, Rational: TryFrom<Q> {
    pub fn new<'a,I>(element_domain:D, exp_privacy_loss_probabilitiies:I) -> PLDomain<D,Q>
    where I: 'a + IntoIterator<Item=&'a (Q, Q)>, Q: 'a
    {
        PLDomain {
            element_domain,
            privacy_loss_distribution: PLDistribution::new(exp_privacy_loss_probabilitiies)
        }
    }
}

impl<D,Q> Domain for PLDomain<D,Q> where D: Domain<Carrier=Q>, Q: Clone + PartialEq, Rational: TryFrom<Q> {
    type Carrier = D::Carrier;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.element_domain.member(val)
    }
}