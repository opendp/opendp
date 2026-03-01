use crate::core::{PartialMeasurement, PartialTransformation};

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, Transformation},
    error::Fallible,
};
use std::ops::Shr;

use super::{make_chain_mt, make_chain_pm, make_chain_tt};

// CHAINING TRANSFORMATION WITH TRANSFORMATION

// PartialTransformation >> Transformation
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, MX, DO, MO>>
    for PartialTransformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = PartialTransformation<DI, MI, DO, MO>;

    fn shr(self, rhs: Transformation<DX, MX, DO, MO>) -> Self::Output {
        PartialTransformation::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_tt(&rhs, &lhs)
        })
    }
}

// Transformation >> PartialTransformation
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, MX, DO, MO>>
    for Transformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = Fallible<Transformation<DI, MI, DO, MO>>;

    fn shr(self, rhs: PartialTransformation<DX, MX, DO, MO>) -> Self::Output {
        let rhs = (rhs).fix(self.output_domain.clone(), self.output_metric.clone())?;
        make_chain_tt(&rhs, &self)
    }
}

// Fallible<Transformation> >> PartialTransformation
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, MX, DO, MO>>
    for Fallible<Transformation<DI, MI, DX, MX>>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = Fallible<Transformation<DI, MI, DO, MO>>;

    fn shr(self, rhs: PartialTransformation<DX, MX, DO, MO>) -> Self::Output {
        let lhs = self?;
        let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
        make_chain_tt(&rhs, &lhs)
    }
}

// PartialTransformation >> PartialTransformation
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, MX, DO, MO>>
    for PartialTransformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = PartialTransformation<DI, MI, DO, MO>;

    fn shr(self, rhs: PartialTransformation<DX, MX, DO, MO>) -> Self::Output {
        PartialTransformation::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
            make_chain_tt(&rhs, &lhs)
        })
    }
}

// (DI, MI) >> PartialTransformation
impl<DI, DO, MI, MO> Shr<PartialTransformation<DI, MI, DO, MO>> for (DI, MI)
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = Fallible<Transformation<DI, MI, DO, MO>>;

    fn shr(self, rhs: PartialTransformation<DI, MI, DO, MO>) -> Self::Output {
        (rhs).fix(self.0, self.1)
    }
}

// CHAINING TRANSFORMATION WITH MEASUREMENT

// PartialTransformation >> Measurement
impl<DI, DX, TO, MI, MX, MO> Shr<Measurement<DX, MX, MO, TO>>
    for PartialTransformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    type Output = PartialMeasurement<DI, MI, MO, TO>;

    fn shr(self, rhs: Measurement<DX, MX, MO, TO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_mt(&rhs, &lhs)
        })
    }
}

// Transformation >> PartialMeasurement
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, MX, MO, TO>>
    for Transformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    type Output = Fallible<Measurement<DI, MI, MO, TO>>;

    fn shr(self, rhs: PartialMeasurement<DX, MX, MO, TO>) -> Self::Output {
        let rhs = rhs.fix(self.output_domain.clone(), self.output_metric.clone())?;
        make_chain_mt(&rhs, &self)
    }
}

// Fallible<Transformation> >> PartialMeasurement
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, MX, MO, TO>>
    for Fallible<Transformation<DI, MI, DX, MX>>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    type Output = Fallible<Measurement<DI, MI, MO, TO>>;

    fn shr(self, rhs: PartialMeasurement<DX, MX, MO, TO>) -> Self::Output {
        let lhs = self?;
        let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
        make_chain_mt(&rhs, &lhs)
    }
}

// PartialTransformation >> PartialMeasurement
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, MX, MO, TO>>
    for PartialTransformation<DI, MI, DX, MX>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    type Output = PartialMeasurement<DI, MI, MO, TO>;

    fn shr(self, rhs: PartialMeasurement<DX, MX, MO, TO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let transformation = self.fix(input_domain, input_metric)?;
            transformation >> rhs
        })
    }
}

// (DI, MI) >> PartialMeasurement
impl<DI, TO, MI, MO> Shr<PartialMeasurement<DI, MI, MO, TO>> for (DI, MI)
where
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, MI, MO, TO>>;

    fn shr(self, rhs: PartialMeasurement<DI, MI, MO, TO>) -> Self::Output {
        (rhs).fix(self.0, self.1)
    }
}

// CHAINING POSTPROCESS WITH MEASUREMENT

// PartialMeasurement >> Function
impl<DI, TX, TO, MI, MO> Shr<Function<TX, TO>> for PartialMeasurement<DI, MI, MO, TX>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = PartialMeasurement<DI, MI, MO, TO>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_pm(&rhs, &lhs)
        })
    }
}

impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, MTI, DO, MTO>>
    for PartialMeasurement<DI, MI, MO, DX::Carrier>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    MTI: 'static + Metric,
    MTO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MTI): MetricSpace,
    (DO, MTO): MetricSpace,
{
    type Output = PartialMeasurement<DI, MI, MO, DO::Carrier>;

    fn shr(self, rhs: Transformation<DX, MTI, DO, MTO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_pm(&rhs.function, &lhs)
        })
    }
}

#[cfg(test)]
mod tests_shr {
    use crate::{
        measurements::then_laplace,
        transformations::{make_split_lines, then_cast_default, then_clamp, then_sum},
    };

    use super::*;

    #[test]
    fn test_shr() -> Fallible<()> {
        (make_split_lines()?
            >> then_cast_default()
            >> then_clamp((0, 1))
            >> then_sum()
            >> then_laplace(1., None))
        .map(|_| ())
    }
}
