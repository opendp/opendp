use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::{FromIterator, IntoIterator};
use std::collections::{BTreeMap, BTreeSet};
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
        let e_y_x = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default();
        let delta_x_y:Rational;
        let delta_y_x = self.exp_privacy_loss_probabilities.iter().fold(Rational::from(0), 
            |delta_y_x,(l_y_x,p_x)| {
                    delta_y_x + if l_y_x>e_y_x {(Rational::from(1)-e_y_x.clone()/l_y_x)*p_x*l_y_x} else {Rational::from(0)}
            });
        if e_y_x>&Rational::from(0) {
            let e_x_y_inv = &Rational::try_from(exp_epsilon.clone()).unwrap_or_default().recip();
            delta_x_y = self.exp_privacy_loss_probabilities.iter().fold(Rational::from(0), 
            |delta_x_y,(l_y_x,p_x)| {
                    delta_x_y + if l_y_x<e_x_y_inv {(Rational::from(1)-l_y_x.clone()/e_x_y_inv)*p_x} else {Rational::from(0)}
            });
        } else {
            delta_x_y = Rational::from(1)
        }
        if delta_x_y > delta_y_x {delta_x_y} else {delta_y_x}
    }

    /// Compute the alphas and the betas
    pub fn tradeoff(&self, n:usize) -> Vec<(Rational, Rational)> {
        let mut result = Vec::new();
        let mut exp_epsilons_set:BTreeSet<Rational> = BTreeSet::new();
        // Initialize the set of possible exp_eps
        for exp_epsilon in self.exp_privacy_loss_probabilities.keys() {
            exp_epsilons_set.insert(exp_epsilon.clone());
            if exp_epsilon>&Rational::from(0) {
                exp_epsilons_set.insert(exp_epsilon.clone().recip());
            }
        }
        let exp_epsilons: Vec<Rational> = exp_epsilons_set.into_iter().collect();
        let mut last_exp_epsilon = exp_epsilons[0].clone();
        let mut last_delta = self.delta(last_exp_epsilon.clone());
        for i in 1..exp_epsilons.len() {
            let exp_epsilon = exp_epsilons[i].clone();
            let delta = self.delta(exp_epsilon.clone());
            result.push((
                (last_delta.clone()-&delta)/(exp_epsilon.clone()-&last_exp_epsilon),
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/(exp_epsilon.clone()-&last_exp_epsilon),
            ));
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        result.push((
            Rational::from(0),
            Rational::from(1)-&last_delta,
        ));
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