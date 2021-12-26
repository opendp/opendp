use crate::core::{Domain, Measure, Measurement, Metric, PrivacyRelation};
use crate::dist::{GaussianTradeOff, MaxDivergence, RenyiDivergence, SmoothedMaxDivergence, UnionRenyiDivergence};
use crate::error::Fallible;

// TODO: replace relation with forward map
trait ForwardMap<QI, QO> {
    fn forward_map(&self, d_in: QI) -> Fallible<QO>;
}
impl<MI: Metric, MO: Measure> ForwardMap<MI::Distance, MO::Distance> for PrivacyRelation<MI, MO> {
    fn forward_map(&self, _d_in: MI::Distance) -> Fallible<MO::Distance> {
        unimplemented!()
    }
}

pub trait CastMeasure<MI: Metric, MO: Measure>: Measure {
    fn cast_relation(measure: MO, relation: PrivacyRelation<MI, MO>) -> (Self, PrivacyRelation<MI, Self>);
}

// TODO: loosen trait bound, f64 is too restrictive
impl<MI: 'static + Metric<Distance=f64>> CastMeasure<MI, GaussianTradeOff<f64>> for SmoothedMaxDivergence<f64> {
    fn cast_relation(
        _measure: GaussianTradeOff<f64>,
        relation: PrivacyRelation<MI, GaussianTradeOff<f64>>
    ) -> (Self, PrivacyRelation<MI, Self>) {
        (
            SmoothedMaxDivergence::default(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, (eps, del): &(f64, f64)| {
                // https://arxiv.org/pdf/1905.02383.pdf#page=13 Corollary 2.13
                use statrs::function::erf;
                fn phi(t: f64) -> f64 {
                    0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
                }
                let mu: f64 = relation.forward_map(d_in)?;
                Ok(*del >= phi(mu / 2. - eps / mu) - eps.exp() * phi(-eps / mu - mu / 2.))
            })
        )
    }
}

impl<MI: 'static + Metric<Distance=f64>> CastMeasure<MI, RenyiDivergence<f64>> for SmoothedMaxDivergence<f64> {
    fn cast_relation(
        measure: RenyiDivergence<f64>,
        relation: PrivacyRelation<MI, RenyiDivergence<f64>>
    ) -> (SmoothedMaxDivergence<f64>, PrivacyRelation<MI, Self>) {
        (
            SmoothedMaxDivergence::default(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, (eps, del): &(f64, f64)| {
                // https://arxiv.org/pdf/1702.07476.pdf#page=5 Proposition 3
                let renyi_eps = relation.forward_map(d_in)?;
                Ok(*eps >= renyi_eps + del.recip().ln() / f64::from(measure.alpha - 1))
            })
        )
    }
}


impl<MI: 'static + Metric<Distance=f64>> CastMeasure<MI, UnionRenyiDivergence<f64>> for SmoothedMaxDivergence<f64> {
    fn cast_relation(
        _measure: UnionRenyiDivergence<f64>,
        relation: PrivacyRelation<MI, UnionRenyiDivergence<f64>>
    ) -> (SmoothedMaxDivergence<f64>, PrivacyRelation<MI, Self>) {
        (
            SmoothedMaxDivergence::default(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, (eps, del): &(f64, f64)| {
                // https://arxiv.org/pdf/1605.02065.pdf#page=6 Proposition 1.3
                let rho = relation.forward_map(d_in)?;
                Ok(*eps >= rho + 2. * (rho * del.recip().ln()).sqrt())
            })
        )
    }
}

// convert from eps-DP to pure zCDP
impl<MI: 'static + Metric<Distance=f64>> CastMeasure<MI, MaxDivergence<f64>> for UnionRenyiDivergence<f64> {
    fn cast_relation(
        _measure: MaxDivergence<f64>,
        relation: PrivacyRelation<MI, MaxDivergence<f64>>
    ) -> (UnionRenyiDivergence<f64>, PrivacyRelation<MI, Self>) {
        (
            UnionRenyiDivergence::default(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, &eps: &f64| {
                // https://arxiv.org/pdf/1605.02065.pdf#page=6 Proposition 1.4
                let rho = relation.forward_map(d_in)?;
                Ok(rho >= eps.powi(2) / 2.)
            })
        )
    }
}

pub fn make_cast_measure<DI: Domain, DO: Domain, MI: Metric, MO1: Measure, MO2: Measure>(
    meas: Measurement<DI, DO, MI, MO1>
) -> Fallible<Measurement<DI, DO, MI, MO2>>
    where MO2: CastMeasure<MI, MO1> {
    let Measurement {
        input_domain, output_domain, function,
        input_metric, output_measure, privacy_relation
    } = meas;

    let (output_measure, privacy_relation) =
        MO2::cast_relation(output_measure, privacy_relation);

    Ok(Measurement::new(
        input_domain,
        output_domain,
        function,
        input_metric,
        output_measure,
        privacy_relation
    ))
}