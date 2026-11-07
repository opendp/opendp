use std::sync::Arc;

use crate::core::PrivacyMap;
use crate::domains::{ArrayDomain, AtomDomain, ExprPlan, VectorDomain, WildExprDomain};
use crate::measurements::{make_noisy_top_k, TopKMeasure};
use crate::metrics::{IntDistance, L01InfDistance, L0InfDistance, LInfDistance};
use crate::polars::{apply_plugin, literal_value_of, match_plugin, OpenDPPlugin};
use crate::traits::{CastInternalRational, InfCast, InfMul, Number};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};
use dashu::float::FBig;

use polars::datatypes::{
    DataType, Field, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
    PolarsDataType, UInt32Type, UInt64Type,
};
use polars::error::polars_bail;
use polars::error::{PolarsError, PolarsResult};
use polars::lazy::dsl::Expr;
use polars::prelude::{AnonymousColumnsUdf, Column, IntoColumn, ListChunked, NamedFrom, Series};
use polars::series::IntoSeries;
use polars_arrow::array::{Array, PrimitiveArray};
use polars_arrow::types::NativeType;
use polars_plan::dsl::ColumnsUdf;
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use super::approximate_c_stability;

#[cfg(test)]
mod test;

/// Make a measurement that reports the indices of the top-k values after adding noise.
///
/// The score array must have a fixed, public length. The result is a list of `k` indices,
/// so the candidate set itself is never inferred from private data.
pub(crate) fn make_expr_noisy_top_k<MI: 'static + UnboundedMetric, MO: 'static + TopKMeasure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, MO, ExprPlan>>
where
    Expr: StableExpr<L01InfDistance<MI>, L0InfDistance<LInfDistance<f64>>>,
{
    let (input, k, negate, scale) = match_noisy_top_k(&expr)?
        .ok_or_else(|| err!(MakeMeasurement, "Expected {}", NoisyTopKPlugin::NAME))?;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "{} requires a scale parameter",
            NoisyTopKPlugin::NAME
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
            NoisyTopKPlugin::NAME
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
            NoisyTopKPlugin::NAME
        );
    }
    let array_domain = middle_domain.column.element_domain::<ArrayDomain>()?;
    if k > array_domain.size {
        return fallible!(
            MakeMeasurement,
            "k ({}) must not exceed the number of candidates ({})",
            k,
            array_domain.size
        );
    }

    use DataType::*;
    let privacy_map = match array_domain.element_domain.dtype() {
        UInt32 => top_k_privacy_map::<u32, _>(array_domain, k, scale, negate)?,
        UInt64 => top_k_privacy_map::<u64, _>(array_domain, k, scale, negate)?,
        Int8 => top_k_privacy_map::<i8, _>(array_domain, k, scale, negate)?,
        Int16 => top_k_privacy_map::<i16, _>(array_domain, k, scale, negate)?,
        Int32 => top_k_privacy_map::<i32, _>(array_domain, k, scale, negate)?,
        Int64 => top_k_privacy_map::<i64, _>(array_domain, k, scale, negate)?,
        Float32 => top_k_privacy_map::<f32, _>(array_domain, k, scale, negate)?,
        Float64 => top_k_privacy_map::<f64, _>(array_domain, k, scale, negate)?,
        _ => {
            return fallible!(
                MakeMeasurement,
                "{} requires numeric array input",
                NoisyTopKPlugin::NAME
            );
        }
    };

    let m_top_k = Measurement::<_, L0InfDistance<LInfDistance<f64>>, _, _>::new(
        middle_domain,
        middle_metric.clone(),
        MO::default(),
        Function::then_expr(move |input_expr| {
            apply_plugin(
                vec![input_expr],
                expr.clone(),
                NoisyTopKPlugin {
                    replacement: MO::REPLACEMENT,
                    negate,
                    scale,
                    k,
                },
            )
        }),
        privacy_map,
    )?;

    t_prior >> m_top_k
}

fn top_k_privacy_map<T: Number, MO: TopKMeasure>(
    array_domain: &ArrayDomain,
    k: usize,
    scale: f64,
    negate: bool,
) -> Fallible<PrivacyMap<L0InfDistance<LInfDistance<f64>>, MO>>
where
    T: Number + InfCast<f64> + CastInternalRational,
    FBig: TryFrom<T> + TryFrom<f64>,
    f64: InfCast<T> + InfCast<usize> + InfCast<IntDistance>,
{
    let atom_domain = array_domain
        .element_domain
        .as_any()
        .downcast_ref::<AtomDomain<T>>()
        .ok_or_else(|| err!(MakeMeasurement, "failed to downcast domain"))?
        .clone();

    let meas = make_noisy_top_k(
        VectorDomain::new(atom_domain),
        LInfDistance::default(),
        MO::default(),
        k,
        scale,
        negate,
    )?;

    Ok(PrivacyMap::new_fallible(
        move |(l0, li): &(IntDistance, f64)| {
            let privacy_loss = meas.map(&T::inf_cast(*li)?)?;
            f64::inf_cast(*l0)?.inf_mul(&privacy_loss)
        },
    ))
}

pub(crate) fn match_noisy_top_k(
    expr: &Expr,
) -> Fallible<Option<(&Expr, usize, bool, Option<f64>)>> {
    let Some(input) = match_plugin::<NoisyTopKShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, k, negate, scale]) = <&[_; 4]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "{:?} expects four inputs",
            NoisyTopKShim::NAME
        );
    };

    let k = literal_value_of::<u64>(k)?
        .ok_or_else(|| err!(MakeMeasurement, "k must be a non-negative integer"))?
        .try_into()
        .map_err(|_| err!(MakeMeasurement, "k is too large"))?;
    if k == 0 {
        return fallible!(MakeMeasurement, "k must be greater than zero");
    }
    let negate = literal_value_of::<bool>(negate)?
        .ok_or_else(|| err!(MakeMeasurement, "negate must be true or false"))?;
    let scale = literal_value_of::<f64>(scale)?;

    Ok(Some((data, k, negate, scale)))
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NoisyTopKShim;

impl AnonymousColumnsUdf for NoisyTopKShim {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }
    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }
    fn get_field(&self, _: &polars::prelude::Schema, fields: &[Field]) -> PolarsResult<Field> {
        top_k_plugin_type_udf(fields)
    }
}

impl ColumnsUdf for NoisyTopKShim {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for NoisyTopKShim {
    const NAME: &'static str = "noisy_top_k";
    #[cfg(feature = "ffi")]
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NoisyTopKPlugin {
    pub replacement: bool,
    pub scale: f64,
    pub negate: bool,
    pub k: usize,
}

impl AnonymousColumnsUdf for NoisyTopKPlugin {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }
    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }
    fn get_field(&self, _: &polars::prelude::Schema, fields: &[Field]) -> PolarsResult<Field> {
        top_k_plugin_type_udf(fields)
    }
}

impl OpenDPPlugin for NoisyTopKPlugin {
    const NAME: &'static str = "noisy_top_k_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }
}

impl ColumnsUdf for NoisyTopKPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, inputs: &mut [Column]) -> PolarsResult<Column> {
        noisy_top_k_udf(inputs, self.clone())
    }
}

fn noisy_top_k_udf(inputs: &[Column], kwargs: NoisyTopKPlugin) -> PolarsResult<Column> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", NoisyTopKPlugin::NAME);
    };
    let NoisyTopKPlugin {
        replacement,
        negate,
        scale,
        k,
    } = kwargs;
    if !scale.is_finite() || scale.is_sign_negative() {
        polars_bail!(InvalidOperation: "{} scale ({}) must be finite and non-negative", NoisyTopKPlugin::NAME, scale);
    }

    let DataType::Array(dtype, width) = series.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", series.dtype());
    };
    if k == 0 || k > *width {
        polars_bail!(InvalidOperation: "k ({}) must be between 1 and the number of candidates ({})", k, width);
    }

    fn select_rows<PT: 'static + PolarsDataType>(
        series: &Column,
        k: usize,
        scale: f64,
        negate: bool,
        replacement: bool,
    ) -> PolarsResult<Column>
    where
        PT::Physical<'static>: NativeType + Number + CastInternalRational,
        FBig: TryFrom<PT::Physical<'static>> + TryFrom<f64>,
        f64: InfCast<PT::Physical<'static>> + InfCast<usize>,
    {
        let array = series.as_materialized_series().array()?;
        let width = array.width();
        let mut output_rows = Vec::with_capacity(array.len());

        for chunk in array.downcast_iter() {
            let values = chunk
                .values()
                .as_any()
                .downcast_ref::<PrimitiveArray<PT::Physical<'static>>>()
                .ok_or_else(|| {
                    PolarsError::InvalidOperation("input dtype does not match".into())
                })?;
            if values.null_count() > 0 {
                polars_bail!(InvalidOperation: "{} does not support null scores", NoisyTopKPlugin::NAME);
            }
            let flat = values.values_iter().cloned().collect::<Vec<_>>();
            for row in flat.chunks(width) {
                let selected = crate::measurements::noisy_top_k(row, scale, k, negate, replacement)
                    .map_err(|e| PolarsError::InvalidOperation(e.to_string().into()))?;
                output_rows.push(Some(Series::new(
                    "".into(),
                    selected.into_iter().map(|i| i as u32).collect::<Vec<_>>(),
                )));
            }
        }

        let mut output: ListChunked = if output_rows.is_empty() {
            let empty = Series::new("".into(), Vec::<u32>::new());
            ListChunked::from_iter([empty]).slice(0, 0)
        } else {
            output_rows.into_iter().collect()
        };
        output.rename(series.name().clone());
        Ok(output.into_series().into_column())
    }

    use DataType::*;
    match dtype.as_ref() {
        UInt32 => select_rows::<UInt32Type>(series, k, scale, negate, replacement),
        UInt64 => select_rows::<UInt64Type>(series, k, scale, negate, replacement),
        Int8 => select_rows::<Int8Type>(series, k, scale, negate, replacement),
        Int16 => select_rows::<Int16Type>(series, k, scale, negate, replacement),
        Int32 => select_rows::<Int32Type>(series, k, scale, negate, replacement),
        Int64 => select_rows::<Int64Type>(series, k, scale, negate, replacement),
        Float32 => select_rows::<Float32Type>(series, k, scale, negate, replacement),
        Float64 => select_rows::<Float64Type>(series, k, scale, negate, replacement),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type found {}", dtype),
    }
}

pub(crate) fn top_k_plugin_type_udf(fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(fields) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", NoisyTopKShim::NAME);
    };
    let DataType::Array(dtype, width) = field.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", field.dtype());
    };
    if *width == 0 {
        polars_bail!(InvalidOperation: "Array must have a non-zero length");
    }
    if !matches!(
        dtype.as_ref(),
        DataType::UInt32
            | DataType::UInt64
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Float32
            | DataType::Float64
    ) {
        polars_bail!(InvalidOperation: "Expected numeric array input, found {:?}", field.dtype());
    }
    Ok(Field::new(
        field.name().clone(),
        DataType::List(Box::new(DataType::UInt32)),
    ))
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func=top_k_plugin_type_udf)]
fn noisy_top_k_plugin(inputs: &[Series], kwargs: NoisyTopKPlugin) -> PolarsResult<Series> {
    let columns: Vec<Column> = inputs
        .iter()
        .cloned()
        .map(IntoColumn::into_column)
        .collect();
    let out = noisy_top_k_udf(&columns, kwargs)?;
    Ok(out.take_materialized_series())
}
