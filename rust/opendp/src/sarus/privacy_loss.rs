use std::clone;
use std::convert::{TryFrom, TryInto};
use std::iter::{FromIterator, IntoIterator};
use std::collections::BTreeMap;

use crate::sarus::positive_rational::PositiveRational;
use crate::error::Fallible;
use crate::dom::AllDomain;
use crate::core::{
    Domain,
    Metric,
    Measure,
    Measurement,
};
use rug::rand::RandGen;
use rug::{Float, Rational};

/// Privacy Loss Measurement (PLM) inspired from PLD http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

pub type PLMInputDomain = AllDomain<bool>;

/// A privacy loss value (log-likelihood)


#[derive(Clone, PartialEq)]
pub struct PLMOutputDomain {
    /// We represent PLD as p_y/p_x -> p_x
    /// contrary to http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf (p_x/p_y -> p_x)
    /// so we don't need +infinity in the number representation
    pub exp_privacy_loss_probabilitiies: BTreeMap<Rational, Rational>
}

impl PLMOutputDomain {
    pub fn new<L,P>(exp_privacy_loss_probabilitiies:&[(L, P)]) -> PLMOutputDomain
    where L: Clone, P: Clone,
    Rational: TryFrom<L>+TryFrom<P> {
        let p_y_x_p_x = exp_privacy_loss_probabilitiies.iter().map(|(l,p)| {
            (Rational::try_from(l.clone()).unwrap_or_default(), Rational::try_from(p.clone()).unwrap_or_default())
        }).collect::<Vec<(Rational, Rational)>>();
        let sum_p_x = p_y_x_p_x.iter().fold(Rational::from(0),
        |s,(_,p)| {s+p});
        let sum_p_y = p_y_x_p_x.iter().fold(Rational::from(0),
        |s,(l,p)| {s+p.clone()*l});
        PLMOutputDomain {exp_privacy_loss_probabilitiies:
            BTreeMap::from_iter(p_y_x_p_x.into_iter().map(
                |(p_y_x,p_x)| (p_y_x*&sum_p_x/&sum_p_y, p_x/&sum_p_x) ))
        }
    }

    /// This is not a correct version of the simplify function
    pub fn simplify(self, precision:u64) -> PLMOutputDomain {
        PLMOutputDomain::new(&self.exp_privacy_loss_probabilitiies.into_iter().map(|(l,p)| {
            (Rational::round(l*precision)/precision,Rational::round(p*precision)/precision)
        }).collect::<Vec<(Rational, Rational)>>())
    }

    /// Use the formula from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta<Q>(&self, exp_epsilon:Q) -> Rational
    where Q: Clone, Rational: TryFrom<Q> {
         
        let e_x_y_inv = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default().recip();
        let e_y_x = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default();
        let (delta_x_y, delta_y_x) = self.exp_privacy_loss_probabilitiies.iter().fold((Rational::from(0),Rational::from(0)), 
        |(delta_x_y, delta_y_x),(l_y_x,p_x)| {
            (
                delta_x_y + if l_y_x<=e_x_y_inv {(Rational::from(1)-l_y_x.clone()/e_x_y_inv)*p_x} else {Rational::from(0)},
                delta_y_x + if l_y_x>=e_y_x {(Rational::from(1)-e_y_x.clone()/l_y_x)*p_x*l_y_x} else {Rational::from(0)},
            )
        });
        println!("{:?} <-> {:?}", delta_x_y, delta_y_x);
        if delta_x_y > delta_y_x {delta_x_y} else {delta_y_x}
    }
}

impl Domain for PLMOutputDomain {
    type Carrier = Rational;
    fn member(&self, privacy_loss: &Self::Carrier) -> Fallible<bool> { Ok(self.exp_privacy_loss_probabilitiies.contains_key(privacy_loss)) }
}