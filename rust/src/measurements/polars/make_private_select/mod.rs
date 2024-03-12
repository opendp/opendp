use polars::prelude::*;

use crate::core::{Function, Measurement, MetricSpace};
use crate::domains::{ExprDomain, LazyFrameContext, LazyFrameDomain};
use crate::error::*;
use opendp_derive::bootstrap;

use super::ExprMeasurement;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    arguments(measurement(c_type = "AnyMeasurement *", rust_type = b"null")),
    generics(T(suppress))
)]
/// Make a Transformation that returns a Measurement in select context.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | `input_metric`                             |
/// | ------------------------------- | ------------------------------------------ |
/// | `LazyFrameDomain`               | `SymmetricDistance`                        |
/// | `LazyFrameDomain`               | `InsertDeleteDistance`                     |
/// | `LazyFrameDomain`               | `ChangeOneDistance` if Margins provided    |
/// | `LazyFrameDomain`               | `HammingDistance` if Margins provided      |
/// | `LazyFrameDomain`               | `AbsoluteDistance`                         |
///
/// # Arguments
/// * `input_domain` - LazyFrameDomain.
/// * `input_metric` - The metric space under which neighboring LazyFrame frames are compared.
///
/// # Generics
/// * `T` - ExprMeasurement.
pub fn make_private_select<T: ExprMeasurement<InputDomain = LazyFrameDomain>>(
    input_domain: LazyFrameDomain,
    input_metric: T::InputMetric,
    measurement: T,
) -> Fallible<Measurement<LazyFrameDomain, LazyFrame, T::InputMetric, T::OutputMeasure>>
where
    (LazyFrameDomain, T::InputMetric): MetricSpace,
    (ExprDomain<LazyFrameDomain>, T::InputMetric): MetricSpace,
{
    let expr_domain = ExprDomain {
        lazy_frame_domain: input_domain.clone(),
        context: LazyFrameContext::Select,
        active_column: None,
    };
    let measurement = measurement.fix(&expr_domain, &input_metric)?;
    let function = measurement.function.clone();
    Measurement::new(
        input_domain,
        Function::new_fallible(move |frame: &LazyFrame| -> Fallible<LazyFrame> {
            let frame = Arc::new(frame.clone());
            let exprs = function.eval(&(frame.clone(), all()))?;

            let frame = Arc::try_unwrap(frame).map_err(|_| {
                err!(
                    FailedFunction,
                    "measurements are not allowed to have side-effects"
                )
            })?;
            Ok(frame.select(&exprs))
        }),
        input_metric,
        measurement.output_measure.clone(),
        measurement.privacy_map.clone(),
    )
}

#[cfg(test)]
#[cfg(feature = "partials")]
mod test_make_private_select {

    use crate::combinators::{make_basic_composition, then_basic_composition};
    use crate::measurements::{make_private_mean_expr, then_private_mean_expr};
    use crate::metrics::InsertDeleteDistance;
    use crate::transformations::{make_col, then_col};

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

        let scale: f64 = 1.0;
        let private_mean = make_private_mean_expr::<_, f64, _>(
            col.output_domain.clone(),
            col.output_metric.clone(),
            scale,
        )?;

        let select_meas = make_private_select::<Measurement<_, _, _, _>>(
            expr_domain.lazy_frame_domain.clone(),
            InsertDeleteDistance::default(),
            make_basic_composition(vec![(col >> private_mean)?])?,
        );

        select_meas?.invoke(&lazy_frame)?.collect()?;

        Ok(())
    }

    #[test]
    fn test_make_private_select_partials() -> Fallible<()> {
        let (mut expr_domain, lazy_frame) = get_select_test_data()?;
        expr_domain.active_column = None;

        let space = (
            expr_domain.lazy_frame_domain.clone(),
            InsertDeleteDistance::default(),
        );

        let select_meas = (space
            >> then_private_select(then_basic_composition(vec![
                then_col("B".to_string()) >> then_private_mean_expr::<_, f64, _>(1.0),
            ])))?;

        select_meas.invoke(&lazy_frame)?.collect()?;

        Ok(())
    }
}
