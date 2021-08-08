use std::iter::{FromIterator, IntoIterator};
use std::collections::BTreeMap;
use rug::Rational;
use crate::error::Fallible;
use crate::domains::AtomDomain;
use crate::core::{
    Domain,
    Metric,
    Measure,
    Measurement,
};

/// Privacy Loss Measurement (PLM) inspired from PLD http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

type PLMInputDomain = AtomDomain<bool>;

#[derive(Clone, PartialEq)]
pub struct PLMOutputDomain {
    privacy_loss_probabilitiies: BTreeMap<Rational, Rational>
}

impl PLMOutputDomain {
    pub fn new<Q: Into<Rational>>(privacy_loss_probabilitiies:Vec<(Q, Q)>) -> PLMOutputDomain {
        PLMOutputDomain {privacy_loss_probabilitiies:
            BTreeMap::from_iter(privacy_loss_probabilitiies.into_iter().map(
                |(q,r)| (q.into(), r.into())))
        }
    }
}

impl Domain for PLMOutputDomain {
    type Carrier = Rational;
    fn member(&self, privacy_loss: &Self::Carrier) -> Fallible<bool> { Ok(self.privacy_loss_probabilitiies.contains_key(privacy_loss)) }
}