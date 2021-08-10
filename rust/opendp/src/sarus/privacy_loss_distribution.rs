use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::{FromIterator, IntoIterator};
use std::collections::BTreeMap;
use std::ops::Mul;

use rug::Rational;


/// Privacy Loss Distribution from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

/// A privacy loss value (log-likelihood)
#[derive(Clone, PartialEq)]
pub struct PLDistribution {
    /// We represent PLD as p_y/p_x -> p_x
    /// contrary to http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf (p_x/p_y -> p_x)
    /// so we don't need +infinity in the number representation
    pub exp_privacy_loss_probabilities: BTreeMap<Rational, Rational>
}

impl<'a> PLDistribution {
    pub fn new<I,Q>(exp_privacy_loss_probabilitiies:I) -> PLDistribution
    where I: 'a + IntoIterator<Item=&'a (Q, Q)>, Q: 'a + Clone, Rational: TryFrom<Q> {
        let p_y_x_p_x: Vec<(Rational,Rational)> = exp_privacy_loss_probabilitiies.into_iter().map(|(l,p)| {
            (Rational::try_from(l.clone()).unwrap_or_default(), Rational::try_from(p.clone()).unwrap_or_default())
        }).collect();
        let sum_p_x = p_y_x_p_x.iter().fold(Rational::from(0),
        |s,(_,p)| {s+p});
        let sum_p_y = p_y_x_p_x.iter().fold(Rational::from(0),
        |s,(l,p)| {s+p.clone()*l});
        PLDistribution {exp_privacy_loss_probabilities:
            BTreeMap::from_iter(p_y_x_p_x.into_iter().map(
                |(p_y_x,p_x)| (p_y_x*&sum_p_x/&sum_p_y, p_x/&sum_p_x) ))
        }
    }

    pub fn probabilities(&self) -> &BTreeMap<Rational, Rational> {
        &self.exp_privacy_loss_probabilities
    }

    pub fn probability<Q>(&self, exp_privacy_loss: Q) -> &Rational
    where Rational: TryFrom<Q> {
        &self.exp_privacy_loss_probabilities[&Rational::try_from(exp_privacy_loss).unwrap_or_default()]
    }

    /// Use the formula from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta<Q>(&self, exp_epsilon: Q) -> Rational
    where Q: Clone, Rational: TryFrom<Q> {

        let e_x_y_inv = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default().recip();
        let e_y_x = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default();
        let (delta_x_y, delta_y_x) = self.exp_privacy_loss_probabilities.iter().fold((Rational::from(0),Rational::from(0)), 
        |(delta_x_y, delta_y_x),(l_y_x,p_x)| {
            (
                delta_x_y + if l_y_x<e_x_y_inv {(Rational::from(1)-l_y_x.clone()/e_x_y_inv)*p_x} else {Rational::from(0)},
                delta_y_x + if l_y_x>e_y_x {(Rational::from(1)-e_y_x.clone()/l_y_x)*p_x*l_y_x} else {Rational::from(0)},
            )
        });
        if delta_x_y > delta_y_x {delta_x_y} else {delta_y_x}
    }

    /// Compute the alphas and the betas
    pub fn tradeoff<I>(&self, exp_epsilons:I) -> Vec<(Rational, Rational)>
    where I: 'a + IntoIterator, I::Item:'a + Clone, Rational: TryFrom<I::Item> {
        let mut result: Vec<(Rational, Rational)> = Vec::new();
        let mut last: (Rational, Rational) = (0.into(),0.into());
        for exp_eps in exp_epsilons {
            let exp_epsilon = Rational::try_from(exp_eps).unwrap_or_default();
            let delta = self.delta::<Rational>(exp_epsilon.clone());
            result.push((
                (last.1.clone()-&delta)/(exp_epsilon.clone()-&last.0),
                ((Rational::from(1)-last.1)*&exp_epsilon-(Rational::from(1)-&delta)*&last.0)/(exp_epsilon.clone()-&last.0),
            ));
            last = (exp_epsilon.clone(), delta)
        }
        result
    }
}

/// Compute the cartesian product of output domains
impl Mul for &PLDistribution {
    type Output = PLDistribution;
    fn mul(self, other: &PLDistribution) -> PLDistribution {
        let mut result = PLDistribution {exp_privacy_loss_probabilities:BTreeMap::new()};
        for (s_epl,s_prob) in &self.exp_privacy_loss_probabilities {
            for (o_epl,o_prob) in &self.exp_privacy_loss_probabilities {
                let epl = s_epl.clone() * o_epl;
                result.exp_privacy_loss_probabilities.entry(epl)
                    .and_modify(|prob| { *prob *= s_prob.clone() * o_prob })
                    .or_insert(s_prob.clone() * o_prob);
            }
        }
        result
    }
}