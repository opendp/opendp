use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::IntoIterator;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Mul;

use rug::Rational;

use crate::error::Fallible;

/// Privacy Loss Distribution from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

/// A privacy loss value (log-likelihood)
#[derive(Clone, PartialEq)]
pub struct PLDistribution {
    /// We represent PLD as p_x/p_y -> p_x
    /// similarly to http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    /// ratio 0 and +infinity are treated separately
    pub exp_privacy_loss_probabilities: BTreeMap<Rational, Rational>,
    pub infinite_privacy_loss_probability: Rational,
}

impl<'a> PLDistribution {
    /// Build a new PLDistribution and renormalize probabilities so that they sum exactly to 1
    pub fn new<I>(exp_privacy_loss_probabilitiies:I, infinite_privacy_loss_probability:Rational) -> PLDistribution
    where I: IntoIterator<Item=(Rational, Rational)> {
        let l_x_y_p_x: Vec<(Rational,Rational)> = exp_privacy_loss_probabilitiies.into_iter().collect();
        let sum_p_x = l_x_y_p_x.iter()
            .filter(|(l,p)| l>0)
            .fold(Rational::from(0), |s,(_,p)| {s+p});
        let sum_p_y = l_x_y_p_x.iter()
            .filter(|(l,p)| l>0)
            .fold(Rational::from(0), |s,(l,p)| {s+p.clone()/l});
        let mut l_x_y_p_x_map:BTreeMap<Rational, Rational> = BTreeMap::new();
        for (l_x_y, p_x) in l_x_y_p_x {
            if l_x_y > 0 {
                l_x_y_p_x_map.entry((Rational::from(1)-&infinite_privacy_loss_probability)*&l_x_y*&sum_p_y/&sum_p_x)
                    .and_modify(|p| { *p += (Rational::from(1)-&infinite_privacy_loss_probability)*&p_x/&sum_p_x })
                    .or_insert((Rational::from(1)-&infinite_privacy_loss_probability)*&p_x/&sum_p_x);
            }
        }
        PLDistribution {exp_privacy_loss_probabilities:l_x_y_p_x_map, infinite_privacy_loss_probability}
    }

    /// Use the formula from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta(&self, exp_epsilon: Rational) -> Rational {
        // todo add inf proba
        let (delta_x_y, delta_y_x) = if &exp_epsilon>&Rational::from(0) {
            self.exp_privacy_loss_probabilities.iter().fold((Rational::from(0), Rational::from(0)), 
            |(delta_x_y,delta_y_x),(l_x_y,p_x)| {
                    (if l_x_y>&exp_epsilon {delta_x_y + (Rational::from(1)-exp_epsilon.clone()/l_x_y)*p_x} else {delta_x_y},
                    if l_x_y<&exp_epsilon.clone().recip() {delta_y_x + (Rational::from(1)-l_x_y.clone()*&exp_epsilon)*p_x/l_x_y} else {delta_y_x})
            })
        } else {
            (Rational::from(1), Rational::from(1))
        };
        Rational::max(delta_x_y,delta_y_x)
    }

    /// Compute the alphas and the betas
    pub fn tradeoff(&self) -> Vec<(Rational, Rational)> {
        // todo add inf proba
        let mut result = Vec::new();
        let mut exp_epsilons_set:BTreeSet<Rational> = BTreeSet::new();
        // Initialize the set of possible exp_eps
        for exp_epsilon in self.exp_privacy_loss_probabilities.keys() {
            exp_epsilons_set.insert(exp_epsilon.clone());
            if exp_epsilon>&Rational::from(0) {
                exp_epsilons_set.insert(exp_epsilon.clone().recip());
            }
        }
        let exp_epsilons: Vec<Rational> = exp_epsilons_set.into_iter().rev().collect();
        let mut last_exp_epsilon = exp_epsilons[0].clone();
        let mut last_delta= self.delta(last_exp_epsilon.clone());
        result.push((Rational::from(0), Rational::from(1)-&last_delta));
        for i in 1..exp_epsilons.len() {
            let exp_epsilon = exp_epsilons[i].clone();
            let delta = self.delta(exp_epsilon.clone());
            let denom = exp_epsilon.clone()-&last_exp_epsilon;
            result.push((
                (last_delta.clone()-&delta)/&denom,
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
            ));
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        let exp_epsilon = Rational::from(0);
        let delta = Rational::from(1);
        let denom = exp_epsilon.clone()-&last_exp_epsilon;
        if denom>Rational::from(0) {
            result.push((
                (last_delta.clone()-&delta)/&denom,
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
            ));
        }
        result
    }

    pub fn f(&self) -> Vec<(f64, f64)> {
        self.tradeoff().into_iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect()
    }
}

/// Compute the composition of PLDs
impl Mul for &PLDistribution {
    // todo add inf proba
    type Output = PLDistribution;
    fn mul(self, other: &PLDistribution) -> PLDistribution {
        let mut result = PLDistribution {exp_privacy_loss_probabilities:BTreeMap::new()};
        for (s_epl,s_prob) in &self.exp_privacy_loss_probabilities {
            for (o_epl,o_prob) in &other.exp_privacy_loss_probabilities {
                let epl = s_epl.clone() * o_epl;
                result.exp_privacy_loss_probabilities.entry(epl)
                    .and_modify(|prob| { *prob += s_prob.clone() * o_prob })
                    .or_insert(s_prob.clone() * o_prob);
            }
        }
        result
    }
}

impl Default for PLDistribution {
    fn default() -> Self {
        PLDistribution::new([(Rational::from(1),Rational::from(1))], Rational::from(0))
    }
}

impl<Q> From<Vec<(Q,Q)>> for PLDistribution
where Rational: TryFrom<Q> {
    fn from(exp_privacy_loss_probabilities: Vec<(Q,Q)>) -> PLDistribution {
        let mut rational_exp_privacy_loss_probabilities: Vec<(Rational,Rational)>;
        let mut rational_infinite_privacy_loss_probability = Rational::from(0);
        for (epl, p) in exp_privacy_loss_probabilities {
            let rat_epl = Rational::try_from(epl).unwrap_or_default();
            let rat_p = Rational::try_from(p).unwrap_or_default();
            if rat_epl == 0 {rational_infinite_privacy_loss_probability += rat_p;}
            else {rational_exp_privacy_loss_probabilities.push((rat_epl, rat_p))}
        }
        PLDistribution::new(rational_exp_privacy_loss_probabilities, rational_infinite_privacy_loss_probability)
    }
}