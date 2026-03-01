#[cfg(feature = "partials")]
mod partials;

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
//
// Partial impls are in the partials.rs module.
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Transformation> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for (MI, MO) >> PartialTransformation chaining.

// Transformation >> Transformation
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, MX, DO, MO>> for Transformation<DI, MI, DX, MX>
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

    fn shr(self, rhs: Transformation<DX, MX, DO, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self)
    }
}

// Fallible<Transformation> >> Transformation
impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, MX, DO, MO>>
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

    fn shr(self, rhs: Transformation<DX, MX, DO, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self?)
    }
}

// CHAINING TRANSFORMATION WITH MEASUREMENT
// There are seven impls:
// 6 for the combinations of
//   {Transformation, Fallible<Transformation>, PartialTransformation} and
//   {Measurement, PartialMeasurement}
//
// Partial impls are in the partials.rs module.
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Measurement> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for (MI, MO) >> PartialMeasurement chaining.

// Transformation >> Measurement
impl<DI, DX, MI, MX, MO, TO> Shr<Measurement<DX, MX, MO, TO>> for Transformation<DI, MI, DX, MX>
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

    fn shr(self, rhs: Measurement<DX, MX, MO, TO>) -> Self::Output {
        make_chain_mt(&rhs, &self)
    }
}

// Fallible<Transformation> >> Measurement
impl<DI, DX, MI, MX, MO, TO> Shr<Measurement<DX, MX, MO, TO>>
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

    fn shr(self, rhs: Measurement<DX, MX, MO, TO>) -> Self::Output {
        make_chain_mt(&rhs, &self?)
    }
}
// CHAINING POSTPROCESS WITH MEASUREMENT
// There are seven impls:
// 6 for the combinations of
//   {Measurement, Fallible<Measurement>, PartialMeasurement} and
//   {Function, Transformation}
//
// Partial impls are in the partials.rs module.
// Partials are never wrapped in Fallible, so Fallible<PartialTransformation> are not included in the impls.
// On the RHS, Fallible<Measurement> is not included, because it is trivial to ?-unwrap.
// The seventh impl is for Function >> Function chaining.

// Measurement >> Function
impl<DI, MI, MO, TX, TO> Shr<Function<TX, TO>> for Measurement<DI, MI, MO, TX>
where
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    TX: 'static,
    TO: 'static,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, MI, MO, TO>>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        make_chain_pm(&rhs, &self)
    }
}

// Fallible<Measurement> >> Function
impl<DI, TX, TO, MI, MO> Shr<Function<TX, TO>> for Fallible<Measurement<DI, MI, MO, TX>>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    type Output = Fallible<Measurement<DI, MI, MO, TO>>;

    fn shr(self, rhs: Function<TX, TO>) -> Self::Output {
        make_chain_pm(&rhs, &self?)
    }
}

// Measurement >> Transformation
impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, MTI, DO, MTO>>
    for Measurement<DI, MI, MO, DX::Carrier>
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
    type Output = Fallible<Measurement<DI, MI, MO, DO::Carrier>>;

    fn shr(self, rhs: Transformation<DX, MTI, DO, MTO>) -> Self::Output {
        make_chain_pm(&rhs.function, &self)
    }
}

// Fallible<Measurement> >> Transformation
impl<DI, DX, DO, MI, MO, MTI, MTO> Shr<Transformation<DX, MTI, DO, MTO>>
    for Fallible<Measurement<DI, MI, MO, DX::Carrier>>
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
    type Output = Fallible<Measurement<DI, MI, MO, DO::Carrier>>;

    fn shr(self, rhs: Transformation<DX, MTI, DO, MTO>) -> Self::Output {
        make_chain_pm(&rhs.function, &self?)
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
