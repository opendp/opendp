use std::fmt::Display;

use crate::core::PrivacyMap;
use crate::domains::{ArrayDomain, AtomDomain, ExprPlan, VectorDomain, WildExprDomain};
use crate::measurements::{TopKMeasure, make_noisy_max};
use crate::measures::{MaxDivergence, ZeroConcentratedDivergence};
use crate::metrics::{IntDistance, L0InfDistance, L01InfDistance, LInfDistance};
use crate::polars::{OpenDPPlugin, apply_plugin, literal_value_of, match_plugin};
use crate::traits::{CastInternalRational, InfCast, InfMul, Number};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};
use dashu::float::FBig;

use polars::datatypes::{
    DataType, Field, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type,
    PolarsDataType, UInt32Type, UInt64Type,
};
use polars::error::polars_bail;
#[cfg(feature = "ffi")]
use polars::error::polars_err;
use polars::error::{PolarsError, PolarsResult};
use polars::lazy::dsl::Expr;
use polars::prelude::{Column, CompatLevel, IntoColumn};
use polars::series::{IntoSeries, Series};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;
use polars_arrow::array::PrimitiveArray;
use polars_arrow::types::NativeType;
use polars_plan::dsl::{ColumnsUdf, GetOutput};
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use super::approximate_c_stability;

#[cfg(test)]
mod test;

/// Make a measurement that reports the index of the maximum value after adding noise.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the selection will be applied
/// * `global_scale` - (Re)scale the noise distribution
pub fn make_expr_noisy_max<MI: 'static + UnboundedMetric, MO: 'static + TopKMeasure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, MO, ExprPlan>>
where
    Expr: StableExpr<L01InfDistance<MI>, L0InfDistance<LInfDistance<f64>>>,
{
    let (input, negate, scale) = match_noisy_max(&expr)?
        .ok_or_else(|| err!(MakeMeasurement, "Expected {}", NoisyMaxPlugin::NAME))?;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "{} requires a scale parameter",
            NoisyMaxPlugin::NAME
        );
    }

    let scale = match scale {
        Some(scale) => scale,
        None => {
            let (l_0, l_inf) = approximate_c_stability(&t_prior)?;
            f64::inf_cast(l_0)?.inf_mul(&l_inf)?
        }
    };
    let global_scale = global_scale.unwrap_or(1.);

    if scale.is_nan() || scale.is_sign_negative() {
        return fallible!(
            MakeMeasurement,
            "{} scale must be a non-negative number",
            NoisyMaxPlugin::NAME
        );
    }
    if global_scale.is_nan() || global_scale.is_sign_negative() {
        return fallible!(
            MakeMeasurement,
            "global_scale ({}) must be a non-negative number",
            global_scale
        );
    }

    let scale = scale.inf_mul(&global_scale)?;

    if middle_domain.column.nullable {
        return fallible!(
            MakeMeasurement,
            "{} requires non-nullable input",
            NoisyMaxPlugin::NAME
        );
    }
    let array_domain = middle_domain.column.element_domain::<ArrayDomain>()?;

    use DataType::*;
    let privacy_map = match array_domain.element_domain.dtype() {
        UInt32 => rnm_privacy_map::<u32, _>(array_domain, scale, negate)?,
        UInt64 => rnm_privacy_map::<u64, _>(array_domain, scale, negate)?,
        Int8 => rnm_privacy_map::<i8, _>(array_domain, scale, negate)?,
        Int16 => rnm_privacy_map::<i16, _>(array_domain, scale, negate)?,
        Int32 => rnm_privacy_map::<i32, _>(array_domain, scale, negate)?,
        Int64 => rnm_privacy_map::<i64, _>(array_domain, scale, negate)?,
        Float32 => rnm_privacy_map::<f32, _>(array_domain, scale, negate)?,
        Float64 => rnm_privacy_map::<f64, _>(array_domain, scale, negate)?,
        _ => {
            return fallible!(
                MakeMeasurement,
                "{} requires numeric array input",
                NoisyMaxPlugin::NAME
            );
        }
    };

    let m_rnm = Measurement::<_, L0InfDistance<LInfDistance<f64>>, _, _>::new(
        middle_domain,
        middle_metric.clone(),
        MO::default(),
        Function::then_expr(move |input_expr| {
            apply_plugin(
                vec![input_expr],
                expr.clone(),
                NoisyMaxPlugin {
                    distribution: MO::DISTRIBUTION,
                    negate,
                    scale,
                },
            )
        }),
        privacy_map,
    )?;

    t_prior >> m_rnm
}

fn rnm_privacy_map<T: Number, MO: TopKMeasure>(
    array_domain: &ArrayDomain,
    scale: f64,
    negate: bool,
) -> Fallible<PrivacyMap<L0InfDistance<LInfDistance<f64>>, MO>>
where
    T: Number + InfCast<f64> + CastInternalRational,
    FBig: TryFrom<T> + TryFrom<f64>,
    f64: InfCast<T> + InfCast<IntDistance>,
{
    let atom_domain = array_domain
        .element_domain
        .as_any()
        .downcast_ref::<AtomDomain<T>>()
        // should be unreachable
        .ok_or_else(|| err!(MakeMeasurement, "failed to downcast domain"))?
        .clone();

    let meas = make_noisy_max(
        VectorDomain::new(atom_domain),
        LInfDistance::default(),
        MO::default(),
        scale,
        negate,
    )?;

    Ok(PrivacyMap::new_fallible(
        move |(l0, li): &(IntDistance, f64)| {
            let epsilon = meas.map(&T::inf_cast(*li)?)?;
            f64::inf_cast(*l0)?.inf_mul(&epsilon)
        },
    ))
}

/// Determine if the given expression is a noisy_max expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the noisy_max expression and the arguments to the mechanism
pub(crate) fn match_noisy_max(expr: &Expr) -> Fallible<Option<(&Expr, bool, Option<f64>)>> {
    let Some(input) = match_plugin::<NoisyMaxShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, negate, scale]) = <&[_; 3]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "{:?} expects three inputs",
            NoisyMaxShim::NAME
        );
    };

    let negate = literal_value_of::<bool>(negate)?.ok_or_else(|| {
        err!(
            MakeMeasurement,
            "Negate must be true or false, found \"{}\".",
            negate
        )
    })?;
    let scale = literal_value_of::<f64>(scale)?;

    Ok(Some((data, negate, scale)))
}

/// Arguments for the noisy_max expression
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NoisyMaxShim;
impl ColumnsUdf for NoisyMaxShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for NoisyMaxShim {
    const NAME: &'static str = "noisy_max";
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noisy_max_plugin_type_udf(fields)
        }))
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum TopKDistribution {
    Exponential,
    Gumbel,
}

impl Display for TopKDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopKDistribution::Exponential => write!(f, "Exponential"),
            TopKDistribution::Gumbel => write!(f, "Gumbel"),
        }
    }
}

/// Arguments for the Noisy Max expression
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NoisyMaxPlugin {
    /// The distribution to sample from.
    pub distribution: TopKDistribution,
    /// The scale of the noise.
    pub scale: f64,
    /// Minimize if true
    pub negate: bool,
}

impl OpenDPPlugin for NoisyMaxPlugin {
    const NAME: &'static str = "noisy_max_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noisy_max_plugin_type_udf(fields)
        }))
    }
}

// allow the NoisyMaxPlugin struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for NoisyMaxPlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Option<Column>> {
        noisy_max_udf(s, self.clone()).map(Some)
    }
}

/// Implementation of the noisy_max expression.
///
/// The Polars engine executes this function over chunks of data.
fn noisy_max_udf(inputs: &[Column], kwargs: NoisyMaxPlugin) -> PolarsResult<Column> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", NoisyMaxPlugin::NAME);
    };

    let NoisyMaxPlugin {
        distribution,
        negate,
        scale,
    } = kwargs;

    if scale.is_sign_negative() {
        polars_bail!(InvalidOperation: "{} scale ({}) must be non-negative", NoisyMaxPlugin::NAME, scale);
    }

    // PT stands for Polars Type
    fn rnm_impl<PT: 'static + PolarsDataType>(
        column: &Column,
        distribution: TopKDistribution,
        scale: f64,
        negate: bool,
    ) -> PolarsResult<Column>
    where
        // the physical (rust) dtype must be a number that can be converted into a rational
        PT::Physical<'static>: NativeType + Number + CastInternalRational,
        FBig: TryFrom<PT::Physical<'static>> + TryFrom<f64>,
        f64: InfCast<PT::Physical<'static>>,
    {
        let domain = VectorDomain::new(AtomDomain::<PT::Physical<'static>>::new_non_nan());
        let metric = LInfDistance::default();
        let function = match distribution {
            TopKDistribution::Exponential => {
                make_noisy_max(domain, metric, MaxDivergence, scale, negate)?
                    .function
                    .clone()
            }
            TopKDistribution::Gumbel => {
                make_noisy_max(domain, metric, ZeroConcentratedDivergence, scale, negate)?
                    .function
                    .clone()
            }
        };
        Ok(column
            .as_materialized_series()
            // unpack the series into a chunked array
            .array()?
            // apply noisy max to each row
            .try_apply_nonnull_values_generic::<UInt32Type, _, _, _>(move |v| {
                let arr = v
                    .as_any()
                    .downcast_ref::<PrimitiveArray<PT::Physical<'static>>>()
                    .ok_or_else(|| {
                        PolarsError::InvalidOperation("input dtype does not match".into())
                    })?;

                let scores = arr.values_iter().cloned().collect::<Vec<_>>();
                PolarsResult::Ok(function.eval(&scores)? as u32)
            })?
            // convert the resulting chunked array back to a series
            .into_series()
            .into_column())
    }

    use DataType::*;
    let Array(dtype, _) = series.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", series.dtype())
    };

    match dtype.as_ref() {
        UInt32 => rnm_impl::<UInt32Type>(series, distribution, scale, negate),
        UInt64 => rnm_impl::<UInt64Type>(series, distribution, scale, negate),
        Int8 => rnm_impl::<Int8Type>(series, distribution, scale, negate),
        Int16 => rnm_impl::<Int16Type>(series, distribution, scale, negate),
        Int32 => rnm_impl::<Int32Type>(series, distribution, scale, negate),
        Int64 => rnm_impl::<Int64Type>(series, distribution, scale, negate),
        Float32 => rnm_impl::<Float32Type>(series, distribution, scale, negate),
        Float64 => rnm_impl::<Float64Type>(series, distribution, scale, negate),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type found {}", dtype),
    }
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn noisy_max(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn noisy_max_plugin_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", NoisyMaxPlugin::NAME)
    };
    use DataType::*;
    let Array(dtype, n) = field.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", field.dtype())
    };

    if *n == 0 {
        polars_bail!(InvalidOperation: "Array must have a non-zero length");
    }

    if matches!(dtype.as_ref(), UInt8 | UInt16) {
        polars_bail!(
            InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
        );
    }
    if !matches!(
        dtype.as_ref(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            field.dtype()
        );
    }
    Ok(Field::new(field.name().clone(), UInt32))
}

// generate the FFI plugin for the noisy_max expression
#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func=noisy_max_plugin_type_udf)]
fn noisy_max_plugin(inputs: &[Series], kwargs: NoisyMaxPlugin) -> PolarsResult<Series> {
    let inputs: Vec<Column> = inputs.iter().cloned().map(|s| s.into_column()).collect();
    let out = noisy_max_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
