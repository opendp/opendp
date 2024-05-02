use polars::prelude::*;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Bounds, ExprDomain, NumericDataType, OuterMetric, SeriesDomain};
use crate::error::*;
use crate::traits::Number;
use crate::transformations::DatasetMetric;

use super::StableExpr;

/// Make a Transformation that returns a `clip(bounds)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_clip<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        input, function, ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected function expression");
    };

    let FunctionExpr::Clip { has_min, has_max } = function else {
        return fallible!(MakeTransformation, "expected clip function");
    };

    if !has_min || !has_max {
        return fallible!(MakeTransformation, "Clip must have min and max");
    }

    let [input, lower, upper] = <[Expr; 3]>::try_from(input)
        .map_err(|_| err!(MakeTransformation, "Clip expects 3 arguments"))?;

    let t_prior = input.make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let (lower, upper) = {
        let series_domain = output_domain.active_series_mut()?;

        use DataType::*;
        match &series_domain.field.dtype {
            UInt32 => extract_bounds::<u32>(lower, upper, series_domain)?,
            UInt64 => extract_bounds::<u64>(lower, upper, series_domain)?,
            Int8 => extract_bounds::<i8>(lower, upper, series_domain)?,
            Int16 => extract_bounds::<i16>(lower, upper, series_domain)?,
            Int32 => extract_bounds::<i32>(lower, upper, series_domain)?,
            Int64 => extract_bounds::<i64>(lower, upper, series_domain)?,
            Float32 => extract_bounds::<f32>(lower, upper, series_domain)?,
            Float64 => extract_bounds::<f64>(lower, upper, series_domain)?,
            UInt8 | UInt16 => {
                return fallible!(
                    MakeTransformation,
                    "u8 and 16 are not supported, please use u32 or u64 instead"
                )
            }
            dtype => return fallible!(MakeTransformation, "unsupported dtype: {}", dtype),
        }
    };

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| expr.clip(lower.clone(), upper.clone())),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}

fn extract_bound<T: NumericDataType>(bound: Expr) -> Fallible<T> {
    let Expr::Literal(value) = bound else {
        return fallible!(MakeTransformation, "bound must be a literal");
    };

    let value = value.to_any_value().ok_or_else(|| {
        err!(
            MakeTransformation,
            "bound must be a numeric dtype, found {:?}",
            value.get_datatype()
        )
    })?;

    value.try_extract().map_err(Into::into)
}

fn extract_bounds<T: Number + NumericDataType + Literal>(
    lower: Expr,
    upper: Expr,
    domain: &mut SeriesDomain,
) -> Fallible<(Expr, Expr)> {
    let bounds = (extract_bound::<T>(lower)?, extract_bound::<T>(upper)?);

    let mut atom_domain = domain.atom_domain::<T>()?.clone();
    atom_domain.bounds = Some(Bounds::new_closed(bounds)?);
    domain.element_domain = Arc::new(atom_domain);

    Ok((bounds.0.lit(), bounds.1.lit()))
}

#[cfg(test)]
mod test;
