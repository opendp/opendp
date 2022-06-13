use crate::{
    core::{Domain, Measure, Measurement, Metric, PrivacyMap},
    dist::{FixedSmoothedMaxDivergence, SmoothedMaxDivergence},
    error::Fallible,
};

#[cfg(feature = "ffi")]
pub mod ffi;

pub trait FixDeltaMeasure: Measure {
    type Atom;
    type FixedMeasure: Measure;

    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure>;
    fn fix_delta(
        &self,
        curve: &Self::Distance,
        delta: &Self::Atom,
    ) -> Fallible<<Self::FixedMeasure as Measure>::Distance>;
}

impl<Q: Clone> FixDeltaMeasure for SmoothedMaxDivergence<Q> {
    type Atom = Q;
    type FixedMeasure = FixedSmoothedMaxDivergence<Q>;

    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure> {
        Ok(FixedSmoothedMaxDivergence::default())
    }
    fn fix_delta(&self, curve: &Self::Distance, delta: &Q) -> Fallible<(Q, Q)> {
        curve.epsilon(delta).map(|v| (v, delta.clone()))
    }
}

pub fn make_fix_delta<DI, DO, MI, MO>(
    measurement: &Measurement<DI, DO, MI, MO>,
    delta: MO::Atom,
) -> Fallible<Measurement<DI, DO, MI, MO::FixedMeasure>>
where
    DI: Domain,
    DO: Domain,
    MI: 'static + Metric,
    MO: 'static + FixDeltaMeasure,
{
    let Measurement {
        input_domain,
        output_domain,
        function,
        input_metric,
        output_measure,
        privacy_map,
    } = measurement.clone();

    Ok(Measurement::new(
        input_domain,
        output_domain,
        function,
        input_metric,
        output_measure.new_fixed_measure()?,
        PrivacyMap::new_fallible(move |d_in| {
            // find the smallest epsilon at the given delta
            let curve = privacy_map.eval(d_in)?;
            output_measure.fix_delta(&curve, &delta)
        }),
    ))
}
