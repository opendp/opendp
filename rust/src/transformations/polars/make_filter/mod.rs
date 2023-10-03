use polars::prelude::*;

use crate::combinators::assert_components_match;
use crate::core::{Function, Metric, MetricSpace, Transformation};
use crate::domains::{ExprDomain, LazyDomain, LazyFrameContext, LazyFrameDomain};
use crate::error::*;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    arguments(transformation(c_type = "AnyTransformation *", rust_type = b"null")),
    generics(T(suppress))
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
pub fn make_filter<T: ExprPredicate<Domain = LazyFrameDomain>>(
    input_domain: LazyFrameDomain,
    input_metric: T::Metric,
    transformation: T,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, T::Metric, T::Metric>>
where
    (LazyFrameDomain, T::Metric): MetricSpace,
    (ExprDomain<LazyFrameDomain>, T::Metric): MetricSpace,
{
    let expr_domain = ExprDomain {
        lazy_frame_domain: input_domain.clone(),
        context: LazyFrameContext::Filter,
        active_column: None,
    };
    let transformation = transformation.fix(&expr_domain, &input_metric)?;

    if transformation.output_domain.active_series()?.field.dtype != DataType::Boolean {
        return fallible!(MakeTransformation, "predicates should evaluate to booleans");
    }

    let function = transformation.function.clone();

    // margin data is invalidated after filtering
    // TODO: consider only dropping counts, leave categories
    let mut output_domain = input_domain.clone();
    output_domain.margins.clear();

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |frame: &LazyFrame| -> Fallible<LazyFrame> {
            let frame = Arc::new(frame.clone());
            let expr = function.eval(&(frame.clone(), all()))?.1;

            let frame = Arc::try_unwrap(frame).map_err(|_| {
                err!(
                    FailedFunction,
                    "transformations are not allowed to have side-effects"
                )
            })?;
            Ok(frame.filter(expr))
        }),
        input_metric.clone(),
        input_metric,
        transformation.stability_map.clone(),
    )
}

/// Either a `Transformation` or a `PartialTransformation` that can be used in the select context.
pub trait ExprPredicate: 'static {
    type Domain: 'static + LazyDomain;
    type Metric: 'static + Metric;
    fn fix(
        self,
        input_domain: &ExprDomain<Self::Domain>,
        input_metric: &Self::Metric,
    ) -> Fallible<
        Transformation<
            ExprDomain<Self::Domain>,
            ExprDomain<Self::Domain>,
            Self::Metric,
            Self::Metric,
        >,
    >;
}

impl<D: 'static + LazyDomain, M: 'static + Metric> ExprPredicate
    for Transformation<ExprDomain<D>, ExprDomain<D>, M, M>
where
    (ExprDomain<D>, M): MetricSpace,
{
    type Domain = D;
    type Metric = M;
    fn fix(self, input_domain: &ExprDomain<D>, input_metric: &M) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<D: 'static + LazyDomain, M: 'static + Metric> ExprPredicate
    for crate::core::PartialTransformation<ExprDomain<D>, ExprDomain<D>, M, M>
where
    (ExprDomain<D>, M): MetricSpace,
{
    type Domain = D;
    type Metric = M;
    fn fix(
        self,
        input_domain: &ExprDomain<D>,
        input_metric: &M,
    ) -> Fallible<Transformation<ExprDomain<D>, ExprDomain<D>, M, M>> {
        crate::core::PartialTransformation::fix(self, input_domain.clone(), input_metric.clone())
    }
}

#[cfg(test)]
#[cfg(feature = "partials")]
mod test_make_filter {
    use crate::core::{ExprFunction, StabilityMap};
    use crate::metrics::InsertDeleteDistance;
    use crate::transformations::{item, make_col};

    use super::*;

    use crate::transformations::polars_test::get_select_test_data;

    #[test]
    fn test_make_private_select_output_lazy_frame() -> Fallible<()> {
        let (mut expr_domain, lazy_frame) = get_select_test_data()?;
        expr_domain.active_column = None;

        let col = make_col(
            expr_domain.clone(),
            InsertDeleteDistance::default(),
            "B".to_string(),
        )?;

        let mut output_domain = col.output_domain.clone();
        output_domain
            .lazy_frame_domain
            .try_column_mut(output_domain.active_column()?)?
            .field
            .dtype = DataType::Boolean;

        let predicate = Transformation::new(
            col.output_domain.clone(),
            output_domain,
            Function::new_expr(|_expr| lit(false)),
            col.output_metric.clone(),
            col.output_metric.clone(),
            StabilityMap::new_from_constant(1),
        )?;

        let filter_trans = make_filter::<Transformation<_, _, _, _>>(
            expr_domain.lazy_frame_domain.clone(),
            InsertDeleteDistance::default(),
            (col >> predicate)?,
        );

        let lf_count = filter_trans?.invoke(&lazy_frame)?.select([len()]);
        assert_eq!(item::<u32>(lf_count)?, 0);
        Ok(())
    }
}
