use core::fmt::Debug;
use std::marker::PhantomData;
use num::{One, Zero, Float};

use crate::error::Fallible;
use crate::traits::{Tolerance, Midpoint};
use crate::samplers::CastInternalReal;
use crate::core::{Measure, Metric, PrivacyRelation};


#[derive(Debug)]
pub struct EpsilonDelta<T: Sized>{pub epsilon: T, pub delta: T}

// Derive annotations force traits to be present on the generic
impl<T: Clone> Clone for EpsilonDelta<T> {
    fn clone(&self) -> Self {
        EpsilonDelta {epsilon: self.epsilon.clone(), delta: self.delta.clone()}
    }
}
impl<T: PartialEq> PartialEq for EpsilonDelta<T> {
    fn eq(&self, other: &Self) -> bool {
        self.epsilon == other.epsilon && self.delta == other.delta
    }
}

#[derive(Clone)]
pub struct FSmoothedMaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for FSmoothedMaxDivergence<Q> {
    fn default() -> Self { FSmoothedMaxDivergence(PhantomData) }
}

impl<Q> PartialEq for FSmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<Q: Clone> Measure for FSmoothedMaxDivergence<Q> {
    type Distance = Vec<EpsilonDelta<Q>>;
}

const MAX_ITERATIONS: usize = 100;

impl<MI, Q> PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>
     where MI: Metric,
           Q: Clone + Zero + One + Float + Midpoint + Tolerance + CastInternalReal,
           MI::Distance: Clone {

    pub fn find_epsilon (&self, d_in: &MI::Distance, delta: Q) -> Fallible<Q> {
        let mut eps_min = Q::zero();
        let mut eps = Q::one();

        for _ in 0..MAX_ITERATIONS {
            let dout = vec![EpsilonDelta {
                epsilon: eps.clone(),
                delta: delta.clone(),
            }];
            let eval = match self.eval(&d_in, &dout) {
                Ok(result) => result,
                Err(_) => {return Ok(Q::one() / Q::zero())}
            };

            if !eval {
                eps = eps.clone() * (Q::one() + Q::one());
            }

            else {
                let eps_mid = eps_min.clone().midpoint(eps);
                let dout = vec![EpsilonDelta {
                    epsilon: eps_mid.clone(),
                    delta: delta.clone(),
                }];
                if self.eval(&d_in, &dout)? {
                    eps = eps_mid.clone();
                } else {
                    eps_min = eps_mid.clone();
                }
                if eps <= Q::TOLERANCE.clone() + eps_min.clone() {
                    return Ok(eps)
                }
            }
        }
        let dout = vec![EpsilonDelta {
            epsilon: eps.clone(),
            delta: delta.clone(),
        }];
        if !self.eval(&d_in, &dout)? {
            return Ok(Q::one() / Q::zero())
        }
        Ok(eps)
    }

    pub fn find_delta (&self, d_in: &MI::Distance, epsilon: Q) -> Fallible<Q> {
        let mut delta_min = Q::zero();
        let mut delta = Q::one();

        for _ in 0..MAX_ITERATIONS {
            let dout = vec![EpsilonDelta {
                epsilon: epsilon.clone(),
                delta: delta.clone(),
            }];
            let eval = match self.eval(&d_in, &dout) {
                Ok(result) => result,
                Err(_) => {return Ok(Q::one())}
            };
            if !eval {
                delta = delta.clone() * (Q::one() + Q::one());
            }

            else {
                let delta_mid = delta_min.midpoint(delta);
                let dout = vec![EpsilonDelta {
                    epsilon: epsilon.clone(),
                    delta: delta_mid.clone(),
                }];
                if self.eval(&d_in, &dout)? {
                    delta = delta_mid.clone();
                } else {
                    delta_min = delta_mid.clone();
                }
                if delta <= Q::TOLERANCE + delta_min.clone() {
                    return Ok(delta)
                }
            }
        }
        let dout = vec![EpsilonDelta {
            epsilon: epsilon.clone(),
            delta: delta.clone(),
        }];
        if !self.eval(&d_in, &dout)? {
            return Ok(Q::one())
        }
        Ok(delta)
    }

    pub fn find_epsilon_delta_family (
        &self,
        d_in: &MI::Distance,
        npoints: u8,
        delta_min: Q
    ) -> Vec<EpsilonDelta<Q>> {
        let max_epsilon = self.find_epsilon(&d_in, delta_min).unwrap();
        let mut min_epsilon = self.find_epsilon(&d_in, Q::one()).unwrap();
        if min_epsilon < Q::zero() {
            min_epsilon = Q::zero();
        }

        if max_epsilon == (Q::one() / Q::zero()) {
            return vec![EpsilonDelta{
                epsilon: Q::one() / Q::zero(),
                delta: Q::one(),
            }]
        }

        let step = (max_epsilon.clone() - min_epsilon.clone())
            / Q::from_internal(rug::Float::with_val(53, npoints - 1));
        (0..npoints)
            .map(|i| min_epsilon.clone() + step.clone() * Q::from_internal(rug::Float::with_val(53, i)))
            .map(|eps| EpsilonDelta{
                epsilon: eps.clone(),
                delta: self.find_delta(&d_in, eps.clone()).unwrap()
            })
            .rev()
            .collect()

        // let step = (max_epsilon.clone().exp() - min_epsilon.clone().exp()) / Q::from_internal(rug::Float::with_val(53, npoints - 1));
        // (0..npoints)
        //     .map(|i| min_epsilon.clone().exp() + step.clone() * Q::from_internal(rug::Float::with_val(53, i)))
        //     .map(|eps| EpsilonDelta{
        //         epsilon: eps.clone(),
        //         delta: self.find_delta(&d_in, eps.clone().ln()).unwrap()
        //     })
        //     .rev()
        //     .collect()
    }
}

// ExpEpsilonDelta
#[derive(Debug)]
pub struct ExpEpsilonDelta{pub exp_epsilon: rug::Rational, pub delta: rug::Rational}

// Derive annotations force traits to be present on the generic
impl Clone for ExpEpsilonDelta {
    fn clone(&self) -> Self {
        ExpEpsilonDelta {exp_epsilon: self.exp_epsilon.clone(), delta: self.delta.clone()}
    }
}
impl PartialEq for ExpEpsilonDelta {
    fn eq(&self, other: &Self) -> bool {
        self.exp_epsilon == other.exp_epsilon && self.delta == other.delta
    }
}

impl ExpEpsilonDelta {
    pub fn new (exp_epsilon: rug::Rational, delta: rug::Rational) -> Self {
        ExpEpsilonDelta {
            exp_epsilon: exp_epsilon,
            delta: delta,
        }
    }

    pub fn from_epsilon_delta <Q> (epsilon_delta: EpsilonDelta<Q>) -> Self
    where Q: CastInternalReal {
        let mut exp_epsilon = epsilon_delta.epsilon.into_internal();
        exp_epsilon.exp_round(rug::float::Round::Up);
        Self::new(
            exp_epsilon.to_rational().unwrap(),
            epsilon_delta.delta.into_internal().to_rational().unwrap()
        )
    }

    pub fn to_epsilon_delta <Q> (&self) -> EpsilonDelta<Q>
    where Q: CastInternalReal {
        let mut epsilon = rug::Float::with_val(53, self.exp_epsilon.to_f64());
        epsilon.ln_round(rug::float::Round::Up);
        EpsilonDelta{
            epsilon: Q::from_internal(epsilon),
            delta: Q::from_internal(rug::Float::with_val(53, self.delta.clone()))
        }
    }
}

// Tradeoff function
#[derive(Debug)]
pub struct AlphasBetas {
    pub alphas: Vec<rug::Rational>,
    pub betas: Vec<rug::Rational>,
}

impl Clone for AlphasBetas {
    fn clone(&self) -> Self {
        AlphasBetas {alphas: self.alphas.clone(), betas: self.betas.clone()}
    }
}

impl AlphasBetas {
    pub fn new (alphas: Vec<rug::Rational>, betas: Vec<rug::Rational>) -> Self {
        AlphasBetas {
            alphas: alphas,
            betas: betas,
        }
    }

    fn sort (&mut self) -> () {
        self.alphas.sort();
        self.betas.sort();
        self.betas.reverse();
    }

    pub fn from_vec_exp_epsilon_delta (exp_epsilons_deltas: Vec<ExpEpsilonDelta>) -> Self {
        let one = rug::Rational::from(1);
        let zero = rug::Rational::from(0);

        let mut vec_exp_epsilon_delta = exp_epsilons_deltas.clone();
        vec_exp_epsilon_delta.sort_by(|a, b| a.delta.partial_cmp(&b.delta).unwrap());
        vec_exp_epsilon_delta.dedup();

        let mut alphas = vec![zero.clone(), one.clone() - vec_exp_epsilon_delta[0].delta.clone()];
        let mut betas = vec![one.clone() - vec_exp_epsilon_delta[0].delta.clone(), zero.clone()];

        let size = vec_exp_epsilon_delta.iter().len();
        for i in 1..size {
            let alpha =
                (vec_exp_epsilon_delta[i-1].delta.clone() - vec_exp_epsilon_delta[i].delta.clone())
                /
                (vec_exp_epsilon_delta[i].exp_epsilon.clone() - vec_exp_epsilon_delta[i-1].exp_epsilon.clone());

            let beta = (
                    vec_exp_epsilon_delta[i].exp_epsilon.clone() *(one.clone() - vec_exp_epsilon_delta[i-1].delta.clone())
                    -
                    vec_exp_epsilon_delta[i-1].exp_epsilon.clone() *(one.clone() - vec_exp_epsilon_delta[i].delta.clone())
                )
                /
                (vec_exp_epsilon_delta[i].exp_epsilon.clone() - vec_exp_epsilon_delta[i-1].exp_epsilon.clone());

            if !alphas.clone().iter().any(|i| i==&alpha) {
                alphas.push(alpha.clone());
                betas.push(beta.clone());
                alphas.push(beta);
                betas.push(alpha);
            }
        }

        let mut alphas_betas = Self::new(alphas, betas);
        alphas_betas.sort();
        alphas_betas
    }

    pub fn to_vec_exp_epsilon_delta (&self) -> Vec<ExpEpsilonDelta> {
        let one = rug::Rational::from(1);
        let mut alphas_betas = self.clone();
        alphas_betas.sort();

        let mut vec_exp_epsilon_delta: Vec<ExpEpsilonDelta> = Vec::new();
        let size = alphas_betas.alphas.iter().len() / 2;
        for i in 0..size {
            let exp_epsilon = (alphas_betas.betas[i+1].clone() - alphas_betas.betas[i].clone())
                / (alphas_betas.alphas[i].clone() - alphas_betas.alphas[i+1].clone());
            let delta = one.clone() - alphas_betas.alphas[i+1].clone() * exp_epsilon.clone() - alphas_betas.betas[i+1].clone();
            vec_exp_epsilon_delta.push(ExpEpsilonDelta::new(exp_epsilon, delta))
        }
        vec_exp_epsilon_delta
    }

    pub fn from_privacy_relation <MI, Q> (
        predicate: &PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>,
        npoints: u8,
        delta_min: Q
    ) -> Self
        where MI: Metric,
              Q: 'static + One + Zero + PartialOrd + CastInternalReal + Clone + Debug + Float + Midpoint + Tolerance,
              MI::Distance: Clone + One {
        let epsilons_deltas = predicate.find_epsilon_delta_family(&MI::Distance::one(), npoints, delta_min);
        let exp_epsilons_deltas: Vec<ExpEpsilonDelta> = epsilons_deltas.iter()
            .map(|ed| ExpEpsilonDelta::from_epsilon_delta(ed.clone()))
            .collect();
        Self::from_vec_exp_epsilon_delta(exp_epsilons_deltas)
        }

    pub fn to_probabilities_ratios (&self) -> ProbabilitiesRatios {
        let mut alphas_betas = self.clone();
        alphas_betas.sort();
        let size = alphas_betas.alphas.iter().len();

        let probas: Vec<rug::Rational> = alphas_betas.clone().alphas[0..size-1].iter()
            .zip(alphas_betas.clone().alphas[1..size].iter())
            .map(|(a1, a2)| a2.clone() - a1.clone())
            .collect();

        let ratios: Vec<rug::Rational> = probas.iter()
            .zip(probas.iter().rev())
            .map(|(p,q)| p.clone() / q)
            .collect();

        let probas_ratios = ProbabilitiesRatios::new(probas, ratios);
        //probas_ratios.normalize();
        probas_ratios
    }

    pub fn to_delta(&self, exp_epsilon: rug::Rational) -> rug::Rational {
        // sup(1 - alphas * exp(epsilon) - betas)
        self.alphas.clone().iter()
            .zip(self.betas.clone().iter())
            .map(|(a,b)| rug::Rational::from(1) - a.clone() * exp_epsilon.clone() - b.clone())
            .max()
            .unwrap()
    }
}

#[derive(Debug)]
pub struct ProbabilitiesRatios {
    pub probas: Vec<rug::Rational>,
    pub ratios: Vec<rug::Rational>,
}

impl Clone for ProbabilitiesRatios {
    fn clone(&self) -> Self {
        ProbabilitiesRatios {probas: self.probas.clone(), ratios: self.ratios.clone()}
    }
}

impl ProbabilitiesRatios {
    pub fn new (probas: Vec<rug::Rational>, ratios: Vec<rug::Rational>) -> Self {
        ProbabilitiesRatios {
            probas: probas,
            ratios: ratios,
        }
    }

    pub fn len (&self) -> usize {
        self.probas.clone().iter().len()
    }

    pub fn normalize(&mut self) -> () {
        self.sort();

        // Add up probas with the same ratio
        let zero = rug::Rational::from(0);
        let mut probas: Vec<rug::Rational> = Vec::new();
        let mut ratios = self.clone().ratios;
        ratios.dedup();
        for ratio in &ratios {
            let proba = rug::Rational::from(
                rug::Rational::sum(
                    self.probas.iter()
                        .zip(self.ratios.iter())
                        .map(|(p,r)| {if r == &ratio.clone() {p.clone()} else {zero.clone()}})
                        .collect::<Vec<rug::Rational>>()
                        .iter()
                    )
            );
            // let mut proba = rug::Rational::from(0);
            // for (p, r) in self.probas.iter().zip(self.ratios.iter()) {
            //     if r == &ratio.clone() {
            //         proba += p.clone();
            //     }
            // }
            probas.push(proba);
        }

        // Normalize probas so that sum(probas) = 1
        let sum_probas = rug::Rational::from(rug::Rational::sum(probas.clone().iter()));
        probas = probas.iter()
            .map(|x| x.clone() / sum_probas.clone()) // rug::Rational::from(x / sum_probas.clone()))
            .skip_while(|x| x == &rug::Rational::from(0))
            .collect();
        let ratios: Vec<rug::Rational> = probas.iter()
            .zip(probas.clone().iter().rev())
            .map(|(p, q)| p.clone() / q) //rug::Rational::from(p / q))
            .collect();

        self.probas = probas;
        self.ratios = ratios;
    }

    pub fn sort (&mut self) -> () {
        let mut proba_ratio_vec: Vec<(rug::Rational, rug::Rational)> = self.probas.iter()
            .zip(self.ratios.iter())
            .map(|(p,r)| (p.clone(), r.clone()))
            .collect();
        proba_ratio_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let probas: Vec<rug::Rational> = proba_ratio_vec.iter()
            .map(|(p, _r)| p.clone())
            .collect();
        let ratios: Vec<rug::Rational> = proba_ratio_vec.iter()
            .map(|(_p, r)| r.clone())
            .collect();
        self.probas = probas;
        self.ratios = ratios;
    }

    pub fn from_vec_exp_epsilon_delta (exp_epsilons_deltas: Vec<ExpEpsilonDelta>) -> Self {
        let alphas_betas = AlphasBetas::from_vec_exp_epsilon_delta(exp_epsilons_deltas);
        alphas_betas.to_probabilities_ratios()
    }

    pub fn from_privacy_relation <MI, Q> (
        predicate: &PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>,
        npoints: u8,
        delta_min: Q
    ) -> Self
        where MI: Metric,
              Q: 'static + One + Zero + PartialOrd + CastInternalReal + Clone + Debug + Float + Midpoint + Tolerance,
              MI::Distance: Clone + One {
        let epsilons_deltas = predicate.find_epsilon_delta_family(&MI::Distance::one(), npoints, delta_min);
        let exp_epsilons_deltas: Vec<ExpEpsilonDelta> = epsilons_deltas.iter()
            .map(|ed| ExpEpsilonDelta::from_epsilon_delta(ed.clone()))
            .collect();
        Self::from_vec_exp_epsilon_delta(exp_epsilons_deltas)
    }

    pub fn compose (&self, other: &Self) -> Self {
        let mut probas: Vec<rug::Rational> = Vec::new();
        let mut ratios: Vec<rug::Rational> = Vec::new();

        let size1 = self.probas.iter().len();
        let size2 = other.probas.iter().len();
        for i in 0..size1 {
            for j in 0..size2 {
                probas.push(rug::Rational::from(self.probas[i].clone() * other.probas[j].clone()));
                ratios.push(rug::Rational::from(self.ratios[i].clone() * other.ratios[j].clone()));
            }
        }
        Self::new(probas, ratios)
    }

    pub fn to_alphas_betas (&self) -> AlphasBetas {
        let proba_ratios = self.clone();
        //proba_ratios.normalize();

        let zero = rug::Rational::from(0);
        let mut alphas:Vec<rug::Rational> = vec![zero.clone()];

        for threshold in &proba_ratios.clone().ratios {
            alphas.push(
                rug::Rational::from(
                    rug::Rational::sum(
                        proba_ratios.probas.iter()
                            .zip(proba_ratios.ratios.iter())
                            .map(|(p,r)| {if r <= &threshold.clone() {p.clone()} else {zero.clone()}})
                            .collect::<Vec<rug::Rational>>()
                            .iter()
                        )
                )
            );
        };
        let betas:Vec<rug::Rational> = alphas.iter().rev().map(|a| a.clone()).collect();
        let mut alphas_betas = AlphasBetas::new(alphas.clone(), betas.clone());
        alphas_betas.sort();
        alphas_betas
    }

}





