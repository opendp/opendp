use polars_plan::dsl::Expr;
use polars_plan::plans::LiteralValue;
use polars_plan::utils::expr_output_name;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, Null, OuterMetric, SeriesDomain, WildExprDomain};
use crate::error::Fallible;
use crate::transformations::DatasetMetric;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a literal.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `expr` - A literal expression.
pub fn make_expr_lit<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Literal(literal_value) = &expr else {
        return fallible!(MakeTransformation, "Expected literal expression");
    };

    let name = expr_output_name(&expr)?;

    macro_rules! series_domain {
        ($ty:ty, $null:expr) => {{
            SeriesDomain::new(name, AtomDomain::<$ty>::new(None, $null))
        }};
    }

    let series_domain = match literal_value.clone().materialize() {
        LiteralValue::Boolean(_) => series_domain!(bool, None),
        LiteralValue::String(_) => series_domain!(String, None),
        LiteralValue::UInt32(_) => series_domain!(u32, None),
        LiteralValue::UInt64(_) => series_domain!(u64, None),
        LiteralValue::Int8(_) => series_domain!(i8, None),
        LiteralValue::Int16(_) => series_domain!(i16, None),
        LiteralValue::Int32(_) => series_domain!(i32, None),
        LiteralValue::Int64(_) => series_domain!(i64, None),
        LiteralValue::Float32(v) => series_domain!(f32, v.is_nan().then(Null::new)),
        LiteralValue::Float64(v) => series_domain!(f64, v.is_nan().then(Null::new)),
        value => return fallible!(MakeTransformation, "unsupported literal value: {:?}", value),
    };

    let output_domain = ExprDomain {
        column: series_domain,
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(expr),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
