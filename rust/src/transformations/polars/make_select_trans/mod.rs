use crate::combinators::assert_components_match;
use num::Zero;
use polars::prelude::*;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
use crate::error::*;
use crate::traits::TotalOrd;

/// Make a Transformation that applies list of transformations in the select context to a Lazy Frame.
///
pub fn make_select_trans<MI, MO>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    transformations: Vec<
        Transformation<ExprDomain<LazyFrameContext>, ExprDomain<LazyFrameContext>, MI, MO>,
    >,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, MI, MO>>
where
    MI: Metric + 'static,
    MO: Metric + 'static,
    MO::Distance: TotalOrd + Zero,
    (LazyFrameDomain, MI): MetricSpace,
    (LazyFrameDomain, MO): MetricSpace,
{
    let expr_input_domain = ExprDomain {
        lazy_frame_domain: input_domain.clone(),
        context: LazyFrameContext::Select,
        active_column: None,
    };

    transformations.iter().try_for_each(|t| {
        Ok(assert_components_match!(
            DomainMismatch,
            t.input_domain,
            expr_input_domain
        ))
    })?;

    let output_metric = transformations
        .first()
        .map(|t| &t.output_metric)
        .ok_or_else(|| err!(MakeTransformation, "transformation list cannot be empty"))?
        .clone();

    if transformations
        .iter()
        .any(|t| t.output_metric != output_metric)
    {
        return fallible!(
            MakeTransformation,
            "transformations' output metrics do not match"
        );
    }

    let output_domain = LazyFrameDomain::new(
        transformations
            .iter()
            .map(|t| &t.output_domain)
            .map(|d| {
                d.lazy_frame_domain
                    .try_column(d.active_column()?)
                    .map(Clone::clone)
            })
            .collect::<Fallible<Vec<SeriesDomain>>>()?,
    )?;

    let stability_maps: Vec<_> = transformations
        .iter()
        .map(|t| &t.stability_map)
        .cloned()
        .collect();

    let stability_map = StabilityMap::new_fallible(move |d_in| {
        stability_maps
            .iter()
            .try_fold(MO::Distance::zero(), |acc, map| {
                acc.total_max(map.eval(d_in)?)
            })
    });

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyFrame> {
            let exprs = transformations
                .iter()
                .map(|t| t.invoke(&(lazy_frame.clone(), all())))
                .map(|res| Ok(res?.1))
                .collect::<Fallible<Vec<Expr>>>()?;

            Ok(lazy_frame.clone().select(&exprs))
        }),
        input_metric,
        output_metric.clone(),
        stability_map,
    )
}

#[cfg(test)]
mod test_make_select_trans {
    use polars::prelude::*;

    use crate::error::ErrorVariant::DomainMismatch;
    use crate::metrics::SymmetricDistance;
    use crate::transformations::make_col;
    use crate::transformations::polars::test::get_test_data;

    use super::*;

    #[test]
    fn test_make_select_trans_output_lazy_frame() -> Fallible<()> {
        let (expr_domain, lazy_frame, lf_domain) = get_test_data()?;
        let select_trans = make_select_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![
                make_col(expr_domain, Default::default(), "B".to_string())?,
            ],
        );

        let lf_res = select_trans
            .unwrap_test()
            .invoke(&lazy_frame)
            .unwrap_test()
            .collect()
            .unwrap_test();
        let lf_exp = df!(
            "B" => &[1.0, 1.0, 2.0],)
        .unwrap_test();

        assert!(lf_exp.frame_equal(&lf_res));

        Ok(())
    }

    #[test]
    fn test_make_select_trans_domain_missmatch() -> Fallible<()> {
        let (mut expr_domain, _, lf_domain) = get_test_data()?;
        expr_domain.context = LazyFrameContext::Filter;

        let error_variant_res = make_select_trans::<SymmetricDistance, SymmetricDistance>(
            lf_domain,
            SymmetricDistance::default(),
            vec![
                make_col::<SymmetricDistance, _>(
                    expr_domain.clone(),
                    SymmetricDistance::default(),
                    "B".to_string(),
                )?,
                make_col::<SymmetricDistance, _>(
                    expr_domain.clone(),
                    SymmetricDistance::default(),
                    "A".to_string(),
                )?,
            ],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .variant;
        let error_variant_exp = DomainMismatch;

        assert_eq!(error_variant_res, error_variant_exp);

        Ok(())
    }

    #[test]
    fn test_make_select_trans_empty_list() -> Fallible<()> {
        let (_, _, lf_domain) = get_test_data()?;
        let error_msg_res = make_select_trans::<_, SymmetricDistance>(
            lf_domain,
            SymmetricDistance::default(),
            vec![],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .message;
        let error_msg_exp = Some("transformation list cannot be empty".to_string());

        assert_eq!(error_msg_res, error_msg_exp);

        Ok(())
    }
}
