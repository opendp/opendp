use std::rc::Rc;

use rug::{float::Round, Float as RugFloat};

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{GaussianDomain, LaplaceDomain},
    measures::SMDCurve,
    metrics::SymmetricDistance,
    traits::{
        samplers::{SampleDiscreteGaussianZ2k, SampleDiscreteLaplaceZ2k},
        Float,
    },
};

use super::PLDistribution;

const PREC: u32 = 128;
const GRID_SIZE: usize = 100;

/// A Measure that comes with a privacy loss distribution.
#[derive(Clone)]
pub struct PLDSmoothedMaxDivergence<MI>
where
    MI: Metric,
{
    privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>,
}

impl<MI> PLDSmoothedMaxDivergence<MI>
where
    MI: Metric,
{
    pub fn new(
        privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>,
    ) -> Self {
        PLDSmoothedMaxDivergence {
            privacy_loss_distribution: privacy_loss_distribution,
        }
    }

    pub fn f(&self, d_in: &MI::Distance) -> Vec<(f64, f64)> {
        (self.privacy_loss_distribution)(d_in)
            .unwrap_or_default()
            .f()
    }

    pub fn simplified_f(&self, d_in: &MI::Distance) -> Vec<(f64, f64)> {
        (self.privacy_loss_distribution)(d_in)
            .unwrap_or_default()
            .simplified()
            .f()
    }
}

impl<MI> Default for PLDSmoothedMaxDivergence<MI>
where
    MI: Metric,
{
    fn default() -> Self {
        PLDSmoothedMaxDivergence::new(Rc::new(|_: &MI::Distance| Ok(PLDistribution::default())))
    }
}

impl<MI> PartialEq for PLDSmoothedMaxDivergence<MI>
where
    MI: Metric,
{
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<MI> Measure for PLDSmoothedMaxDivergence<MI>
where
    MI: Metric,
    MI::Distance: Clone,
{
    type Distance = SMDCurve<f64>;
}

impl<MI: Metric> std::fmt::Debug for PLDSmoothedMaxDivergence<MI> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PLDSmoothedMaxDivergence").finish()
    }
}

// A way to build privacy relations from privacy loss distribution approximation

pub fn make_pld_privacy_map<MI>(
    privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>,
) -> PrivacyMap<MI, PLDSmoothedMaxDivergence<MI>>
where
    MI: Metric,
    MI::Distance: 'static + Clone,
{
    PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
        // TODO: this is backwards-- delta is what really comes in here
        let d_in = d_in.clone();
        let privacy_loss_distribution = privacy_loss_distribution.clone();
        Ok(SMDCurve::new(move |epsilon| {
            let mut exp_epsilon = rug::Float::with_val_round(64, epsilon, Round::Down).0;
            exp_epsilon.exp_round(Round::Down);
            Ok(privacy_loss_distribution(&d_in)?
                .delta(&exp_epsilon.to_rational().unwrap())
                .to_f64())
        }))
    })
}

// Gaussian mechanism
fn gaussian_cdf(x: RugFloat, mu: RugFloat, sigma: RugFloat) -> RugFloat {
    0.5 * (1.0 + RugFloat::erf((x - mu) / (RugFloat::with_val(PREC, 2.0).sqrt() * sigma)))
}

/// Gaussian pld
fn gaussian_pld<'a>(
    scale: f64,
    grid_size: usize,
) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = RugFloat::with_val(PREC, -3.0) * scale.clone();
    let max = RugFloat::with_val(PREC, 3.0) * scale.clone();
    let sigma = RugFloat::with_val(PREC, scale);
    move |d_in| {
        let mu = RugFloat::with_val(PREC, d_in);
        let mut last_x = RugFloat::with_val(PREC, min.clone());
        let mut x = RugFloat::with_val(PREC, min.clone());
        let mut exp_eps = RugFloat::with_val(PREC, 0);
        let mut prob = gaussian_cdf(x.clone(), RugFloat::with_val(PREC, 0), sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(RugFloat, RugFloat)>::new();
        exp_privacy_loss_probabilities.push((exp_eps, prob));
        for k in 0..grid_size {
            x = min.clone()
                + (max.clone() - min.clone()) * RugFloat::with_val(PREC, k + 1)
                    / RugFloat::with_val(PREC, grid_size);
            exp_eps = RugFloat::exp(
                (mu.clone() * x.clone() - 0.5 * mu.clone().square()) / sigma.clone().square(),
            );
            prob = gaussian_cdf(x.clone(), RugFloat::with_val(PREC, 0), sigma.clone())
                - gaussian_cdf(last_x.clone(), RugFloat::with_val(PREC, 0), sigma.clone());
            exp_privacy_loss_probabilities.push((exp_eps.clone(), prob.clone()));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_gaussian<D>(
    scale: f64,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, PLDSmoothedMaxDivergence<D::InputMetric>>>
where
    D: GaussianDomain<Atom = f64>,
    D::Atom: Float + SampleDiscreteLaplaceZ2k,
    (D, D::InputMetric): MetricSpace,
{
    if scale < 0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let privacy_loss_distribution = Rc::new(gaussian_pld(scale.clone(), GRID_SIZE));
    Measurement::new(
        D::default(),
        D::new_map_function(move |value| {
            D::Atom::sample_discrete_gaussian_Z2k(*value, scale.clone(), 1074)
        }),
        D::InputMetric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_map(privacy_loss_distribution),
    )
}

fn laplace_cdf(x: RugFloat, mu: RugFloat, sigma: RugFloat) -> RugFloat {
    let y = (x.clone() - mu.clone()) / sigma.clone();
    RugFloat::with_val(PREC, 0.5)
        * (RugFloat::with_val(PREC, 1)
            + y.clone().signum()
                * (RugFloat::with_val(PREC, 1) - RugFloat::exp(-(y.clone().abs()))))
}

// fn laplace_pdf(x:RugFloat, mu:RugFloat, sigma:RugFloat) -> RugFloat {
//     let y = (x.clone()-mu.clone())/sigma.clone();
//     RugFloat::exp(-(y.clone().abs()))/sigma.clone()
// }

fn laplace_exp_eps(x: RugFloat, mu: RugFloat, sigma: RugFloat) -> RugFloat {
    if x.clone() - &mu < RugFloat::with_val(PREC, -5) * &sigma {
        RugFloat::with_val(PREC, 0)
    } else if x.clone() - &mu > RugFloat::with_val(PREC, 5) * &sigma {
        RugFloat::with_val(PREC, 1)
    } else {
        RugFloat::exp(
            (x.clone() / sigma.clone()).abs() - ((x.clone() - mu.clone()) / sigma.clone()).abs(),
        )
    }
}

/// Laplace pld
fn laplace_pld<'a>(scale: f64, grid_size: usize) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = RugFloat::with_val(PREC, -5.0) * scale.clone();
    let max = RugFloat::with_val(PREC, 5.0) * scale.clone();
    let sigma = RugFloat::with_val(PREC, scale);
    move |d_in| {
        let mu = RugFloat::with_val(PREC, d_in);
        let mut last_x = RugFloat::with_val(PREC, min.clone());
        let mut x = RugFloat::with_val(PREC, min.clone());
        let mut exp_eps = RugFloat::exp(-mu.clone() / sigma.clone());
        let mut prob = laplace_cdf(x.clone(), RugFloat::with_val(PREC, 0), sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(RugFloat, RugFloat)>::new();
        exp_privacy_loss_probabilities.push((exp_eps, prob));
        for k in 0..grid_size {
            x = min.clone()
                + (max.clone() - min.clone()) * RugFloat::with_val(PREC, k + 1)
                    / RugFloat::with_val(PREC, grid_size);
            exp_eps = laplace_exp_eps(x.clone(), mu.clone(), sigma.clone());
            prob = laplace_cdf(x.clone(), RugFloat::with_val(PREC, 0), sigma.clone())
                - laplace_cdf(last_x.clone(), RugFloat::with_val(PREC, 0), sigma.clone());
            // prob = laplace_pdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
            exp_privacy_loss_probabilities.push((exp_eps.clone(), prob.clone()));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_laplace<D>(
    scale: f64,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, PLDSmoothedMaxDivergence<D::InputMetric>>>
where
    D: LaplaceDomain<Atom = f64>,
    D::Atom: Float + SampleDiscreteGaussianZ2k,
    (D, D::InputMetric): MetricSpace,
{
    if scale < 0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let privacy_loss_distribution = Rc::new(laplace_pld(scale.clone(), GRID_SIZE));
    Measurement::new(
        D::default(),
        D::new_map_function(move |value| {
            D::Atom::sample_discrete_laplace_Z2k(*value, scale.clone(), 1074)
        }),
        D::InputMetric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_map(privacy_loss_distribution),
    )
}

/// Randmized Response
fn epsilon_delta_pld<'a>(
    epsilon: f64,
    delta: f64,
) -> impl Fn(&u32) -> Fallible<PLDistribution> + 'a {
    move |n| {
        let n_exp_eps = ((*n as f64) * epsilon).exp();
        let n_delta = (*n as f64) * delta;
        Ok(PLDistribution::from(vec![
            (0.0, n_delta),
            (n_exp_eps, (1.0 - n_delta) / (1.0 + n_exp_eps)),
            (
                n_exp_eps.recip(),
                (1.0 - n_delta) * n_exp_eps / (1.0 + n_exp_eps),
            ),
        ]))
    }
}

pub fn make_pld_epsilon_delta(
    epsilon: f64,
    delta: f64,
) -> Fallible<
    Measurement<
        VectorDomain<AtomDomain<u32>>,
        AtomDomain<u32>,
        SymmetricDistance,
        PLDSmoothedMaxDivergence<SymmetricDistance>,
    >,
> {
    if delta < 0.0 {
        return fallible!(MakeMeasurement, "delta must not be negative");
    }
    let privacy_loss_distribution = Rc::new(epsilon_delta_pld(epsilon, delta));
    Measurement::new(
        VectorDomain::new(AtomDomain::new_closed((0u32, 1u32))?),
        Function::new_fallible(|&_| fallible!(NotImplemented, "not implemented")),
        SymmetricDistance::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_map(privacy_loss_distribution),
    )
}

pub fn make_pld_composition<DI, TO0, TO1, MI>(
    measurement0: &Measurement<DI, TO0, MI, PLDSmoothedMaxDivergence<MI>>,
    measurement1: &Measurement<DI, TO1, MI, PLDSmoothedMaxDivergence<MI>>,
) -> Fallible<Measurement<DI, (TO0, TO1), MI, PLDSmoothedMaxDivergence<MI>>>
where
    DI: 'static + Domain,
    TO0: 'static,
    TO1: 'static,
    MI: 'static + Metric,
    MI::Distance: Clone,
    (DI, MI): MetricSpace,
{
    if measurement0.input_domain != measurement1.input_domain {
        return fallible!(DomainMismatch, "Input domain mismatch");
    } else if measurement0.input_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, "Input metric mismatch");
    }
    let pld_0 = measurement0
        .output_measure
        .privacy_loss_distribution
        .clone();
    let pld_1 = measurement1
        .output_measure
        .privacy_loss_distribution
        .clone();
    let privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>> =
        Rc::new(move |d_in: &MI::Distance| Ok(&(pld_0)(d_in)? * &(pld_1)(d_in)?));
    let funcs = (measurement0.function.clone(), measurement1.function.clone());
    Measurement::new(
        measurement0.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            Ok((funcs.0.eval(arg)?, funcs.1.eval(arg)?))
        }),
        measurement0.input_metric.clone(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_map(privacy_loss_distribution),
    )
}
