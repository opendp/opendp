use std::clone;
use std::iter::{FromIterator, IntoIterator};
use std::collections::BTreeMap;

use crate::sarus::positive_rational::PositiveRational;
use crate::error::Fallible;
use crate::domains::AtomDomain;
use crate::core::{
    Domain,
    Metric,
    Measure,
    Measurement,
};

/// Privacy Loss Measurement (PLM) inspired from PLD http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
pub type PLMInputDomain = AtomDomain<bool>;

/// A privacy loss value (log-likelihood)


#[derive(Clone, PartialEq)]
pub struct PLMOutputDomain {
    // p_x/p_y -> p_x
    pub exp_privacy_loss_probabilitiies: BTreeMap<PositiveRational, PositiveRational>
}

impl PLMOutputDomain {
    pub fn new<L,P>(exp_privacy_loss_probabilitiies:&[(L, P)]) -> PLMOutputDomain
    where L: Clone + Into<PositiveRational>,
    P: Clone + Into<PositiveRational>, {
        let sum_p_x = exp_privacy_loss_probabilitiies.iter().fold(PositiveRational::from(0),
        |s,(_,p)| {s+p.clone().into()});
        let sum_p_y = exp_privacy_loss_probabilitiies.iter().fold(PositiveRational::from(0),
        |s,(l,p)| {s+(p.clone().into())/(l.clone().into())});
        PLMOutputDomain {exp_privacy_loss_probabilitiies:
            BTreeMap::from_iter(exp_privacy_loss_probabilitiies.iter().map(
                |(l,p)| (l.clone().into()*sum_p_y.clone()/sum_p_x.clone(), p.clone().into()/sum_p_x.clone()) ))
        }
    }

    /// http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta(&self, exp_epsilon:PositiveRational) -> PositiveRational {
        let (delta_x_y, delta_y_x) = self.exp_privacy_loss_probabilitiies.iter().fold((PositiveRational::from(0),PositiveRational::from(0)), 
        |(delta_x_y, delta_y_x),(l_x_y,p_x)| {
            (
                delta_x_y + if l_x_y>=&exp_epsilon {(PositiveRational::from(1)-exp_epsilon.clone()/l_x_y.clone())*p_x.clone()} else {PositiveRational::from(0)},
                delta_y_x + if l_x_y<=&exp_epsilon {(PositiveRational::from(1)-l_x_y.clone()/exp_epsilon.clone())*p_x.clone()/l_x_y.clone()} else {PositiveRational::from(0)},
            )
        });
        println!("{:#?}, {:#?}", delta_x_y, delta_y_x);
        if delta_x_y > delta_y_x {delta_x_y} else {delta_y_x}
    }
}

impl Domain for PLMOutputDomain {
    type Carrier = PositiveRational;
    fn member(&self, privacy_loss: &Self::Carrier) -> Fallible<bool> { Ok(self.exp_privacy_loss_probabilitiies.contains_key(privacy_loss)) }
}