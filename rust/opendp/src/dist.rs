//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;
use num::{One, Zero, Float};

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric, PrivacyRelation};
use crate::error::Fallible;
use crate::traits::{Tolerance, Midpoint};
use crate::samplers::CastInternalReal;

// default type for distances between datasets
pub type IntDistance = u32;

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for MaxDivergence<Q> {
    fn default() -> Self { MaxDivergence(PhantomData) }
}

impl<Q> PartialEq for MaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<Q: Clone> Measure for MaxDivergence<Q> {
    type Distance = Q;
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for SmoothedMaxDivergence<Q> {
    fn default() -> Self { SmoothedMaxDivergence(PhantomData) }
}

impl<Q> PartialEq for SmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<Q: Clone> Measure for SmoothedMaxDivergence<Q> {
    type Distance = (Q, Q);
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
use core::fmt::Debug;

impl<MI, Q> PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>
     where MI: Metric,
           Q: Clone + Zero + One + Float + Midpoint + Tolerance + CastInternalReal,// + Float + One + Zero + Tolerance + Midpoint + PartialOrd + CastInternalReal,
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

        // let step = (max_epsilon.clone().exp() - min_epsilon.clone().exp()) / rug::Float::with_val(53, npoints - 1);
        // (0..npoints)
        //     .map(|i| Q::from_internal(
        //         (min_epsilon.clone().exp() + step.clone() * rug::Float::with_val(4, i)).ln()
        //     ))
        //     .map(|eps| EpsilonDelta{
        //         epsilon: eps.clone(),
        //         delta: self.find_delta(&d_in, eps.clone()).unwrap()
        //     })
        //     .rev()
        //     .collect()
    }
}

/// Metrics
#[derive(Clone)]
pub struct SymmetricDistance;

impl Default for SymmetricDistance {
    fn default() -> Self { SymmetricDistance }
}

impl PartialEq for SymmetricDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Metric for SymmetricDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for SymmetricDistance {}

#[derive(Clone)]
pub struct SubstituteDistance;

impl Default for SubstituteDistance {
    fn default() -> Self { SubstituteDistance }
}

impl PartialEq for SubstituteDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl Metric for SubstituteDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for SubstituteDistance {}

// Sensitivity in P-space
pub struct LpDistance<Q, const P: usize>(PhantomData<Q>);
impl<Q, const P: usize> Default for LpDistance<Q, P> {
    fn default() -> Self { LpDistance(PhantomData) }
}

impl<Q, const P: usize> Clone for LpDistance<Q, P> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q, const P: usize> PartialEq for LpDistance<Q, P> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q, const P: usize> Metric for LpDistance<Q, P> {
    type Distance = Q;
}
impl<Q, const P: usize> SensitivityMetric for LpDistance<Q, P> {}

pub type L1Distance<Q> = LpDistance<Q, 1>;
pub type L2Distance<Q> = LpDistance<Q, 2>;


pub struct AbsoluteDistance<Q>(PhantomData<Q>);
impl<Q> Default for AbsoluteDistance<Q> {
    fn default() -> Self { AbsoluteDistance(PhantomData) }
}

impl<Q> Clone for AbsoluteDistance<Q> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q> PartialEq for AbsoluteDistance<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q> Metric for AbsoluteDistance<Q> {
    type Distance = Q;
}
impl<Q> SensitivityMetric for AbsoluteDistance<Q> {}

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


// Tradeoff
#[derive(Debug)]
pub struct Tradeoff {
    pub alphas: Vec<rug::Rational>,
    pub betas: Vec<rug::Rational>,
}

impl Clone for Tradeoff {
    fn clone(&self) -> Self {
        Tradeoff {alphas: self.alphas.clone(), betas: self.betas.clone()}
    }
}

impl Tradeoff {
    pub fn new (alphas: Vec<rug::Rational>, betas: Vec<rug::Rational>) -> Self {
        Tradeoff {
            alphas: alphas,
            betas: betas,
        }
    }

    pub fn new_from_vec_epsilon_delta <Q> (mut vec_epsilon_delta: Vec<EpsilonDelta<Q>>) -> Self
        where Q: 'static + One + Zero + PartialOrd + CastInternalReal + Clone + Debug {
        let one = Q::one().into_internal().to_rational().unwrap();
        let zero = Q::zero().into_internal().to_rational().unwrap();

        vec_epsilon_delta.sort_by(|a, b| b.delta.partial_cmp(&a.delta).unwrap());
        let rational_vec_exp_epsilon_delta: Vec<(rug::Rational, rug::Rational)> = vec_epsilon_delta.iter()
            .map(|x| {
                let mut exp_epsilon = x.epsilon.clone().into_internal();
                exp_epsilon.exp_round(rug::float::Round::Up);
                (exp_epsilon.to_rational().unwrap(), x.delta.clone().into_internal().to_rational().unwrap())
            })
            .collect();

        let mut alphas = vec![zero.clone(), one.clone() - rational_vec_exp_epsilon_delta[0].1.clone()];
        let mut betas = vec![one.clone() - rational_vec_exp_epsilon_delta[0].1.clone(), zero.clone()];

        let size = vec_epsilon_delta.iter().len();
        for i in 1..size {
            let alpha =
                (rational_vec_exp_epsilon_delta[i-1].1.clone() - rational_vec_exp_epsilon_delta[i].1.clone())
                /
                (rational_vec_exp_epsilon_delta[i].0.clone() - rational_vec_exp_epsilon_delta[i-1].0.clone());

            let beta = (
                    rational_vec_exp_epsilon_delta[i].0.clone() *(one.clone() - rational_vec_exp_epsilon_delta[i-1].1.clone())
                    -
                    rational_vec_exp_epsilon_delta[i-1].0.clone() *(one.clone() - rational_vec_exp_epsilon_delta[i].1.clone())
                )
                /
                (rational_vec_exp_epsilon_delta[i].0.clone() - rational_vec_exp_epsilon_delta[i-1].0.clone());
            alphas.push(alpha.clone());
            betas.push(beta.clone());
            alphas.push(beta);
            betas.push(alpha);
        }

        alphas.sort();
        betas.sort();
        betas.reverse();
        Self::new(alphas, betas)
    }

}



