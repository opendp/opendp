use polars::lazy::dsl::Expr;
use polars::prelude::*;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, LazyFrameDomain, SeriesDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    arguments(transformation(c_type = "AnyTransformation *", rust_type = b"null")),
    generics(M(suppress))
)]
/// Make a Transformation that filters a LazyFrame.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | `input_metric`         |
/// | ------------------------------- | ---------------------- |
/// | `LazyFrameDomain`               | `SymmetricDistance`    |
/// | `LazyFrameDomain`               | `InsertDeleteDistance` |
///
/// # Arguments
/// * `input_domain` - LazyFrameDomain.
/// * `input_metric` - The metric space under which neighboring LazyFrame frames are compared.
///
/// # Generics
/// * `T` - ExprPredicate.
pub fn make_filter<M: DatasetMetric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, M, M>>
where
    (LazyFrameDomain, M): MetricSpace,
    (ExprDomain<LazyFrameDomain>, M): MetricSpace,
{
    let predicate_domain = row_by_row_translate(expr.clone(), input_domain.clone())?;

    if predicate_domain.series_domains.len() != 1 {
        return fallible!(MakeTransformation, "predicates must be univariate");
    }

    if predicate_domain.series_domains[0].field.dtype != DataType::Boolean {
        return fallible!(MakeTransformation, "predicates must be boolean");
    }

    // margin data counts become upper bounds after filtering
    let mut output_domain = input_domain.clone();
    output_domain
        .margins
        .iter_mut()
        .for_each(|(_k, m)| m.upper_bound = true);

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |frame: &LazyFrame| -> Fallible<LazyFrame> {
            Ok(frame.clone().filter(expr.clone()))
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

fn row_by_row_translate(
    expr: Expr,
    mut input_domain: LazyFrameDomain,
) -> Fallible<LazyFrameDomain> {
    Ok(match expr {
        Expr::Alias(expr, name) => {
            let mut domain = row_by_row_translate(*expr, input_domain)?;
            if domain.series_domains.len() != 1 {
                return fallible!(
                    MakeTransformation,
                    "expected exactly one column under alias"
                );
            }
            domain.series_domains[0].field.name = (&*name).into();
            domain
        }
        Expr::Column(name) => {
            let column = (input_domain.column(name))
                .ok_or_else(|| err!(MakeTransformation, "column does not exist"))?;
            input_domain.series_domains = vec![column.clone()];
            input_domain
        }
        Expr::Columns(names) => {
            input_domain.series_domains = names
                .iter()
                .map(|name| {
                    (input_domain.column(name).cloned())
                        .ok_or_else(|| err!(MakeTransformation, "column does not exist"))
                })
                .collect::<Fallible<Vec<_>>>()?;
            input_domain
        }
        Expr::DtypeColumn(_) => {
            return fallible!(MakeTransformation, "DtypeColumn not implemented")
        }
        Expr::Literal(literal) => {
            let name = "literal";
            let series_domain = match literal {
                LiteralValue::Boolean(_) => SeriesDomain::new(name, AtomDomain::<bool>::default()),
                LiteralValue::String(_) => SeriesDomain::new(name, AtomDomain::<String>::default()),
                LiteralValue::UInt32(_) => SeriesDomain::new(name, AtomDomain::<u32>::default()),
                LiteralValue::UInt64(_) => SeriesDomain::new(name, AtomDomain::<u64>::default()),
                LiteralValue::Int32(_) => SeriesDomain::new(name, AtomDomain::<i32>::default()),
                LiteralValue::Int64(_) => SeriesDomain::new(name, AtomDomain::<i64>::default()),
                LiteralValue::Float32(v) => SeriesDomain::new(
                    name,
                    AtomDomain::<f32> {
                        bounds: None,
                        nullable: v.is_nan(),
                    },
                ),
                LiteralValue::Float64(v) => SeriesDomain::new(
                    name,
                    AtomDomain::<f64> {
                        bounds: None,
                        nullable: v.is_nan(),
                    },
                ),
                _ => return fallible!(MakeTransformation, "literal dtype not implemented"),
            };
            LazyFrameDomain::new(vec![series_domain])?
        }
        Expr::BinaryExpr { left, op, right } => {
            use polars::lazy::dsl::Operator::*;
            let left_domain = row_by_row_translate(*left, input_domain.clone())?;
            row_by_row_translate(*right, input_domain.clone())?;

            if left_domain.series_domains.len() != 1 {
                return fallible!(
                    MakeTransformation,
                    "binary Expr expected univariate arguments"
                );
            }
            // name gets pulled from the lhs
            let name = left_domain.series_domains[0].field.name.as_str();

            let output_series = match op {
                Eq | EqValidity | NotEq | NotEqValidity | Lt | LtEq | Gt | GtEq | And | Or | Xor => {
                    SeriesDomain::new(name, AtomDomain::<bool>::default())
                }
                _ => return fallible!(MakeTransformation, "binary op not supported")
                // polars::lazy::dsl::Operator::Plus => todo!(),
                // polars::lazy::dsl::Operator::Minus => todo!(),
                // polars::lazy::dsl::Operator::Multiply => todo!(),
                // polars::lazy::dsl::Operator::Divide => todo!(),
                // polars::lazy::dsl::Operator::TrueDivide => todo!(),
                // polars::lazy::dsl::Operator::FloorDivide => todo!(),
                // polars::lazy::dsl::Operator::Modulus => todo!(),
            };
            LazyFrameDomain::new(vec![output_series])?
        }
        Expr::Cast {
            expr,
            data_type,
            strict,
        } => {
            input_domain.series_domains.map(|sd| {cast_variant(sd,data_type,strict)?})
            input_domain.margins = null; 
            input_domain
        },
        // Expr::Sort { expr, options } => todo!(),
        // Expr::Take { expr, idx } => todo!(),
        // Expr::SortBy {
        //     expr,
        //     by,
        //     descending,
        // } => todo!(),
        // Expr::Agg(_) => todo!(),
        // Expr::Ternary {
        //     predicate,
        //     truthy,
        //     falsy,
        // } => todo!(),
        // Expr::Function {
        //     input,
        //     function,
        //     options,
        // } => todo!(),
        // Expr::Explode(_) => todo!(),
        // Expr::Filter { input, by } => todo!(),
        // Expr::Window {
        //     function,
        //     partition_by,
        //     order_by,
        //     options,
        // } => todo!(),
        // Expr::Wildcard => todo!(),
        // Expr::Slice {
        //     input,
        //     offset,
        //     length,
        // } => todo!(),
        // Expr::Exclude(_, _) => todo!(),
        // Expr::KeepName(_) => todo!(),
        // Expr::Count => todo!(),
        // Expr::Nth(_) => todo!(),
        // Expr::RenameAlias { function, expr } => todo!(),
        // Expr::AnonymousFunction {
        //     input,
        //     function,
        //     output_type,
        //     options,
        // } => todo!(),
        // Expr::Selector(_) => todo!(),
        _ => return fallible!(MakeTransformation, "expr is not implemented"),
    })
}

fn cast_variant(
	mut input_domain: SeriesDomain,
    data_type: DataType,
    strict: bool,
) -> Fallible<SeriesDomain> {

    // create a new element domain based on the target_dtype
    macro_rules! new_element_domain {
        ($ty:ty) => {
            Arc::new(AtomDomain::<$ty>::default())
        };
    }
	
    input_domain.element_domain = match data_type {
        DataType::Boolean => new_element_domain!(bool),
        DataType::UInt8 => new_element_domain!(u8),
        DataType::UInt16 => new_element_domain!(u16),
        DataType::UInt32 => new_element_domain!(u32),
        DataType::UInt64 => new_element_domain!(u64),
        DataType::Int8 => new_element_domain!(i8),
        DataType::Int16 => new_element_domain!(i16),
        DataType::Int32 => new_element_domain!(i32),
        DataType::Int64 => new_element_domain!(i64),
        DataType::Float32 => Arc::new(AtomDomain::<f32>::new_nullable()), 
        DataType::Float64 => Arc::new(AtomDomain::<f64>::new_nullable()),
        DataType::String => new_element_domain!(str),
        _ => return fallible!(MakeDomain, "unsupported type {}", data_type),
    };

    input_domain.field.dtype = data_type;
    if strict {
        input_domain.nullable = true;
    }
    Ok(input_domain)

}

#[cfg(test)]
#[cfg(feature = "partials")]
mod test_make_filter {
    use crate::metrics::InsertDeleteDistance;
    use crate::transformations::item;

    use super::*;

    use crate::transformations::polars_test::get_select_test_data;

    #[test]
    fn test_make_private_select_output_lazy_frame() -> Fallible<()> {
        let (mut expr_domain, lazy_frame) = get_select_test_data()?;
        expr_domain.active_column = None;

        let filter_trans = make_filter(
            expr_domain.lazy_frame_domain.clone(),
            InsertDeleteDistance::default(),
            col("B").gt(lit(2)),
        );

        let lf_count = filter_trans?.invoke(&lazy_frame)?.select([len()]);
        assert_eq!(item::<u32>(lf_count)?, 0);
        Ok(())
    }
}
