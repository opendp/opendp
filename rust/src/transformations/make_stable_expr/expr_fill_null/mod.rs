use polars::prelude::*;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Domain, ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, SeriesDomain};
use crate::error::*;
use crate::traits::Primitive;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `fill_null(constant)` expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The imputation expression
pub fn make_expr_fill_null<M: OuterMetric>(
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
    let Some((input, constant)) = match_fill_null(&expr)? else {
        return fallible!(MakeTransformation, "expected fill_null expression");
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let series_domain = output_domain.active_series_mut()?;
    series_domain.nullable = false;

    let dtype = series_domain.field.dtype.clone();

    use DataType::*;
    let constant = match &dtype {
        UInt32 => reconcile_domain::<u32>(constant, series_domain),
        UInt64 => reconcile_domain::<u64>(constant, series_domain),
        Int8 => reconcile_domain::<i8>(constant, series_domain),
        Int16 => reconcile_domain::<i16>(constant, series_domain),
        Int32 => reconcile_domain::<i32>(constant, series_domain),
        Int64 => reconcile_domain::<i64>(constant, series_domain),
        Float32 => reconcile_domain::<f32>(constant, series_domain),
        Float64 => reconcile_domain::<f64>(constant, series_domain),
        Boolean => reconcile_domain::<bool>(constant, series_domain),
        String => reconcile_domain::<str>(constant, series_domain),
        UInt8 | UInt16 => {
            fallible!(
                MakeTransformation,
                "u8 and 16 are not supported, please use u32 or u64 instead"
            )
        }
        dtype => fallible!(MakeTransformation, "unsupported dtype: {}", dtype),
    }?;

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| {
                let mut expr = expr.fill_null(constant.clone());

                // Update the super type of the fill_null function
                // This is necessary because it initializes to Unknown,
                // and Unknown dtype panics when serialized for FFI.
                let Expr::Function {
                    function: FunctionExpr::FillNull { super_type },
                    ..
                } = &mut expr
                else {
                    unreachable!();
                };
                *super_type = dtype.clone();
                expr
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}

pub fn match_fill_null(expr: &Expr) -> Fallible<Option<(&Expr, &LiteralValue)>> {
    let Expr::Function {
        input,
        function: FunctionExpr::FillNull { .. },
        ..
    } = expr
    else {
        return Ok(None);
    };

    let Ok([input, constant]) = <&[_; 2]>::try_from(input.as_slice()) else {
        return fallible!(MakeTransformation, "fill_null expects 2 arguments");
    };

    let Expr::Literal(constant) = constant else {
        return fallible!(
            MakeTransformation,
            "fill_null expects a literal as second argument"
        );
    };
    Ok(Some((input, constant)))
}

fn reconcile_domain<T: TryGetValue + ?Sized>(
    constant: &LiteralValue,
    series_domain: &SeriesDomain,
) -> Fallible<Expr>
where
    T::Owned: Primitive + Literal,
{
    let constant = constant
        .to_any_value()
        .ok_or_else(|| err!(MakeTransformation))?
        .strict_cast(&series_domain.field.dtype)?;
    let constant = T::try_get_value(constant)?;

    if !series_domain.atom_domain::<T::Owned>()?.member(&constant)? {
        return fallible!(
            MakeTransformation,
            "constant {:?} is not in domain {:?}",
            constant,
            series_domain
        );
    }
    Ok(lit(constant))
}

trait TryGetValue: ToOwned {
    fn try_get_value(value: AnyValue) -> Fallible<Self::Owned>;
}

macro_rules! impl_try_get_value {
    ($ty:ty, $ident:ident) => {
        impl TryGetValue for $ty {
            fn try_get_value(value: AnyValue) -> Fallible<Self::Owned> {
                if let AnyValue::$ident(v) = value {
                    Ok(v.to_owned())
                } else {
                    fallible!(MakeTransformation, "expected {}", stringify!($ty))
                }
            }
        }
    };
}
impl_try_get_value!(u8, UInt8);
impl_try_get_value!(u16, UInt16);
impl_try_get_value!(u32, UInt32);
impl_try_get_value!(u64, UInt64);
impl_try_get_value!(i8, Int8);
impl_try_get_value!(i16, Int16);
impl_try_get_value!(i32, Int32);
impl_try_get_value!(i64, Int64);
impl_try_get_value!(f32, Float32);
impl_try_get_value!(f64, Float64);
impl_try_get_value!(bool, Boolean);
impl_try_get_value!(str, String);
