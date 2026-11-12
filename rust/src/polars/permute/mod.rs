//! Uniform random permutations of Polars expression rows.
//!
//! Struct fields share a permutation. The sampler materializes the input and
//! an O(n) index buffer.

use polars::prelude::*;
use polars_plan::prelude::FunctionOptions;
use pyo3_polars::derive::polars_expr;
use rand::Rng;

use crate::traits::samplers::GeneratorOpenDP;

/// A permutation changes neither the field name, dtype, nor length.
fn permute_type_udf(fields: &[Field]) -> PolarsResult<Field> {
    let [data] = fields else {
        polars_bail!(InvalidOperation: "permute expects one input");
    };
    Ok(data.clone())
}

#[polars_expr(output_type_func=permute_type_udf)]
fn permute_plugin(inputs: &[Series]) -> PolarsResult<Series> {
    permute(inputs)
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
pub(crate) fn match_permute(expr: &Expr) -> Option<&Expr> {
    let Expr::Function {
        input,
        function:
            FunctionExpr::FfiPlugin {
                lib,
                symbol,
                kwargs,
                flags,
            },
    } = expr
    else {
        return None;
    };
    if !lib.contains("opendp")
        || symbol.as_str() != "permute_plugin"
        || !kwargs.is_empty()
        || flags != &FunctionOptions::length_preserving()
    {
        return None;
    }
    let [data] = input.as_slice() else {
        return None;
    };
    Some(data)
}

#[cfg(test)]
mod test;
