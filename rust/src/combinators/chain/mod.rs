#[cfg(feature = "ffi")]
mod ffi;

mod shr;

use crate::core::{
    Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap, StabilityMap,
    Transformation,
};
use crate::error::{Error, ErrorVariant, Fallible};
use std::fmt::Debug;

const ERROR_URL: &str = "https://github.com/opendp/opendp/discussions/297";

macro_rules! assert_components_match {
    ($variant:ident, $v1:expr, $v2:expr) => {
        if &$v1 != &$v2 {
            return Err($crate::combinators::mismatch_error(
                $crate::error::ErrorVariant::$variant,
                &$v1,
                &$v2,
            ));
        }
    };
}
pub(crate) use assert_components_match;

pub(crate) fn mismatch_error<T: Debug>(variant: ErrorVariant, struct1: &T, struct2: &T) -> Error {
    let str1 = format!("{:?}", struct1);
    let str2 = format!("{:?}", struct2);

    let mode = match &variant {
        ErrorVariant::DomainMismatch => "domain",
        ErrorVariant::MetricMismatch => "metric",
        ErrorVariant::MeasureMismatch => "measure",
        _ => unimplemented!("unrecognized error variant"),
    };

    let explanation = if str1 == str2 {
        format!("\n    The structure of the intermediate {mode}s are the same, but the parameters differ.\n    shared_{mode}: {str1}\n", mode=mode, str1=str1)
    } else {
        format!(
            "\n    output_{mode}: {struct1}\n    input_{mode}:  {struct2}\n",
            mode = mode,
            struct1 = str1,
            struct2 = str2
        )
    };

    Error {
        variant,
        message: Some(format!(
            "Intermediate {}s don't match. See {}{}",
            mode, ERROR_URL, explanation
        )),
        backtrace: err!(@backtrace),
    }
}

/// Construct the functional composition (`measurement1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.
///
/// # Arguments
/// * `measurement1` - outer measurement/mechanism
/// * `transformation0` - inner transformation
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DX` - Intermediate Domain.
/// * `TO` - Output Type.
/// * `MI` - Input Metric.
/// * `MX` - Intermediate Metric.
/// * `MO` - Output Measure.
pub fn make_chain_mt<DI, DX, TO, MI, MX, MO>(
    measurement1: &Measurement<DX, TO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Measurement<DI, TO, MI, MO>>
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
    assert_components_match!(
        DomainMismatch,
        transformation0.output_domain,
        measurement1.input_domain
    );
    assert_components_match!(
        MetricMismatch,
        transformation0.output_metric,
        measurement1.input_metric
    );

    Measurement::new(
        transformation0.input_domain.clone(),
        Function::make_chain(&measurement1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        measurement1.output_measure.clone(),
        PrivacyMap::make_chain(&measurement1.privacy_map, &transformation0.stability_map),
    )
}

/// Construct the functional composition (`transformation1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `transformation1(transformation0(x))`.
///
/// # Arguments
/// * `transformation1` - outer transformation
/// * `transformation0` - inner transformation
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DX` - Intermediate Domain.
/// * `DO` - Output Domain.
/// * `MI` - Input Metric.
/// * `MX` - Intermediate Metric.
/// * `MO` - Output Metric.
pub fn make_chain_tt<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Transformation<DI, DO, MI, MO>>
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
    assert_components_match!(
        DomainMismatch,
        transformation0.output_domain,
        transformation1.input_domain
    );

    assert_components_match!(
        MetricMismatch,
        transformation0.output_metric,
        transformation1.input_metric
    );

    Transformation::new(
        transformation0.input_domain.clone(),
        transformation1.output_domain.clone(),
        Function::make_chain(&transformation1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        transformation1.output_metric.clone(),
        StabilityMap::make_chain(
            &transformation1.stability_map,
            &transformation0.stability_map,
        ),
    )
}

/// Construct the functional composition (`postprocess1` ○ `measurement0`).
/// Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
/// Used to represent non-interactive postprocessing.
///
/// # Arguments
/// * `postprocess1` - outer postprocessing transformation
/// * `measurement0` - inner measurement/mechanism
///
/// # Generics
/// * `DI` - Input Domain.
/// * `TX` - Intermediate Type.
/// * `TO` - Output Type.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_chain_pm<DI, TX, TO, MI, MO>(
    postprocess1: &Function<TX, TO>,
    measurement0: &Measurement<DI, TX, MI, MO>,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    Measurement::new(
        measurement0.input_domain.clone(),
        Function::make_chain(postprocess1, &measurement0.function),
        measurement0.input_metric.clone(),
        measurement0.output_measure.clone(),
        measurement0.privacy_map.clone(),
    )
}

// UNIT TESTS
#[cfg(test)]
mod test;
