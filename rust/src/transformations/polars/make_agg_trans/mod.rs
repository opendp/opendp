use num::Zero;
use polars::export::regex::internal::InputAt;
use polars::prelude::*;
use crate::combinators::assert_components_match;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, LazyFrameContext, LazyFrameDomain, LazyGroupByContext, LazyGroupByDomain};
use crate::error::*;
use crate::metrics::L1;
use crate::traits::TotalOrd;

pub fn make_agg_trans<T: AggTransformation>(
    input_domain: LazyGroupByDomain,
    input_metric:  T::InputMetric,
    transformations: Vec<T>,
) -> Fallible<Transformation<LazyGroupByDomain, LazyFrameDomain, T::InputMetric, T::OutputMetric>>
where
    <T::OutputMetric as Metric>::Distance: TotalOrd + Zero,
    (LazyFrameDomain, T::InputMetric): MetricSpace,
    (LazyFrameDomain, T::OutputMetric): MetricSpace,
{
    // resolve transformations
    let expr_input_domain = ExprDomain {
        lazy_frame_domain: input_domain.lazy_frame_domain.clone(),
        context: LazyGroupByContext {columns: input_domain.grouping_columns.clone()},
        active_column: None,
    };

    let transformations = (transformations.into_iter())
        .map(|t| t.fix(&expr_input_domain, &input_metric))
        .collect::<Fallible<Vec<_>>>()?;

    let function = Function::new_fallible(move |lazy_frame: &LazyGroupBy| -> Fallible<LazyFrame> {
        let exprs = transformations
            .iter()
            .map(|t| t.invoke(&(lazy_frame.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        Ok(lazy_frame.agg(&exprs))
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
        input_domain,
        input_domain.lazy_frame_domain.clone(),
        function,
        input_metric,
        output_metric,
        stability_map,
    )

}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the select context.
pub trait AggTransformation: 'static {
    type InputMetric: 'static + Metric;
    type OutputMetric: 'static + Metric;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByContext>,
        input_metric: &Self::InputMetric,
    ) -> Fallible<
        Transformation<
            ExprDomain<LazyGroupByContext>,
            ExprDomain<LazyGroupByContext>,
            Self::InputMetric,
            Self::OutputMetric,
        >,
    >;
}

impl<MI: 'static + Metric, MO: 'static + Metric> AggTransformation
for Transformation<ExprDomain<LazyGroupByContext>, ExprDomain<LazyGroupByContext>, MI, MO>
    where
        (ExprDomain<LazyGroupByContext>, MI): MetricSpace,
        (ExprDomain<LazyGroupByContext>, MO): MetricSpace,
{
    type InputMetric = MI;
    type OutputMetric = MO;
    fn fix(self, input_domain: &ExprDomain<LazyGroupByContext>, input_metric: &MI) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}


#[cfg(test)]
mod test_make_agg_trans {
    use polars::prelude::*;

    use crate::metrics::SymmetricDistance;
    use crate::transformations::make_col;
    use crate::transformations::polars::test::get_test_data;

    use super::*;

    #[test]
    fn test_make_agg_trans_output_lazy_frame() -> Fallible<()> {
        let (mut expr_domain, lazy_frame, lf_domain) = get_test_data()?;
        let grouping_columns = vec!["A".to_string()];
        let lazy_g = lazy_frame.groupby([col("A")]);

        let agg_trans = make_agg_trans(
            LazyGroupByDomain{lazy_frame_domain: lf_domain, grouping_columns},
            SymmetricDistance::default(),
            vec![make_col(expr_domain, SymmetricDistance::default(), "B".to_string()).unwrap_test()],
        );

        let lf_res = agg_trans
            .unwrap_test()
            .invoke(&lazy_g)
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
}
