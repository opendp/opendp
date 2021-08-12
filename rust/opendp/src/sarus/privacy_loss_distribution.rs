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
    pub fn new<I>(exp_privacy_loss_probabilitiies:I) -> PLDistribution
    where I: IntoIterator<Item=(Rational, Rational)> {
        let p_y_x_p_x: Vec<(Rational,Rational)> = exp_privacy_loss_probabilitiies.into_iter().collect();
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
        let exp_epsilon = Rational::try_from(exp_epsilon).unwrap_or_default();
        let (delta_x_y, delta_y_x) = if &exp_epsilon>&Rational::from(0) {
            self.exp_privacy_loss_probabilities.iter().fold((Rational::from(0), Rational::from(0)), 
            |(delta_x_y,delta_y_x),(l_y_x,p_x)| {
                    (if l_y_x<&exp_epsilon.clone().recip() {delta_x_y + (Rational::from(1)-l_y_x.clone()*exp_epsilon.clone())*p_x} else {delta_x_y},
                    if l_y_x>&exp_epsilon {delta_y_x + (Rational::from(1)-exp_epsilon.clone()/l_y_x)*p_x*l_y_x} else {delta_y_x})
            })
        } else {
            (Rational::from(1), Rational::from(1))
        };
        println!("exp_epsilon: {:?}, delta_x_y: {:?} - delta_y_x: {:?}", exp_epsilon.to_f64(), delta_x_y.to_f64(), delta_y_x.to_f64());
        Rational::max(delta_x_y,delta_y_x)
    }

    /// Compute the alphas and the betas
    pub fn tradeoff(&self) -> Vec<(Rational, Rational)> {
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
        result.push((Rational::from(0),Rational::from(1)-&last_delta));
        result
    }

    pub fn f(&self) -> Vec<(f64, f64)> {
        self.tradeoff().into_iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect()
    }
}

/// Compute the composition of PLDs
impl Mul for &PLDistribution {
    type Output = PLDistribution;
    fn mul(self, other: &PLDistribution) -> PLDistribution {
        let mut result = PLDistribution {exp_privacy_loss_probabilities:BTreeMap::new()};
        for (s_epl,s_prob) in &self.exp_privacy_loss_probabilities {
            for (o_epl,o_prob) in &other.exp_privacy_loss_probabilities {
                let epl = s_epl.clone() * o_epl;
                result.exp_privacy_loss_probabilities.entry(epl)
                    .and_modify(|prob| { *prob *= s_prob.clone() * o_prob })
                    .or_insert(s_prob.clone() * o_prob);
            }
        }
        result
    }
}

impl Default for PLDistribution {
    fn default() -> Self {
        PLDistribution::new([(Rational::from(1),Rational::from(1))])
    }
}

impl<Q> From<Vec<(Q,Q)>> for PLDistribution
where Rational: TryFrom<Q> {
    fn from(exp_privacy_loss_probabilities: Vec<(Q,Q)>) -> PLDistribution {
        let rational_exp_privacy_loss_probabilities: Vec<(Rational,Rational)> = exp_privacy_loss_probabilities.into_iter().map(|(epl, p)| 
            (Rational::try_from(epl).unwrap_or_default(), Rational::try_from(p).unwrap_or_default())
        ).collect();
        PLDistribution::new(rational_exp_privacy_loss_probabilities)
    }
}