use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::IntoIterator;

use crate::error::Fallible;
use crate::dom::AllDomain;
use crate::dist::{IntDistance, SymmetricDistance, SmoothedMaxDivergence};
use crate::core::{Domain, Function, Measurement, PrivacyRelation};
use rug::{Float, Rational, float::Round};

use super::PLDistribution;

/// Privacy Loss Measurement (PLM) inspired from PLD http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

pub type PLMInputDomain = AllDomain<bool>;

pub type PLMOutputDomain = PLDistribution;

impl Domain for PLMOutputDomain {
    type Carrier = Rational;
    fn member(&self, privacy_loss: &Self::Carrier) -> Fallible<bool> { Ok(self.exp_privacy_loss_probabilities.contains_key(privacy_loss)) }
}

pub trait FDifferentialPrivacy {
    fn f(&self, n:usize) -> Vec<(f64, f64)>;
}

impl FDifferentialPrivacy for PLMMeasurement {
    fn f(&self, n:usize) -> Vec<(f64, f64)> {
        self.output_domain.tradeoff(n).into_iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect()
    }
}

pub type PLMMeasurement = Measurement<PLMInputDomain, PLMOutputDomain, SymmetricDistance, SmoothedMaxDivergence<Rational>>;

pub fn make_plm<'a,I,Q>(exp_privacy_loss_probabilitiies:I) -> Fallible<PLMMeasurement>
where I: 'a + IntoIterator<Item=&'a (Q, Q)>, Q: 'a + Clone, Rational: TryFrom<Q> {
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
        Ok(delta >= &out_dom.delta(exp_epsilon))
    })
}


// Measurement<PLMInputDomain, PLMOutputDomain, SymmetricDistance, SmoothedMaxDivergence<Rational>>
pub fn make_basic_composition(measurement0: &PLMMeasurement, measurement1: &PLMMeasurement) -> Fallible<PLMMeasurement> {
        if measurement0.input_domain != measurement1.input_domain {
            return fallible!(DomainMismatch, "Input domain mismatch");
        } else if measurement0.input_metric != measurement1.input_metric {
            return fallible!(MetricMismatch, "Input metric mismatch");
        } else if measurement0.output_measure != measurement1.output_measure {
            return fallible!(MeasureMismatch, "Output measure mismatch");
        }
        Ok(make_plm(&measurement0.output_domain.exp_privacy_loss_probabilities.clone().into_iter().collect::<Vec<(Rational, Rational)>>())?)
}