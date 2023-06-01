#[cfg(feature = "partials")]
use crate::core::{PartialMeasurement, PartialTransformation};

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, Transformation},
    error::Fallible,
};
use std::ops::Shr;

use super::{make_chain_mt, make_chain_pm, make_chain_tt};

// CHAINING TRANSFORMATION WITH TRANSFORMATION
// There are seven impls:
// 6 for the combinations of
//   {Transformation, Fallible<Transformation>, PartialTransformation} and
//   {Transformation, PartialTransformation}
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Transformation> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for (MI, MO) >> PartialTransformation chaining.

// Transformation >> Transformation
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>> for Transformation<DI, DX, MI, MX>
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
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self)
    }
}

// Fallible<Transformation> >> Transformation
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>>
    for Fallible<Transformation<DI, DX, MI, MX>>
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
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self?)
    }
}

// PartialTransformation >> Transformation
#[cfg(feature = "partials")]
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>>
    for PartialTransformation<DI, DX, MI, MX>
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
    type Output = PartialTransformation<DI, DO, MI, MO>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        PartialTransformation::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_tt(&rhs, &lhs)
        })
    }
}

// Transformation >> PartialTransformation
#[cfg(feature = "partials")]
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, DO, MX, MO>>
    for Transformation<DI, DX, MI, MX>
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
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: PartialTransformation<DX, DO, MX, MO>) -> Self::Output {
        let rhs = (rhs).fix(self.output_domain.clone(), self.output_metric.clone())?;
        make_chain_tt(&rhs, &self)
    }
}

// Fallible<Transformation> >> PartialTransformation
#[cfg(feature = "partials")]
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, DO, MX, MO>>
    for Fallible<Transformation<DI, DX, MI, MX>>
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
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: PartialTransformation<DX, DO, MX, MO>) -> Self::Output {
        let lhs = self?;
        let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
        make_chain_tt(&rhs, &lhs)
    }
}

// PartialTransformation >> PartialTransformation
#[cfg(feature = "partials")]
impl<DI, DX, DO, MI, MX, MO> Shr<PartialTransformation<DX, DO, MX, MO>>
    for PartialTransformation<DI, DX, MI, MX>
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
    type Output = PartialTransformation<DI, DO, MI, MO>;

    fn shr(self, rhs: PartialTransformation<DX, DO, MX, MO>) -> Self::Output {
        PartialTransformation::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
            make_chain_tt(&rhs, &lhs)
        })
    }
}

// (DI, MI) >> PartialTransformation
#[cfg(feature = "partials")]
impl<DI, DO, MI, MO> Shr<PartialTransformation<DI, DO, MI, MO>> for (DI, MI)
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DO, MO): MetricSpace,
{
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: PartialTransformation<DI, DO, MI, MO>) -> Self::Output {
        (rhs).fix(self.0, self.1)
    }
}

// CHAINING TRANSFORMATION WITH MEASUREMENT
// There are seven impls:
// 6 for the combinations of
//   {Transformation, Fallible<Transformation>, PartialTransformation} and
//   {Measurement, PartialMeasurement}
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Measurement> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for (MI, MO) >> PartialMeasurement chaining.

// Transformation >> Measurement
impl<DI, DX, TO, MI, MX, MO> Shr<Measurement<DX, TO, MX, MO>> for Transformation<DI, DX, MI, MX>
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
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, TO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self)
    }
}

// Fallible<Transformation> >> Measurement
impl<DI, DX, TO, MI, MX, MO> Shr<Measurement<DX, TO, MX, MO>>
    for Fallible<Transformation<DI, DX, MI, MX>>
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
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, TO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self?)
    }
}

// PartialTransformation >> Measurement
#[cfg(feature = "partials")]
impl<DI, DX, TO, MI, MX, MO> Shr<Measurement<DX, TO, MX, MO>>
    for PartialTransformation<DI, DX, MI, MX>
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
    type Output = PartialMeasurement<DI, TO, MI, MO>;

    fn shr(self, rhs: Measurement<DX, TO, MX, MO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_mt(&rhs, &lhs)
        })
    }
}

// Transformation >> PartialMeasurement
#[cfg(feature = "partials")]
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, TO, MX, MO>>
    for Transformation<DI, DX, MI, MX>
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
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: PartialMeasurement<DX, TO, MX, MO>) -> Self::Output {
        let rhs = rhs.fix(self.output_domain.clone(), self.output_metric.clone())?;
        make_chain_mt(&rhs, &self)
    }
}

// Fallible<Transformation> >> PartialMeasurement
#[cfg(feature = "partials")]
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, TO, MX, MO>>
    for Fallible<Transformation<DI, DX, MI, MX>>
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
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: PartialMeasurement<DX, TO, MX, MO>) -> Self::Output {
        let lhs = self?;
        let rhs = rhs.fix(lhs.output_domain.clone(), lhs.output_metric.clone())?;
        make_chain_mt(&rhs, &lhs)
    }
}

// PartialTransformation >> PartialMeasurement
#[cfg(feature = "partials")]
impl<DI, DX, TO, MI, MX, MO> Shr<PartialMeasurement<DX, TO, MX, MO>>
    for PartialTransformation<DI, DX, MI, MX>
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
    type Output = PartialMeasurement<DI, TO, MI, MO>;

    fn shr(self, rhs: PartialMeasurement<DX, TO, MX, MO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let transformation = self.fix(input_domain, input_metric)?;
            transformation >> rhs
        })
    }
}

// (DI, MI) >> PartialMeasurement
#[cfg(feature = "partials")]
impl<DI, TO, MI, MO> Shr<PartialMeasurement<DI, TO, MI, MO>> for (DI, MI)
where
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: PartialMeasurement<DI, TO, MI, MO>) -> Self::Output {
        (rhs).fix(self.0, self.1)
    }
}

// CHAINING POSTPROCESS WITH MEASUREMENT
// There are seven impls:
// 6 for the combinations of
//   {Measurement, Fallible<Measurement>, PartialMeasurement} and
//   {Function, Transformation}
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Measurement> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for Function >> Function chaining.

// Measurement >> Function
impl<DI, TX, TO, MI, MO> Shr<Function<TX, TO>> for Measurement<DI, TX, MI, MO>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        make_chain_pm(&rhs, &self)
    }
}

// Fallible<Measurement> >> Function
impl<DI, TX, TO, MI, MO> Shr<Function<TX, TO>> for Fallible<Measurement<DI, TX, MI, MO>>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, TO, MI, MO>>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        make_chain_pm(&rhs, &self?)
    }
}

// PartialMeasurement >> Function
#[cfg(feature = "partials")]
impl<DI, TX, TO, MI, MO> Shr<Function<TX, TO>> for PartialMeasurement<DI, TX, MI, MO>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = PartialMeasurement<DI, TO, MI, MO>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_pm(&rhs, &lhs)
        })
    }
}

// Measurement >> Transformation
impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, DO, MTI, MTO>>
    for Measurement<DI, DX::Carrier, MI, MO>
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
    type Output = Fallible<Measurement<DI, DO::Carrier, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MTI, MTO>) -> Self::Output {
        make_chain_pm(&rhs.function, &self)
    }
}

// Fallible<Measurement> >> Transformation
impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, DO, MTI, MTO>>
    for Fallible<Measurement<DI, DX::Carrier, MI, MO>>
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
    type Output = Fallible<Measurement<DI, DO::Carrier, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MTI, MTO>) -> Self::Output {
        make_chain_pm(&rhs.function, &self?)
    }
}

#[cfg(feature = "partials")]
impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, DO, MTI, MTO>>
    for PartialMeasurement<DI, DX::Carrier, MI, MO>
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
    type Output = PartialMeasurement<DI, DO::Carrier, MI, MO>;

    fn shr(self, rhs: Transformation<DX, DO, MTI, MTO>) -> Self::Output {
        PartialMeasurement::new(move |input_domain, input_metric| {
            let lhs = self.fix(input_domain, input_metric)?;
            make_chain_pm(&rhs.function, &lhs)
        })
    }
}

// Function >> Function
impl<TI, TX, TO> Shr<Function<TX, TO>> for Function<TI, TX>
where
    TI: 'static,
    TX: 'static,
    TO: 'static,
{
    type Output = Function<TI, TO>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        Function::make_chain(&rhs, &self)
    }
}

#[cfg(test)]
mod tests_shr {
    use crate::measurements::make_base_discrete_laplace;
    use crate::transformations::{
        make_bounded_sum, make_cast_default, make_clamp, make_split_lines,
    };

    use super::*;

    #[test]
    fn test_shr() -> Fallible<()> {
        (make_split_lines()?
            >> make_cast_default()?
            >> make_clamp((0, 1))?
            >> make_bounded_sum((0, 1))?
            >> make_base_discrete_laplace(1.)?)
        .map(|_| ())
    }
}
