use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::core::{Domain, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric};
use crate::error::*;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(generics(M(suppress)))]
/// Make a Transformation that returns a `col(column_name)` expression for a Lazy Frame.
///
/// | input_metric               | input_domain                     |
/// | -------------------------- | -------------------------------- |
/// | `SymmetricDistance`        | `ExprDomain<LazyFrameContext>`   |
/// | `InsertDeleteDistance`     | `ExprDomain<LazyFrameContext>`   |
/// | `L1<SymmetricDistance>`    | `ExprDomain<LazyGroupByDomain>` |
/// | `L1<InsertDeleteDistance>` | `ExprDomain<LazyGroupByDomain>` |
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
///
/// # Generics
/// * `M` - type of metric. see above table.
pub fn make_col<M>(
    input_domain: ExprDomain<M::LazyDomain>,
    input_metric: M,
    col_name: String,
) -> Fallible<Transformation<ExprDomain<M::LazyDomain>, ExprDomain<M::LazyDomain>, M, M>>
where
    M: OuterMetric,
    M::Distance: Clone + 'static,
    (ExprDomain<M::LazyDomain>, M): MetricSpace,
{
    if input_domain.active_column.is_some() {
        return fallible!(
            MakeTransformation,
            "make_col cannot be applied to an expression with an active column"
        );
    }

    let mut output_domain = input_domain.clone();
    output_domain.active_column = Some(col_name.clone());

    // ensure that column exists in the domain
    output_domain.active_series()?;

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(
            // in most other situations, we would use `Function::new_expr`, but we need to return a Fallible here
            move |(frame, expr): &(Arc<<M::LazyDomain as Domain>::Carrier>, Expr)| -> Fallible<(Arc<<M::LazyDomain as Domain>::Carrier>, Expr)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "make_col has to be the first expression in the expression chain"
                    );
                }
                Ok((frame.clone(), col(&*col_name)))
            },
        ),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::metrics::{Lp, SymmetricDistance};
    use crate::transformations::polars_test::{get_grouped_test_data, get_select_test_data};

    use super::*;

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let (mut expr_domain, lazy_frame) = get_select_test_data()?;
        let active_col = expr_domain.active_column.take().unwrap();
        let expr_exp = col(&active_col);

        let transformation = make_col(expr_domain, SymmetricDistance, active_col)?;
        let expr_res = transformation.invoke(&(lazy_frame, all()))?.1;

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_no_wildcard() -> Fallible<()> {
        let (mut expr_domain, lazy_frame) = get_select_test_data()?;
        let active_col = expr_domain.active_column.take().unwrap();

        let transformation = make_col(expr_domain, SymmetricDistance, active_col.clone())?;
        let error_res = transformation
            .invoke(&(lazy_frame, col(&active_col)))
            .map(|v| v.1)
            .unwrap_err()
            .variant;
        assert_eq!(error_res, ErrorVariant::FailedFunction);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_wrong_col() -> Fallible<()> {
        let (mut expr_domain, _) = get_select_test_data()?;
        expr_domain.active_column.take();
        let selected_col = "D";

        let error_res = make_col(expr_domain, SymmetricDistance, selected_col.to_string())
            .map(|v| v.input_domain.clone())
            .unwrap_err()
            .variant;
        assert_eq!(error_res, ErrorVariant::FailedFunction);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_domain() -> Fallible<()> {
        let (mut expr_domain, _) = get_grouped_test_data()?;
        let active_col = expr_domain.active_column.take().unwrap();

        let transformation = make_col(
            expr_domain.clone(),
            Lp(SymmetricDistance),
            active_col.clone(),
        )?;

        let expr_domain_res = transformation.output_domain.clone();

        let mut expr_domain_exp = expr_domain;
        expr_domain_exp.active_column = Some(active_col);

        assert_eq!(expr_domain_res, expr_domain_exp);

        Ok(())
    }
}
