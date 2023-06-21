use crate::combinators::assert_components_match;
use num::Zero;
use opendp_derive::bootstrap;
use polars::prelude::*;

#[cfg(feature = "partials")]
use crate::core::PartialTransformation;
use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
use crate::error::*;
use crate::traits::TotalOrd;

#[bootstrap(ffi = false)]
/// Make a Transformation that applies list of transformations in the select context to a Lazy Frame.
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
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared.
/// * `transformations` - Vector of transformation to be applied in the select context.
///
/// # Generics
/// * `T` - SelectTransformation type for Transformation or PartialTransformation.
pub fn make_select_trans<T: SelectTransformation>(
    input_domain: LazyFrameDomain,
    input_metric: T::InputMetric,
    transformations: Vec<T>,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, T::InputMetric, T::OutputMetric>>
where
    <T::OutputMetric as Metric>::Distance: TotalOrd + Zero,
    (LazyFrameDomain, T::InputMetric): MetricSpace,
    (LazyFrameDomain, T::OutputMetric): MetricSpace,
{
    // resolve transformations
    let expr_input_domain = ExprDomain {
        lazy_frame_domain: input_domain.clone(),
        context: LazyFrameContext::Select,
        active_column: None,
    };
    let transformations = (transformations.into_iter())
        .map(|t| t.fix(&expr_input_domain, &input_metric))
        .collect::<Fallible<Vec<_>>>()?;

    // output domain
    let output_domain = LazyFrameDomain::new(
        (transformations.iter())
            .map(|t| t.output_domain.active_series().map(Clone::clone))
            .collect::<Fallible<Vec<SeriesDomain>>>()?,
    )?;

    // function
    let functions: Vec<_> = (transformations.iter())
        .map(|t| t.function.clone())
        .collect();

    let function = Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyFrame> {
        let exprs = (functions.iter())
            .map(|t| t.eval(&(lazy_frame.clone(), all())))
            .map(|res| Ok(res?.1))
            .collect::<Fallible<Vec<Expr>>>()?;

        Ok(lazy_frame.clone().select(&exprs))
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
        output_domain,
        function,
        input_metric,
        output_metric,
        stability_map,
    )
}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the select context.
pub trait SelectTransformation: 'static {
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

impl<MI: 'static + Metric, MO: 'static + Metric> SelectTransformation
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
impl<MI: 'static + Metric, MO: 'static + Metric> SelectTransformation
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
mod test_make_select_trans {
    use crate::error::ErrorVariant::DomainMismatch;
    use crate::metrics::SymmetricDistance;
    use crate::transformations::polars::test::get_test_data;
    use crate::transformations::{make_col, then_col};

    use super::*;

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_select_trans_output_lazy_frame() -> Fallible<()> {
        let (_, lazy_frame, lf_domain) = get_test_data()?;
        let space = (lf_domain, SymmetricDistance);

        // demonstrates how you can pass partial constructors
        let select_trans = (space >> then_select_trans(vec![then_col("B".to_string())]))?;

        let lf_res = select_trans.invoke(&lazy_frame)?.collect()?;
        let lf_exp = df!("B" => &[1.0, 1.0, 2.0])?;

        assert_eq!(lf_exp, lf_res);
        Ok(())
    }

    #[test]
    fn test_make_select_trans_domain_mismatch() -> Fallible<()> {
        let (mut expr_domain, _, lf_domain) = get_test_data()?;
        expr_domain.context = LazyFrameContext::Filter;

        let error_variant_res = make_select_trans(
            lf_domain,
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
    fn test_make_select_trans_empty_list() -> Fallible<()> {
        // must be explicit about the generic type because the compiler can't infer it from the empty list
        let error_msg_res = make_select_trans::<Transformation<_, _, _, SymmetricDistance>>(
            get_test_data()?.2,
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
