use std::sync::Arc;

use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;
use std::collections::HashMap;
use std::sync::LazyLock;

use super::StableExpr;
use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{L01InfDistance, LpDistance};
use crate::transformations::traits::UnboundedMetric;

#[cfg(test)]
mod test;

// Constant hashmap of types to their allowed cast types.
// This is necessary to prevent downcasting.
static ALLOWED_TRANSFORMATIONS: LazyLock<HashMap<DataType, Vec<DataType>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        DataType::Int8,
        vec![
            DataType::Int8,
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
        ],
    );
    m.insert(
        DataType::Int16,
        vec![DataType::Int16, DataType::Int32, DataType::Int64],
    );
    m.insert(DataType::Int32, vec![DataType::Int32, DataType::Int64]);
    m.insert(DataType::Int64, vec![DataType::Int64]);
    m
});

/// Make a Transformation that casts an aggregation output to int 64.
/// Casting aggregations to i64 before noise is added can enable negative values.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `expr` - The input measurement to be cast
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_aggregation<MI, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, L01InfDistance<MI>, ExprDomain, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (WildExprDomain, L01InfDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
    Expr: StableExpr<L01InfDistance<MI>, LpDistance<P, f64>>,
{
    let Expr::Cast {
        expr: input,
        dtype: to_type,
        mut options,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected cast expression");
    };

    let to_type_dtype = to_type
        .as_literal()
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "make_cast_aggregation only supports literal dtype"
            )
        })?
        .clone();

    match &to_type_dtype {
        dtype if ALLOWED_TRANSFORMATIONS.contains_key(dtype) => {}
        _ => {
            return fallible!(
                MakeTransformation,
                "make_cast_aggregation cast expects target integer dtype, found {}",
                to_type_dtype
            );
        }
    }

    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;

    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let active_column = &mut output_domain.column;

    match &to_type_dtype {
        DataType::Int8 => active_column.set_element_domain(AtomDomain::<i8>::default()),
        DataType::Int16 => active_column.set_element_domain(AtomDomain::<i16>::default()),
        DataType::Int32 => active_column.set_element_domain(AtomDomain::<i32>::default()),
        DataType::Int64 => active_column.set_element_domain(AtomDomain::<i64>::default()),

        _ => unreachable!(),
    }

    let in_dtype = active_column.dtype().clone();
    if !ALLOWED_TRANSFORMATIONS
        .get(&in_dtype)
        .is_some_and(|targets| targets.contains(&to_type_dtype))
    {
        return fallible!(
            MakeTransformation,
            "cannot downcast from {} to {}",
            in_dtype,
            to_type_dtype
        );
    }
    println!(
        "Input type: {:?}, Allowed transformations: {:?}",
        in_dtype,
        ALLOWED_TRANSFORMATIONS.get(&in_dtype)
    );

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Cast {
                expr: Arc::new(expr),
                dtype: to_type.clone(),
                options,
            }),
            StabilityMap::new(Clone::clone),
        )?
}
