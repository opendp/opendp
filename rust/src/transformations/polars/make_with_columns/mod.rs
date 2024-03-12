use crate::combinators::assert_components_match;
use crate::transformations::DatasetMetric;
use opendp_derive::bootstrap;
use polars::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "partials")]
use crate::core::PartialTransformation;
use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
use crate::error::*;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(transformations(rust_type = "Vec<AnyTransformationPtr>")),
    dependencies("$get_dependencies_iterable(transformations)"),
    generics(T(suppress))
)]
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
    input_metric: T::Metric,
    transformations: Vec<T>,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, T::Metric, T::Metric>>
where
    T::Metric: IsDatasetMetric,
    <T::Metric as Metric>::Distance: PartialEq + Clone,
    (LazyFrameDomain, T::Metric): MetricSpace,
{
    let expr_input_domain =
        ExprDomain::new(input_domain.clone(), LazyFrameContext::WithColumns, None);

    // resolve transformation
    let transformation = (transformations.into_iter())
        .map(|t| t.fix(&expr_input_domain, &input_metric))
        .collect::<Fallible<Vec<_>>>()?;

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
        let lazy_frame = Arc::new(lazy_frame.clone());
        let exprs = (functions.iter())
            .map(|t| t.eval(&(lazy_frame.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        Ok((*lazy_frame).clone().with_columns(&exprs))
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

    let stability_map =
        StabilityMap::new_fallible(move |d_in: &<T::Metric as Metric>::Distance| {
            let d_in = d_in.clone();
            (stability_maps.iter()).try_for_each(|map| {
                if d_in != map.eval(&d_in)? {
                    return fallible!(FailedMap, "stability maps must be 1-stable");
                }
                Ok(())
            })?;

            Ok(d_in)
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

pub trait IsDatasetMetric {
    fn is_dataset_metric(&self) -> Fallible<()>;
}

impl<M: DatasetMetric> IsDatasetMetric for M {
    fn is_dataset_metric(&self) -> Fallible<()> {
        Ok(())
    }
}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the `with_columns`` context.
pub trait WithColumnsTransformation: 'static {
    type Metric: 'static + Metric;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyFrameDomain>,
        input_metric: &Self::Metric,
    ) -> Fallible<
        Transformation<
            ExprDomain<LazyFrameDomain>,
            ExprDomain<LazyFrameDomain>,
            Self::Metric,
            Self::Metric,
        >,
    >;
}

impl<M: 'static + Metric> WithColumnsTransformation
    for Transformation<ExprDomain<LazyFrameDomain>, ExprDomain<LazyFrameDomain>, M, M>
where
    (ExprDomain<LazyFrameDomain>, M): MetricSpace,
{
    type Metric = M;
    fn fix(self, input_domain: &ExprDomain<LazyFrameDomain>, input_metric: &M) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<M: 'static + Metric> WithColumnsTransformation
    for PartialTransformation<ExprDomain<LazyFrameDomain>, ExprDomain<LazyFrameDomain>, M, M>
where
    (ExprDomain<LazyFrameDomain>, M): MetricSpace,
{
    type Metric = M;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyFrameDomain>,
        input_metric: &M,
    ) -> Fallible<Transformation<ExprDomain<LazyFrameDomain>, ExprDomain<LazyFrameDomain>, M, M>>
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
    use crate::transformations::polars_test::get_select_test_data;
    use crate::transformations::{make_col, then_col};

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_with_columns_output_lazy_frame() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;
        let space = (expr_domain.clone().lazy_frame_domain, SymmetricDistance);

        // demonstrates how you can pass partial constructors
        let with_columns = (space >> then_with_columns(vec![then_col("B".to_string())]))?;

        let lf_res = with_columns.invoke(&lazy_frame.clone())?.collect()?;
        let lf_exp = (*lazy_frame).clone().with_columns([col("B")]).collect()?;

        assert_eq!(lf_exp, lf_res);
        println!("{:?}", lf_res);
        println!("{:?}", with_columns.output_domain);
        assert!(with_columns.output_domain.member(&lf_res.lazy())?);

        Ok(())
    }

    #[test]
    fn test_make_with_columns_domain_mismatch() -> Fallible<()> {
        let (mut expr_domain, _) = get_select_test_data()?;
        expr_domain.active_column.take();
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

    // #[test]
    // fn test_make_with_columns_empty_list() -> Fallible<()> {
    //     // must be explicit about the generic type because the compiler can't infer it from the empty list
    //     let error_msg_res = make_with_columns::<Transformation<_, _, _, SymmetricDistance>>(
    //         get_select_test_data()?.0.lazy_frame_domain,
    //         SymmetricDistance,
    //         vec![],
    //     )
    //     .map(|v| v.input_domain.clone())
    //     .unwrap_err()
    //     .message;
    //     let error_msg_exp = Some("transformation list cannot be empty".to_string());

    //     assert_eq!(error_msg_res, error_msg_exp);

    //     Ok(())
    // }
}
