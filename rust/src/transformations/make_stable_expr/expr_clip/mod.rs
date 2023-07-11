use polars::prelude::*;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Bounds, ExprDomain, NumericDataType, SeriesDomain};
use crate::error::*;
use crate::traits::Number;

use super::{DatasetOuterMetric, StableExpr};

/// Make a Transformation that returns a `clip(bounds)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_clip<MS, MI>(
    input_domain: ExprDomain,
    input_metric: MS,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, MS, MI>>
where
    MS: 'static + DatasetOuterMetric,
    MI: 'static + DatasetOuterMetric,
    MI::Distance: Clone,
    (ExprDomain, MS): MetricSpace,
    (ExprDomain, MI): MetricSpace,
    Expr: StableExpr<MS, MI>,
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
    let (input_domain, input_metric) = t_prior.output_space();

    let mut output_domain = t_prior.output_domain.clone();
    let (lower, upper) = {
        let series_domain = output_domain.active_series_mut()?;

        match &series_domain.field.dtype {
            DataType::UInt32 => extract_bounds::<u32>(lower, upper, series_domain)?,
            DataType::UInt64 => extract_bounds::<u64>(lower, upper, series_domain)?,
            DataType::Int8 => extract_bounds::<i8>(lower, upper, series_domain)?,
            DataType::Int16 => extract_bounds::<i16>(lower, upper, series_domain)?,
            DataType::Int32 => extract_bounds::<i32>(lower, upper, series_domain)?,
            DataType::Int64 => extract_bounds::<i64>(lower, upper, series_domain)?,
            DataType::Float32 => extract_bounds::<f32>(lower, upper, series_domain)?,
            DataType::Float64 => extract_bounds::<f64>(lower, upper, series_domain)?,
            _ => return fallible!(MakeTransformation, "unsupported dtype"),
        }
    };

    let series_name = input_domain.active_series()?.field.name.as_str();

    // Margins on the active_column could be preserved but this functionality has not been implemented yet
    output_domain
        .frame_domain
        .margins
        .retain(|s, _| !s.contains(series_name));

    t_prior
        >> Transformation::new(
            input_domain.clone(),
            output_domain,
            Function::new_expr(move |expr| expr.clip(lower.clone(), upper.clone())),
            input_metric.clone(),
            input_metric,
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
mod test_make_clamp {
    use crate::domains::{AtomDomain, LogicalPlanDomain};
    use crate::metrics::SymmetricDistance;
    use crate::transformations::polars_test::get_test_data;

    use super::*;

    #[test]
    fn test_make_expr_clip() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let lp = lf.logical_plan;
        let expr_domain = lf_domain.clone().select();

        let expected = col("A").clip(lit(1), lit(3));

        let t_clip: Transformation<_, _, _, SymmetricDistance> = expected
            .clone()
            .make_stable(expr_domain, SymmetricDistance)?;
        let actual = t_clip.invoke(&(lp, all()))?.1;

        assert_eq!(expected, actual);

        let mut series_domain = lf_domain.series_domains[0].clone();
        series_domain.element_domain = Arc::new(AtomDomain::<i32>::new_closed((1, 3))?);

        let mut lf_domain_exp = LogicalPlanDomain::new(vec![series_domain])?;
        lf_domain_exp.margins = t_clip.output_domain.frame_domain.margins.clone();

        assert_eq!(t_clip.output_domain, lf_domain_exp.select());

        Ok(())
    }
}
