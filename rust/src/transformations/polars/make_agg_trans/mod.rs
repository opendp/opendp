use num::Zero;
use polars::prelude::*;

use crate::core::{Function, Metric, MetricSpace, Transformation};
use crate::domains::{Context, ExprDomain, LazyFrameDomain};
use crate::error::*;
use crate::traits::TotalOrd;
use crate::transformations::make_context_trans;

pub fn make_agg_trans<MI, MO>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    transformations: Vec<Transformation<ExprDomain, ExprDomain, MI, MO>>,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, MI, MO>>
where
    MI: Metric + 'static,
    MO: Metric + 'static,
    MO::Distance: TotalOrd + Zero,
    (LazyFrameDomain, MI): MetricSpace,
    (LazyFrameDomain, MO): MetricSpace,
{
    let first_transformation = transformations
        .first()
        .ok_or_else(|| err!(MakeTransformation, "transformation list cannot be empty"))?;
    let context = first_transformation.input_domain.context.clone();

    let context_columns = match context {
        Context::Agg { ref columns } => columns.clone(),
        _ => vec![],
    };

    if context_columns.is_empty() {
        return fallible!(MakeTransformation, "grouping columns cannot be empty");
    }

    if transformations
        .iter()
        .any(|t| t.input_domain.context != context)
    {
        return fallible!(
            MakeTransformation,
            "grouping columns have to be the same for all expressions"
        );
    }

    let trans = transformations.clone();

    let function = Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyFrame> {
        let exprs = transformations
            .iter()
            .map(|t| t.invoke(&(lazy_frame.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        let column_exprs: Vec<_> = context_columns.iter().map(|c| col(c.as_ref())).collect();

        Ok(lazy_frame.clone().groupby_stable(column_exprs).agg(&exprs))
    });

    make_context_trans(input_domain, input_metric, trans, context, None, function)
}

#[cfg(test)]
mod test_make_agg_trans {
    use polars::prelude::*;

    use crate::domains::Context;
    use crate::error::ErrorVariant::{MakeTransformation};
    use crate::metrics::SymmetricDistance;
    use crate::transformations::make_col;
    use crate::transformations::polars::test::get_test_data;

    use super::*;

    #[test]
    fn test_make_agg_trans_output_lazy_frame() -> Fallible<()> {
        let (mut expr_domain, lazy_frame, lf_domain) = get_test_data()?;
        let grouping_columns = vec!["A".to_string()];

        expr_domain.context = Context::Agg {
            columns: grouping_columns,
        };

        let agg_trans = make_agg_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![make_col(expr_domain, SymmetricDistance::default(), "B").unwrap_test()],
        );

        let lf_res = agg_trans
            .unwrap_test()
            .invoke(&lazy_frame)
            .unwrap_test()
            .collect()
            .unwrap_test();

        let lf_exp = lazy_frame
            .groupby_stable([col("A")])
            .agg([col("B")])
            .collect()
            .unwrap_test();

        assert!(lf_exp.frame_equal(&lf_res));

        Ok(())
    }

    #[test]
    fn test_make_select_trans_domain_missmatch() -> Fallible<()> {
        let (mut expr_domain, _, lf_domain) = get_test_data()?;
        expr_domain.context = Context::Filter;

        let error_variant_res = make_agg_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![
                make_col(expr_domain.clone(), SymmetricDistance::default(), "B").unwrap_test(),
                make_col(expr_domain.clone(), SymmetricDistance::default(), "A").unwrap_test(),
            ],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .variant;
        let error_variant_exp = MakeTransformation;

        assert_eq!(error_variant_res, error_variant_exp);

        Ok(())
    }

    #[test]
    fn test_make_select_trans_empty_list() -> Fallible<()> {
        let (_, _, lf_domain) = get_test_data()?;
        let error_msg_res =
            make_agg_trans::<_, SymmetricDistance>(lf_domain, SymmetricDistance::default(), vec![])
                .map(|v| v.input_domain.clone())
                .unwrap_err()
                .message;
        let error_msg_exp = Some("transformation list cannot be empty".to_string());

        assert_eq!(error_msg_res, error_msg_exp);

        Ok(())
    }
}
