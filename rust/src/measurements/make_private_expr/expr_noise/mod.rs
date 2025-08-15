use crate::core::{Measure, MetricSpace, PrivacyMap};
use crate::domains::{
    AtomDomain, ExprDomain, ExprPlan, NumericDataType, OuterMetric, VectorDomain, WildExprDomain,
};
use crate::measurements::{
    DiscreteGaussian, DiscreteLaplace, MakeNoise, make_gaussian, make_laplace,
};
use crate::measures::ZeroConcentratedDivergence;
use crate::metrics::{L1Distance, L01InfDistance, L2Distance};
use crate::polars::{OpenDPPlugin, apply_plugin, literal_value_of, match_plugin};
use crate::traits::{InfMul, Number};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
    measures::MaxDivergence,
};
use dashu::rational::RBig;
use polars::prelude::{Column, IntoColumn, PolarsNumericType};

use polars_arrow::array::PrimitiveArray;
use serde::de::value::Error;

use polars::chunked_array::ChunkedArray;
use polars::datatypes::{ArrayFromIter, DataType, Field, PolarsDataType};
use polars::error::PolarsResult;
use polars::error::polars_bail;
use polars::lazy::dsl::Expr;
use polars::series::{IntoSeries, Series};
#[cfg(feature = "ffi")]
use polars::{datatypes::CompatLevel, error::polars_err};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;
use polars_plan::dsl::{ColumnsUdf, GetOutput};
use polars_plan::prelude::FunctionOptions;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};

use super::approximate_c_stability;

#[cfg(test)]
mod test;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct NoiseShim;
impl ColumnsUdf for NoiseShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for NoiseShim {
    const NAME: &'static str = "noise";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noise_plugin_type_udf(fields)
        }))
    }
}

/// Arguments for the noise expression
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct NoisePlugin {
    /// The distribution to sample from
    pub distribution: Distribution,

    /// The scale of the noise
    pub scale: f64,

    /// Distinguish between integer or floating-point support.
    pub support: Support,
}

// allow the NoiseArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for NoisePlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to NoiseArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Option<Column>> {
        noise_udf(s, self.clone()).map(Some)
    }
}

impl OpenDPPlugin for NoisePlugin {
    const NAME: &'static str = "noise_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noise_plugin_type_udf(fields)
        }))
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Distribution {
    Laplace,
    Gaussian,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Support {
    Integer,
    Float,
}

pub trait NoiseExprMeasure: 'static + Measure<Distance = f64> {
    type Metric: 'static + OuterMetric<Distance = f64>;
    const DISTRIBUTION: Distribution;
    type Dist;
    fn map_function<T: Number>(
        input_metric: &Self::Metric,
        scale: f64,
    ) -> Fallible<PrivacyMap<Self::Metric, Self>>
    where
        Self::Dist: MakeNoise<VectorDomain<AtomDomain<T>>, Self::Metric, Self>,
        (VectorDomain<AtomDomain<T>>, Self::Metric): MetricSpace;
}

impl NoiseExprMeasure for MaxDivergence {
    type Metric = L1Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Laplace;
    type Dist = DiscreteLaplace;
    fn map_function<T: Number>(
        input_metric: &Self::Metric,
        scale: f64,
    ) -> Fallible<PrivacyMap<Self::Metric, Self>>
    where
        Self::Dist: MakeNoise<VectorDomain<AtomDomain<T>>, Self::Metric, Self>,
        (VectorDomain<AtomDomain<T>>, Self::Metric): MetricSpace,
    {
        Ok(make_laplace(
            VectorDomain::new(AtomDomain::<T>::new_non_nan()),
            input_metric.clone(),
            scale,
            None,
        )?
        .privacy_map
        .clone())
    }
}
impl NoiseExprMeasure for ZeroConcentratedDivergence {
    type Metric = L2Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Gaussian;
    type Dist = DiscreteGaussian;
    fn map_function<T: Number>(
        input_metric: &Self::Metric,
        scale: f64,
    ) -> Fallible<PrivacyMap<Self::Metric, Self>>
    where
        Self::Dist: MakeNoise<VectorDomain<AtomDomain<T>>, Self::Metric, Self>,
        (VectorDomain<AtomDomain<T>>, Self::Metric): MetricSpace,
    {
        Ok(make_gaussian(
            VectorDomain::new(AtomDomain::<T>::new_non_nan()),
            input_metric.clone(),
            scale,
            None,
        )?
        .privacy_map
        .clone())
    }
}

/// Make a measurement that adds noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub fn make_expr_noise<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, L01InfDistance<MI>, MO>>
where
    Expr: StableExpr<L01InfDistance<MI>, MO::Metric>,
    (ExprDomain, MO::Metric): MetricSpace,
    // This is ugly, but necessary because MO is generic
    MO::Dist: MakeNoise<VectorDomain<AtomDomain<u32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<u64>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i8>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i16>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i64>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<f32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<f64>>, MO::Metric, MO>,
    (VectorDomain<AtomDomain<u32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<u64>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i8>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i16>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i64>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<f32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<f64>>, MO::Metric): MetricSpace,
{
    let Some((input, distribution, scale)) = match_noise_shim(&expr)? else {
        return fallible!(MakeMeasurement, "Expected noise function");
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "Noise mechanism requires either a scale to be set on the expression or a param to be passed to the constructor"
        );
    }

    let scale = match scale {
        Some(scale) => scale,
        None => approximate_c_stability(&t_prior)?,
    };
    let global_scale = global_scale.unwrap_or(1.);
    let scale = scale.inf_mul(&global_scale)?;
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "noise scale must not be negative");
    }

    if middle_domain.column.nullable {
        return fallible!(
            MakeMeasurement,
            "Noise mechanism requires non-nullable input"
        );
    }
    if let Some(distribution) = distribution {
        if MO::DISTRIBUTION != distribution {
            return fallible!(
                MakeMeasurement,
                "expected {:?} distribution, found {:?}",
                MO::DISTRIBUTION,
                distribution
            );
        }
    };

    let support = match middle_domain.column.dtype() {
        dt if dt.is_integer() => Support::Integer,
        dt if dt.is_float() => Support::Float,
        dt => {
            return fallible!(
                MakeMeasurement,
                "expected numeric data type, found {:?}",
                dt
            );
        }
    };

    use DataType::*;
    let privacy_map = match middle_domain.column.dtype() {
        UInt32 => MO::map_function::<u32>(&middle_metric, scale),
        UInt64 => MO::map_function::<u64>(&middle_metric, scale),
        Int8 => MO::map_function::<i8>(&middle_metric, scale),
        Int16 => MO::map_function::<i16>(&middle_metric, scale),
        Int32 => MO::map_function::<i32>(&middle_metric, scale),
        Int64 => MO::map_function::<i64>(&middle_metric, scale),
        Float32 => MO::map_function::<f32>(&middle_metric, scale),
        Float64 => MO::map_function::<f64>(&middle_metric, scale),
        dtype => {
            return fallible!(
                MakeMeasurement,
                "Expected numeric data type, found {}",
                dtype
            );
        }
    }?;

    let m_noise = Measurement::<_, _, MO::Metric, _>::new(
        middle_domain,
        Function::then_expr(move |input_expr| {
            apply_plugin(
                vec![input_expr],
                expr.clone(),
                NoisePlugin {
                    scale,
                    distribution: MO::DISTRIBUTION,
                    support,
                },
            )
        }),
        middle_metric,
        MO::default(),
        privacy_map,
    )?;

    t_prior >> m_noise
}

/// Determine if the given expression is a noise expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the Noise expression and optional scale of noise
pub(crate) fn match_noise_shim(
    expr: &Expr,
) -> Fallible<Option<(&Expr, Option<Distribution>, Option<f64>)>> {
    let Some(input) = match_plugin::<NoiseShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, distribution, scale]) = <&[_; 3]>::try_from(input.as_slice()) else {
        return fallible!(MakeMeasurement, "Noise expects three input expressions");
    };

    let distribution = if let Some(dist) = literal_value_of::<String>(distribution)? {
        let dist = Distribution::deserialize(dist.into_deserializer())
            .map_err(|e: Error| err!(FailedFunction, "{:?}", e))?;
        Some(dist)
    } else {
        None
    };

    let scale = literal_value_of::<f64>(scale)?;

    Ok(Some((data, distribution, scale)))
}

// Code comment, not documentation:
// When using the plugin API from other languages, the NoiseArgs struct is serialized inside a FunctionExpr::FfiPlugin.
// When using the Rust API directly, the NoiseArgs struct is stored inside an AnonymousFunction.

/// Implementation of the noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn noise_udf(inputs: &[Column], kwargs: NoisePlugin) -> PolarsResult<Column> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "noise expects a single input expression");
    };
    let series = series.as_materialized_series();

    let NoisePlugin {
        distribution,
        scale,
        ..
    } = kwargs;

    use DataType::*;
    match series.dtype() {
        UInt32 => noise_impl::<u32>(series, scale, distribution),
        UInt64 => noise_impl::<u64>(series, scale, distribution),
        Int8 => noise_impl::<i8>(series, scale, distribution),
        Int16 => noise_impl::<i16>(series, scale, distribution),
        Int32 => noise_impl::<i32>(series, scale, distribution),
        Int64 => noise_impl::<i64>(series, scale, distribution),
        Float32 => noise_impl::<f32>(series, scale, distribution),
        Float64 => noise_impl::<f64>(series, scale, distribution),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type, found {}", dtype),
    }.map(|s| s.into_column())
}

// PT stands for Polars Type
fn noise_impl<T: NumericDataType>(
    series: &Series,
    scale: f64,
    distribution: Distribution,
) -> PolarsResult<Series>
where
    T: Number,
    T::NumericPolars: PolarsNumericType,
    // must be able to construct the chunked array corresponding to the physical dtype from an iterator
    <T::NumericPolars as PolarsDataType>::Array: ArrayFromIter<T> + ArrayFromIter<Option<T>>,
    // must be able to convert the chunked array to a series
    ChunkedArray<T::NumericPolars>: IntoSeries,
    DiscreteLaplace: MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<f64>, MaxDivergence>,
    DiscreteGaussian:
        MakeNoise<VectorDomain<AtomDomain<T>>, L2Distance<f64>, ZeroConcentratedDivergence>,
    RBig: TryFrom<T>,
{
    let domain = VectorDomain::new(AtomDomain::<T>::new_non_nan());
    let function = match distribution {
        Distribution::Laplace => make_laplace(domain, L1Distance::default(), scale, None)?
            .function
            .clone(),
        Distribution::Gaussian => make_gaussian(domain, L2Distance::default(), scale, None)?
            .function
            .clone(),
    };
    let chunk_iter = series
        // .i32()?.to_vec_null_aware()
        // unpack the series into a chunked array
        .unpack::<T::NumericPolars>()?
        .downcast_iter()
        .map(|chunk| {
            function
                .eval(&chunk.values().to_vec())
                .map(PrimitiveArray::from_vec)
        });

    Ok(ChunkedArray::try_from_chunk_iter(series.name().clone(), chunk_iter)?.into_series())
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn noise(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn noise_plugin_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "noise expects a single input field")
    };
    use DataType::*;
    if matches!(field.dtype(), UInt8 | UInt16) {
        polars_bail!(
            InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
        );
    }
    if !matches!(
        field.dtype(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            field.dtype()
        );
    }
    Ok(field.clone())
}

// generate the FFI plugin for the noise expression
#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func=noise_plugin_type_udf)]
fn noise_plugin(inputs: &[Series], kwargs: NoisePlugin) -> PolarsResult<Series> {
    let inputs: Vec<Column> = inputs.iter().cloned().map(|s| s.into_column()).collect();
    let out = noise_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
