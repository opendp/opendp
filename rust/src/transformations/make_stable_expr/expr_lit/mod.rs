use polars::prelude::AnyValue;
use polars_plan::dsl::Expr;
use polars_plan::plans::LiteralValue;
use polars_plan::utils::expr_output_name;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, NaN, OuterMetric, SeriesDomain, WildExprDomain};
use crate::error::Fallible;
use crate::metrics::MicrodataMetric;

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
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M::InnerMetric: MicrodataMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Literal(literal_value) = &expr else {
        return fallible!(MakeTransformation, "Expected literal expression");
    };

    let name = expr_output_name(&expr)?;

    macro_rules! series_domain {
        ($ty:ty, $null:expr) => {{ SeriesDomain::new(name, AtomDomain::<$ty>::new(None, $null)) }};
    }

    let LiteralValue::Scalar(literal_value) = literal_value.clone().materialize() else {
        return fallible!(
            MakeTransformation,
            "unsupported literal value: {:?}",
            literal_value
        );
    };

    let series_domain = match literal_value.value() {
        AnyValue::Boolean(_) => series_domain!(bool, None),
        AnyValue::String(_) => series_domain!(String, None),
        AnyValue::StringOwned(_) => series_domain!(String, None),
        AnyValue::UInt32(_) => series_domain!(u32, None),
        AnyValue::UInt64(_) => series_domain!(u64, None),
        AnyValue::Int8(_) => series_domain!(i8, None),
        AnyValue::Int16(_) => series_domain!(i16, None),
        AnyValue::Int32(_) => series_domain!(i32, None),
        AnyValue::Int64(_) => series_domain!(i64, None),
        AnyValue::Float32(v) => series_domain!(f32, v.is_nan().then(NaN::new)),
        AnyValue::Float64(v) => series_domain!(f64, v.is_nan().then(NaN::new)),
        value => return fallible!(MakeTransformation, "unsupported literal value: {:?}", value),
    };

    let output_domain = ExprDomain {
        column: series_domain,
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::from_expr(expr),
        StabilityMap::new(Clone::clone),
    )
}
