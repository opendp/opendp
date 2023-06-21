use crate::combinators::assert_components_match;
use num::Zero;
use opendp_derive::bootstrap;
use polars::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "partials")]
use crate::core::PartialTransformation;
use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{DatasetMetric, ExprDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
use crate::error::*;
use crate::traits::TotalOrd;

#[bootstrap(ffi = false)]
/// Make a Transformation that applies list of transformations in the `with_columns`` context to a Lazy Frame.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | `input_metric`                             |
/// | ------------------------------- | ------------------------------------------ |
/// | `LazyFrameDomain`               | `SymmetricDistance`                        |
/// | `LazyFrameDomain`               | `InsertDeleteDistance`                     |
/// | `LazyFrameDomain`               | `ChangeOneDistance` if Margins provided    |
/// | `LazyFrameDomain`               | `HammingDistance` if Margins provided      |
///
/// # Arguments
/// * `input_domain` - Domain of the Lazy Frame.
/// * `input_metric` - DatasetMetric under which neighboring LazyFrames are compared.
/// * `transformation` - Expression transformation to be applied in the `with_columns` context.
///
/// # Generics
/// * `T` - WithColumnsTransformation type for Transformation or PartialTransformation.
pub fn make_with_columns<T: WithColumnsTransformation>(
    input_domain: LazyFrameDomain,
    input_metric: T::InputMetric,
    transformation: Vec<T>,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, T::InputMetric, T::OutputMetric>>
where
    T::OutputMetric: DatasetMetric,
    (LazyFrameDomain, T::InputMetric): MetricSpace,
    (LazyFrameDomain, T::OutputMetric): MetricSpace,
{
    let expr_input_domain =
        ExprDomain::new(input_domain.clone(), LazyFrameContext::Select, None, true);

    // resolve transformation
    let transformation = (transformation.into_iter())
        .map(|t| t.fix(&expr_input_domain, &input_metric))
        .collect::<Fallible<Vec<_>>>()?;

    if transformation
        .iter()
        .any(|t| !t.clone().output_domain.aligned)
    {
        return fallible!(
            MakeTransformation,
            "make_with_columns can be applied to an aligned transformation only"
        );
    }

    // output domain
    let transformation_series = (transformation.iter())
        .map(|t| t.output_domain.active_series().map(Clone::clone))
        .collect::<Fallible<Vec<SeriesDomain>>>()?;

    // output domain
    let mut series_map: HashMap<String, SeriesDomain> = HashMap::new();

    // populate series_map with transformation_series
    for series in &transformation_series {
        series_map.insert(series.clone().field.name.to_string(), series.clone());
    }

    // replace old series with new ones
    let mut output_series = input_domain.series_domains.clone();
    for series in &mut output_series {
        if let Some(new_series) = series_map.get(series.field.name.as_str()) {
            *series = new_series.clone();
        }
    }

    let output_domain = LazyFrameDomain::new(output_series)?;

    // function
    let functions: Vec<_> = (transformation.iter())
        .map(|t| t.function.clone())
        .collect();

    let function = Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyFrame> {
        let exprs = (functions.iter())
            .map(|t| t.eval(&(lazy_frame.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        Ok(lazy_frame.clone().with_columns(&exprs))
    });

    // output metric
    let output_metric = (transformation.first())
        .map(|t| &t.output_metric)
        .ok_or_else(|| err!(MakeTransformation, "transformation list cannot be empty"))?
        .clone();

    transformation.iter().try_for_each(|t| {
        Ok(assert_components_match!(
            MetricMismatch,
            t.output_metric,
            output_metric
        ))
    })?;

    // stability map
    let stability_maps: Vec<_> = (transformation.iter())
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
        output_domain,
        function,
        input_metric,
        output_metric,
        stability_map,
    )
}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the `with_columns`` context.
pub trait WithColumnsTransformation: 'static {
    type InputMetric: 'static + Metric;
    type OutputMetric: 'static + Metric;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyFrameContext>,
        input_metric: &Self::InputMetric,
    ) -> Fallible<
        Transformation<
            ExprDomain<LazyFrameContext>,
            ExprDomain<LazyFrameContext>,
            Self::InputMetric,
            Self::OutputMetric,
        >,
    >;
}

impl<MI: 'static + Metric, MO: 'static + Metric> WithColumnsTransformation
    for Transformation<ExprDomain<LazyFrameContext>, ExprDomain<LazyFrameContext>, MI, MO>
where
    (ExprDomain<LazyFrameContext>, MI): MetricSpace,
    (ExprDomain<LazyFrameContext>, MO): MetricSpace,
{
    type InputMetric = MI;
    type OutputMetric = MO;
    fn fix(self, input_domain: &ExprDomain<LazyFrameContext>, input_metric: &MI) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<MI: 'static + Metric, MO: 'static + Metric> WithColumnsTransformation
    for PartialTransformation<ExprDomain<LazyFrameContext>, ExprDomain<LazyFrameContext>, MI, MO>
where
    (ExprDomain<LazyFrameContext>, MI): MetricSpace,
    (ExprDomain<LazyFrameContext>, MO): MetricSpace,
{
    type InputMetric = MI;
    type OutputMetric = MO;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyFrameContext>,
        input_metric: &MI,
    ) -> Fallible<Transformation<ExprDomain<LazyFrameContext>, ExprDomain<LazyFrameContext>, MI, MO>>
    {
        self.fix(input_domain.clone(), input_metric.clone())
    }
}

#[cfg(test)]
mod test_make_with_columns {
    use super::*;
    use crate::core::Domain;
    use crate::error::ErrorVariant::DomainMismatch;
    use crate::metrics::SymmetricDistance;
    use crate::transformations::polars::test::get_test_data;
    use crate::transformations::{make_col, then_col};

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_with_columns_output_lazy_frame() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let space = (expr_domain.clone().lazy_frame_domain, SymmetricDistance);

        // demonstrates how you can pass partial constructors
        let with_columns = (space >> then_with_columns(vec![then_col("B".to_string())]))?;

        let lf_res = with_columns.invoke(&lazy_frame.clone())?.collect()?;
        let lf_exp = lazy_frame.clone().with_columns([col("B")]).collect()?;

        assert_eq!(lf_exp, lf_res);
        println!("{:?}", lf_res);
        println!("{:?}", with_columns.output_domain);
        assert!(with_columns.output_domain.member(&lf_res.lazy())?);

        Ok(())
    }

    #[test]
    fn test_make_with_columns_domain_mismatch() -> Fallible<()> {
        let (mut expr_domain, _) = get_test_data()?;
        expr_domain.context = LazyFrameContext::Filter;

        let error_variant_res = make_with_columns(
            expr_domain.lazy_frame_domain.clone(),
            SymmetricDistance,
            vec![make_col(expr_domain, SymmetricDistance, "B".to_string())?],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .variant;
        let error_variant_exp = DomainMismatch;

        assert_eq!(error_variant_res, error_variant_exp);

        Ok(())
    }

    #[test]
    fn test_make_with_columns_empty_list() -> Fallible<()> {
        // must be explicit about the generic type because the compiler can't infer it from the empty list
        let error_msg_res = make_with_columns::<Transformation<_, _, _, SymmetricDistance>>(
            get_test_data()?.0.lazy_frame_domain,
            SymmetricDistance,
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
