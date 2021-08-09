use std::clone::Clone;
use std::convert::{TryFrom, TryInto};
use std::iter::{FromIterator, IntoIterator};
use std::collections::BTreeMap;

use crate::error::Fallible;
use crate::dom::AllDomain;
use crate::dist::{IntDistance, SymmetricDistance, SmoothedMaxDivergence};
use crate::core::{Domain, Function, Measurement, PrivacyRelation};
use rug::ops::DivRounding;
use rug::{Integer, Float, Rational, float::Round};

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
    pub fn new<Q>(exp_privacy_loss_probabilitiies:&[(Q, Q)]) -> PLMOutputDomain
    where Q: Clone, Rational: TryFrom<Q> {
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
                delta_x_y + if l_y_x<e_x_y_inv {(Rational::from(1)-l_y_x.clone()/e_x_y_inv)*p_x} else {Rational::from(0)},
                delta_y_x + if l_y_x>e_y_x {(Rational::from(1)-e_y_x.clone()/l_y_x)*p_x*l_y_x} else {Rational::from(0)},
            )
        });
        if delta_x_y > delta_y_x {delta_x_y} else {delta_y_x}
    }
}

impl Domain for PLMOutputDomain {
    type Carrier = Rational;
    fn member(&self, privacy_loss: &Self::Carrier) -> Fallible<bool> { Ok(self.exp_privacy_loss_probabilitiies.contains_key(privacy_loss)) }
}

pub fn make_plm<Q>(exp_privacy_loss_probabilitiies:&[(Q, Q)]) -> Fallible<Measurement<PLMInputDomain, PLMOutputDomain, SymmetricDistance, SmoothedMaxDivergence<Rational>>>
    where Q: Clone, Rational: From<Q> {
    let out_dom = PLMOutputDomain::new(exp_privacy_loss_probabilitiies);
    let priv_rel = make_plm_privacy_relation(out_dom.clone());
    Ok(Measurement::new(
        PLMInputDomain::new(),
        out_dom,
        Function::new_fallible(|&_| fallible!(NotImplemented)),
        SymmetricDistance::default(),
        SmoothedMaxDivergence::default(),
        priv_rel,
    ))
}

fn make_plm_privacy_relation(out_dom: PLMOutputDomain) -> PrivacyRelation<SymmetricDistance, SmoothedMaxDivergence<Rational>> {
    PrivacyRelation::new_fallible( move |d_in: &IntDistance, (epsilon, delta): &(Rational, Rational)| {
        if d_in<&0 {
            return fallible!(InvalidDistance, "Privacy Loss Mechanism: input sensitivity must be non-negative")
        }
        if delta<=&0 {
            return fallible!(InvalidDistance, "Privacy Loss Mechanism: delta must be positive")
        }
        let mut exp_epsilon = Float::with_val_round(64, epsilon, Round::Down).0;
        exp_epsilon.exp_round(Round::Down);
        Ok(delta >= &(out_dom.delta(exp_epsilon)))
    })
}