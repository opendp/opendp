use std::rc::Rc;

use rug::{Float, float::Round};

use crate::{core::{Measure, Measurement, Metric, PrivacyRelation}, error::Fallible, meas::{GaussianDomain, LaplaceDomain}};

use super::PLDistribution;

const PREC:u32 = 64;

/// A Measure that comes with a privacy loss distribution.
#[derive(Clone)]
pub struct PLDSmoothedMaxDivergence<MI> where MI: Metric {
    privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>,
}

impl<MI> PLDSmoothedMaxDivergence<MI> where MI: Metric {
    pub fn new(privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>) -> Self {
        PLDSmoothedMaxDivergence {
            privacy_loss_distribution: privacy_loss_distribution
        }
    }

    pub fn f(&self, d_in: &MI::Distance) -> Vec<(f64, f64)> {
        (self.privacy_loss_distribution)(d_in).unwrap_or_default().f()
    }
}

impl<MI> Default for PLDSmoothedMaxDivergence<MI> where MI: Metric {
    fn default() -> Self {
        PLDSmoothedMaxDivergence::new(Rc::new(|_:&MI::Distance| Ok(PLDistribution::default())))
    }
}

impl<MI> PartialEq for PLDSmoothedMaxDivergence<MI> where MI:Metric {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<MI> Measure for PLDSmoothedMaxDivergence<MI> where MI: Metric, MI::Distance: Clone {
    type Distance = (f64, f64);
}

// A way to build privacy relations from privacy loss distribution approximation

pub fn make_pld_privacy_relation<MI>(privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>) -> PrivacyRelation<MI, PLDSmoothedMaxDivergence<MI>>
    where MI: Metric, MI::Distance: 'static + Clone {
    PrivacyRelation::new_fallible( move |d_in: &MI::Distance, (epsilon, delta): &(f64, f64)| {
        if delta<=&0.0 {
            return fallible!(InvalidDistance, "Privacy Loss Mechanism: delta must be positive")
        }
        let mut exp_epsilon = rug::Float::with_val_round(64, epsilon, Round::Down).0;
        exp_epsilon.exp_round(Round::Down);
        Ok(delta >= &privacy_loss_distribution(d_in)?.delta(exp_epsilon))
    })
}

// Gaussian mechanism
fn gaussian_cdf(x:Float, mu:Float, sigma:Float) -> Float {
    0.5*(1.0 + Float::erf((x-mu)/(Float::with_val(PREC,2.0).sqrt()*sigma)))
}

/// Gaussian pld
fn gaussian_pld<'a>(scale: f64, grid_size: usize) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = Float::with_val(PREC, -3.0)*scale.clone();
    let max = Float::with_val(PREC, 3.0)*scale.clone();
    let sigma = Float::with_val(PREC, scale);
    move |d_in| {
        let mu = Float::with_val(PREC, d_in);
        let mut last_x = Float::with_val(PREC, min.clone());
        let mut x = Float::with_val(PREC, min.clone());
        let mut exp_eps = Float::with_val(PREC, 0);
        let mut prob = gaussian_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(Float, Float)>::new();
        exp_privacy_loss_probabilities.push((
            exp_eps,
            prob
        ));
        for k in 0..grid_size {
            x = min.clone() + (max.clone()-min.clone())*Float::with_val(PREC, k+1)/Float::with_val(PREC, grid_size);
            exp_eps = Float::exp((mu.clone()*x.clone()-0.5*mu.clone().square())/sigma.clone().square());
            prob = gaussian_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone()) - gaussian_cdf(last_x.clone(),Float::with_val(PREC, 0),sigma.clone());
            exp_privacy_loss_probabilities.push((
                exp_eps.clone(),
                prob.clone(),
            ));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_gaussian<D>(scale: f64) -> Fallible<Measurement<D, D, D::Metric, PLDSmoothedMaxDivergence<D::Metric>>>
    where D: GaussianDomain<Atom=f64> {
    if scale<0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let privacy_loss_distribution = Rc::new(gaussian_pld(scale.clone(), 100));
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}

fn laplace_cdf(x:Float, mu:Float, sigma:Float) -> Float {
    let y = (x.clone()-mu.clone())/sigma.clone();
    Float::with_val(PREC, 0.5)*(
        Float::with_val(PREC, 1)+y.clone().signum()*(
            Float::with_val(PREC, 1)-Float::exp(-(y.clone().abs()))
        )
    )
}

fn laplace_pdf(x:Float, mu:Float, sigma:Float) -> Float {
    let y = (x.clone()-mu.clone())/sigma.clone();
    Float::exp(-(y.clone().abs()))/sigma.clone()
}

fn laplace_exp_eps(x:Float, mu:Float, sigma:Float) -> Float {
    Float::exp((x.clone()/sigma.clone()).abs()-((x.clone()-mu.clone())/sigma.clone()).abs())
}

/// Laplace pld
fn laplace_pld<'a>(scale: f64, grid_size: usize) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = Float::with_val(PREC, -5.0)*scale.clone();
    let max = Float::with_val(PREC, 5.0)*scale.clone();
    let sigma = Float::with_val(PREC, scale);
    move |d_in| {
        let mu = Float::with_val(PREC, d_in);
        let mut last_x = Float::with_val(PREC, min.clone());
        let mut x = Float::with_val(PREC, min.clone());
        let mut exp_eps = Float::exp(-mu.clone()/sigma.clone());
        let mut prob = laplace_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(Float, Float)>::new();
        exp_privacy_loss_probabilities.push((
            exp_eps,
            prob
        ));
        for k in 0..grid_size {
            x = min.clone() + (max.clone()-min.clone())*Float::with_val(PREC, k+1)/Float::with_val(PREC, grid_size);
            exp_eps = laplace_exp_eps(x.clone(),mu.clone(),sigma.clone());
            prob = laplace_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone()) - laplace_cdf(last_x.clone(),Float::with_val(PREC, 0),sigma.clone());
            // prob = laplace_pdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
            exp_privacy_loss_probabilities.push((
                exp_eps.clone(),
                prob.clone(),
            ));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_laplace<D>(scale: f64) -> Fallible<Measurement<D, D, D::Metric, PLDSmoothedMaxDivergence<D::Metric>>>
    where D: LaplaceDomain<Atom=f64> {
    if scale<0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let privacy_loss_distribution = Rc::new(laplace_pld(scale.clone(), 100));
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}