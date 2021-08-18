use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::IntoIterator;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Mul;

use rug::{Integer, Rational};

const GRID_SIZE:usize = 100;
const DENOM:usize = 1000000;

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
        let mut p_y_x_p_x_map:BTreeMap<Rational, Rational> = BTreeMap::new();
        for (p_y_x, p_x) in p_y_x_p_x {
            p_y_x_p_x_map.entry(p_y_x*&sum_p_x/&sum_p_y)
                    .and_modify(|p| { *p += p_x.clone()/&sum_p_x })
                    .or_insert(p_x/&sum_p_x);
        }
        PLDistribution {exp_privacy_loss_probabilities:p_y_x_p_x_map}
    }

    /// Use the formula from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta(&self, exp_epsilon: Rational) -> Rational {
        let zero_infinite_privacy_loss_delta = self.exp_privacy_loss_probabilities.get(&Rational::from(0)).unwrap_or(&Rational::from(0)).clone();
        let (delta_x_y, delta_y_x) = if &exp_epsilon>&Rational::from(0) {
            self.exp_privacy_loss_probabilities.iter()
            .fold(( zero_infinite_privacy_loss_delta.clone(), zero_infinite_privacy_loss_delta), 
            |(delta_x_y,delta_y_x),(l_y_x,p_x)| {
                    (if l_y_x<&exp_epsilon.clone().recip() {delta_x_y + (Rational::from(1)-l_y_x.clone()*exp_epsilon.clone())*p_x} else {delta_x_y},
                    if l_y_x>&exp_epsilon {delta_y_x + (Rational::from(1)-exp_epsilon.clone()/l_y_x)*p_x*l_y_x} else {delta_y_x})
            })
        } else {
            (Rational::from(1), Rational::from(1))
        };
        Rational::max(delta_x_y,delta_y_x)
    }

    /// Compute a delta and simplifies it to the simple fraction just below
    /// The degree of simplification is expressed by giving the target denominator
    pub fn simplified_delta(&self, exp_epsilon: Rational, denom: usize) -> Rational {
        let delta = self.delta(exp_epsilon);
        let num = Integer::from(denom) * delta.numer() / delta.denom();
        Rational::from((num,denom))
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
        
        // Reverse the exp epsilons to have them by decreasing order
        let exp_epsilons: Vec<Rational> = exp_epsilons_set.into_iter().rev().collect();
        // Insert the first points
        result.push((Rational::from(0), Rational::from(1)));
        let mut last_exp_epsilon = exp_epsilons[0].clone();
        let mut last_delta= self.delta(last_exp_epsilon.clone());
        result.push((Rational::from(0), Rational::from(1)-&last_delta));
        for i in 1..exp_epsilons.len() {
            let exp_epsilon = exp_epsilons[i].clone();
            let delta = self.delta(exp_epsilon.clone());
            let denom = exp_epsilon.clone()-&last_exp_epsilon;
            if denom!=Rational::from(0) {
                result.push((
                    (last_delta.clone()-&delta)/&denom,
                    ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
                ));
            }
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        let exp_epsilon = Rational::from(0);
        let delta = Rational::from(1);
        let denom = exp_epsilon.clone()-&last_exp_epsilon;
        if denom!=Rational::from(0) {
            result.push((
                (last_delta.clone()-&delta)/&denom,
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
            ));
        }
        result.push((Rational::from(1), Rational::from(0)));
        result
    }

    pub fn f(&self) -> Vec<(f64, f64)> {
        self.tradeoff().into_iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect()
    }

    /// Compute a vctor of deltas
    pub fn deltas(&self, exp_epsilons:Vec<Rational>) -> Vec<(Rational, Rational)> {
        exp_epsilons.into_iter().map(|e| {(e.clone(),self.delta(e))}).collect()
    }

    /// Probabilities computed from simplified deltas
    pub fn simplified_probabilities(&self, exp_epsilons:Vec<Rational>, denom:usize) -> Vec<(Rational, Rational)> {
        let mut result = Vec::new();
        let mut exp_epsilons_set:BTreeSet<Rational> = BTreeSet::new();
        // Initialize the set of possible exp_eps
        for exp_epsilon in exp_epsilons {exp_epsilons_set.insert(exp_epsilon.clone());}
        // Reverse the exp epsilons to have them by decreasing order
        let exp_epsilons: Vec<Rational> = exp_epsilons_set.into_iter().rev().collect();
        // Insert the first point
        result.push((exp_epsilons[0].clone(), Rational::from(0)));
        let mut last_exp_epsilon = exp_epsilons[0].clone();
        let mut last_delta= self.simplified_delta(last_exp_epsilon.clone(), denom);
        for i in 1..exp_epsilons.len() {
            let exp_epsilon = exp_epsilons[i].clone();
            let delta = self.simplified_delta(exp_epsilon.clone(), denom);
            let denom = exp_epsilon.clone()-&last_exp_epsilon;
            if denom!=Rational::from(0) {result.push((exp_epsilon.clone(), (last_delta.clone()-&delta)/&denom));}
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        let exp_epsilon = Rational::from(0);
        let delta = Rational::from(1);
        let denom = exp_epsilon.clone()-&last_exp_epsilon;
        if denom!=Rational::from(0) {result.push((exp_epsilon.clone(), (last_delta.clone()-&delta)/&denom));}
        result.push((Rational::from(0), Rational::from(1)));
        result.windows(2).map(|window| {(window[0].0.clone(), window[1].1.clone()-window[0].1.clone())}).collect()
    }

    /// TODO
    pub fn simplified(&self) -> PLDistribution {
        let exp_epsilons: Vec<Rational> = self.exp_privacy_loss_probabilities.iter().map(|(l,p)| l.clone()).collect();
        let mut result_exp_epsilons: Vec<Rational> = Vec::new();
        let min_exp_epsilon = exp_epsilons.first().unwrap_or(&Rational::from(0)).clone();
        let max_exp_epsilon = exp_epsilons.last().unwrap_or(&Rational::from(1)).clone();
        let max_result_exp_epsilon = if min_exp_epsilon == 0 {max_exp_epsilon} else {Rational::max(min_exp_epsilon.recip(), max_exp_epsilon)};
        result_exp_epsilons.push(Rational::from(0));
        for i in 0..GRID_SIZE {
            result_exp_epsilons.push((max_result_exp_epsilon.clone()-Rational::from(i)*(max_result_exp_epsilon.clone()-Rational::from(1))/Rational::from(GRID_SIZE)).recip());
        }
        result_exp_epsilons.push(Rational::from(1));
        for i in 0..GRID_SIZE {
            result_exp_epsilons.push(Rational::from(1)+Rational::from(i+1)*(max_result_exp_epsilon.clone()-Rational::from(1))/Rational::from(GRID_SIZE));
        }
        PLDistribution::new(self.simplified_probabilities(result_exp_epsilons, DENOM))
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
                    .and_modify(|prob| { *prob += s_prob.clone() * o_prob })
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