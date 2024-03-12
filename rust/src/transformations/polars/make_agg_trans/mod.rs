use crate::combinators::assert_components_match;
use num::Zero;
use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    ExprDomain, LazyFrameDomain, LazyGroupByContext, LazyGroupByDomain, OuterMetric,
};
use crate::error::*;
use crate::traits::TotalOrd;

#[bootstrap(ffi = false)]
pub fn make_agg_trans<T: AggTransformation>(
    input_domain: LazyGroupByDomain,
    input_metric: T::InputMetric,
    transformations: Vec<T>,
) -> Fallible<
    Transformation<
        LazyGroupByDomain,
        LazyFrameDomain,
        T::InputMetric,
        <T::OutputMetric as OuterMetric>::InnerMetric,
    >,
>
where
    T::OutputMetric: OuterMetric,
    <T::OutputMetric as Metric>::Distance: TotalOrd + Zero,
    (LazyGroupByDomain, T::InputMetric): MetricSpace,
    (
        LazyFrameDomain,
        <T::OutputMetric as OuterMetric>::InnerMetric,
    ): MetricSpace,
{
    // resolve transformations
    let expr_input_domain = ExprDomain::new(
        input_domain.lazy_frame_domain.clone(),
        LazyGroupByContext {
            columns: input_domain.grouping_columns.clone(),
        },
        None,
    );

    let transformations = (transformations.into_iter())
        .map(|t| t.fix(&expr_input_domain, &input_metric))
        .collect::<Fallible<Vec<_>>>()?;

    let functions: Vec<_> = (transformations.iter())
        .map(|t| t.function.clone())
        .collect();

    let function = Function::new_fallible(move |lazy_group: &LazyGroupBy| -> Fallible<LazyFrame> {
        let seed = Arc::new(lazy_group.clone());
        let exprs = (functions.iter())
            .map(|t| t.eval(&(seed.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        Ok(lazy_group.clone().agg(&exprs))
    });

    // output metric
    let output_metric = (transformations.first())
        .map(|t| &t.output_metric)
        .ok_or_else(|| err!(MakeTransformation, "transformation list cannot be empty"))?
        .clone();

    transformations.iter().try_for_each(|t| {
        Ok(assert_components_match!(
            MetricMismatch,
            t.output_metric,
            output_metric
        ))
    })?;

    // stability map
    let stability_maps: Vec<_> = (transformations.iter())
        .map(|t| t.stability_map.clone())
        .collect();

    let stability_map = StabilityMap::new_fallible(move |d_in| {
        (stability_maps.iter())
            .try_fold(<T::OutputMetric as Metric>::Distance::zero(), |acc, map| {
                acc.total_max(map.eval(d_in)?)
            })
    });

    Transformation::new(
        input_domain.clone(),
        input_domain.lazy_frame_domain.clone(),
        function,
        input_metric,
        output_metric.inner_metric(),
        stability_map,
    )
}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the select context.
pub trait AggTransformation: 'static {
    type InputMetric: 'static + Metric;
    type OutputMetric: 'static + Metric;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &Self::InputMetric,
    ) -> Fallible<
        Transformation<
            ExprDomain<LazyGroupByDomain>,
            ExprDomain<LazyGroupByDomain>,
            Self::InputMetric,
            Self::OutputMetric,
        >,
    >;
}

impl<MI: 'static + Metric, MO: 'static + Metric> AggTransformation
    for Transformation<ExprDomain<LazyGroupByDomain>, ExprDomain<LazyGroupByDomain>, MI, MO>
where
    (ExprDomain<LazyGroupByDomain>, MI): MetricSpace,
    (ExprDomain<LazyGroupByDomain>, MO): MetricSpace,
{
    type InputMetric = MI;
    type OutputMetric = MO;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &MI,
    ) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<MI: 'static + Metric, MO: 'static + Metric> AggTransformation
    for crate::core::PartialTransformation<
        ExprDomain<LazyGroupByDomain>,
        ExprDomain<LazyGroupByDomain>,
        MI,
        MO,
    >
where
    (ExprDomain<LazyGroupByDomain>, MI): MetricSpace,
    (ExprDomain<LazyGroupByDomain>, MO): MetricSpace,
{
    type InputMetric = MI;
    type OutputMetric = MO;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &MI,
    ) -> Fallible<
        Transformation<ExprDomain<LazyGroupByDomain>, ExprDomain<LazyGroupByDomain>, MI, MO>,
    > {
        self.fix(input_domain.clone(), input_metric.clone())
    }
}

#[cfg(test)]
mod test_make_agg_trans {
    use polars::prelude::*;

    use super::*;
    use crate::metrics::{Lp, SymmetricDistance};
    use crate::transformations::polars_test::get_select_test_data;
    use crate::transformations::{make_col, then_col};

    #[test]
    fn test_make_agg_trans_output_lazy_frame() -> Fallible<()> {
        let (expr_domain_base, lazy_frame) = get_select_test_data()?;
        let grouping_columns = vec!["A".to_string()];

        let expr_domain = ExprDomain::new(
            expr_domain_base.lazy_frame_domain.clone(),
            LazyGroupByContext {
                columns: grouping_columns.clone(),
            },
            None,
        );

        let lazy_g = (*lazy_frame).clone().group_by_stable([col("A")]);

        let agg_trans = make_agg_trans(
            LazyGroupByDomain {
                lazy_frame_domain: expr_domain.lazy_frame_domain.clone(),
                grouping_columns,
            },
            Lp(SymmetricDistance::default()),
            vec![make_col(
                expr_domain,
                Lp(SymmetricDistance::default()),
                "B".to_string(),
            )?],
        );

        let lf_res = agg_trans?.invoke(&lazy_g)?.collect()?;

        let lf_exp = (*lazy_frame)
            .clone()
            .group_by_stable([col("A")])
            .agg([col("B")])
            .collect()?;

        assert!(lf_exp.equals(&lf_res));

        Ok(())
    }

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_agg_trans_output_partial() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;
        let grouping_columns = vec!["A".to_string()];
        let lazy_gb_domain = LazyGroupByDomain {
            lazy_frame_domain: expr_domain.lazy_frame_domain,
            grouping_columns,
        };
        let lazy_g = (*lazy_frame).clone().group_by_stable([col("A")]);

        let space = (lazy_gb_domain, Lp(SymmetricDistance));

        // demonstrates how you can pass partial constructors
        let agg_trans = (space >> then_agg_trans(vec![then_col("B".to_string())]))?;

        let lf_res = agg_trans.invoke(&lazy_g)?.collect()?;
        let lf_exp = (*lazy_frame)
            .clone()
            .group_by_stable([col("A")])
            .agg([col("B")])
            .collect()?;

        assert!(lf_exp.equals(&lf_res));
        Ok(())
    }
}
