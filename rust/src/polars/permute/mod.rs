//! Uniform random permutations of Polars expression rows.
//!
//! Struct fields share a permutation. The sampler materializes the input and
//! an O(n) index buffer.

use std::sync::Arc;

use polars::prelude::*;
#[cfg(feature = "ffi")]
use polars_plan::dsl::FunctionExpr;
use polars_plan::prelude::FunctionOptions;
#[cfg(feature = "ffi")]
use pyo3_polars::derive::polars_expr;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::Fallible;
use crate::polars::{OpenDPPlugin, match_plugin};
use crate::traits::samplers::GeneratorOpenDP;

/// A permutation changes neither the field name, dtype, nor length.
fn permute_type_udf(fields: &[Field]) -> PolarsResult<Field> {
    let [data] = fields else {
        polars_bail!(InvalidOperation: "permute expects one input");
    };
    Ok(data.clone())
}

#[cfg(feature = "ffi")]
#[polars_expr(output_type_func=permute_type_udf)]
fn permute_plugin(inputs: &[Series]) -> PolarsResult<Series> {
    permute(inputs)
}

/// Native Rust expression implementation of the same trusted permutation.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PermutePlugin;

impl ColumnsUdf for PermutePlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, input: &mut [Column]) -> PolarsResult<Column> {
        let [data] = input else {
            polars_bail!(ComputeError: "permute expects one input");
        };
        Ok(permute(&[data.as_materialized_series().clone()])?.into_column())
    }
}

impl AnonymousColumnsUdf for PermutePlugin {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }

    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }

    fn get_field(&self, _: &Schema, fields: &[Field]) -> PolarsResult<Field> {
        permute_type_udf(fields)
    }
}

impl OpenDPPlugin for PermutePlugin {
    const NAME: &'static str = "permute_plugin";

    fn function_options() -> FunctionOptions {
        FunctionOptions::length_preserving()
    }
}

/// Fisher-Yates permutation using the configured OpenDP random generator.
fn permute(inputs: &[Series]) -> PolarsResult<Series> {
    let [data] = inputs else {
        polars_bail!(ComputeError: "permute expects one input");
    };
    let n = data.len();
    if n <= 1 {
        return Ok(data.clone());
    }
    let end = IdxSize::try_from(n)
        .map_err(|_| polars_err!(ComputeError: "permute input exceeds Polars index range"))?;
    let mut indices: Vec<IdxSize> = (0..end).collect();
    let mut rng = GeneratorOpenDP::new();
    for i in 0..(n - 1) {
        let j = rng.gen_range(i, n);
        indices.swap(i, j);
    }
    rng.error
        .map_err(|e| polars_err!(ComputeError: "permute entropy error: {}", e))?;
    data.take(&IdxCa::from_vec("index".into(), indices))
}

/// Recognize only the full, length-preserving OpenDP permutation expression.
///
/// Both Python's FFI plugin and Rust's native anonymous UDF are trusted.
pub(crate) fn match_permute(expr: &Expr) -> Fallible<Option<&Expr>> {
    let Some(input) = match_plugin::<PermutePlugin>(expr)? else {
        return Ok(None);
    };

    // The generic matcher establishes plugin identity. These additional
    // checks certify the exact length-preserving permutation contract.
    let is_trusted = match expr {
        #[cfg(feature = "ffi")]
        Expr::Function {
            function: FunctionExpr::FfiPlugin { kwargs, flags, .. },
            ..
        } => kwargs.is_empty() && flags == &PermutePlugin::function_options(),
        Expr::AnonymousFunction { options, .. } => options == &PermutePlugin::function_options(),
        _ => false,
    };
    if !is_trusted {
        return Ok(None);
    }

    let [data] = input.as_slice() else {
        return Ok(None);
    };
    Ok(Some(data))
}

#[cfg(test)]
mod test;
