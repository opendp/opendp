use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, ExprMetric};
use crate::error::*;
use crate::metrics::{InsertDeleteDistance, SymmetricDistance};
use crate::traits::{ExactIntCast, Number};

#[bootstrap(ffi = false)]
/// Make a Transformation that returns a `cut(breaks, labels, left_closed, include_breaks)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `breaks` - A list of unique cut points.
/// * `labels` - Labels to assign to bins. If given, the length must be len(breaks) + 1.
/// * `left_closed` - Whether intervals should be \[) instead of the default of (]
/// * `include_breaks` - Include the the right endpoint of the bin each observation falls in. If True, the resulting column will be a Struct.
///
/// # Generics
/// * `M` - Type of metric.
///
pub fn make_cut<MI>(
    input_domain: ExprDomain<MI::Context>,
    input_metric: MI,
    breaks: Vec<f64>,
    labels: Option<Vec<String>>,
    left_closed: bool,
    include_breaks: bool,
) -> Fallible<Transformation<ExprDomain<MI::Context>, ExprDomain<MI::Context>, MI, MI>>
where
    MI: CutExprMetric + Sync + Send + 'static,
    MI::Distance: Number + ExactIntCast<i64> + Clone + 'static,
    (ExprDomain<MI::Context>, MI): MetricSpace,
{
    let output_domain = input_domain.clone();

    let num_partitions = breaks.len();

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_expr(move |expr| {
            expr.clone().cut(
                breaks.clone(),
                labels.clone(),
                left_closed.clone(),
                include_breaks.clone(),
            )
        }),
        input_metric.clone(),
        input_metric.clone(),
        StabilityMap::new_fallible(move |d_in: &MI::Distance| {
            Ok(MI::Distance::exact_int_cast(
                if <MI::Context as Context>::GROUPBY {
                    input_metric.max_changed_partitions(num_partitions, &d_in)?
                } else {
                    1
                },
            )?)
        }),
    )
}

pub trait CutExprMetric: ExprMetric {
    type Metric: ExprMetric<
        Context = Self::Context,
        InnerMetric = Self::InnerMetric,
        Distance = Self::Distance,
    >;
    fn max_changed_partitions(
        &self,
        num_partitions: usize,
        d_in: &Self::Distance,
    ) -> Fallible<usize>;
}

impl CutExprMetric for SymmetricDistance {
    type Metric = SymmetricDistance;
    fn max_changed_partitions(
        &self,
        num_partitions: usize,
        _d_in: &Self::Distance,
    ) -> Fallible<usize> {
        Ok(num_partitions)
    }
}

impl CutExprMetric for InsertDeleteDistance {
    type Metric = InsertDeleteDistance;
    fn max_changed_partitions(
        &self,
        _num_partitions: usize,
        d_in: &Self::Distance,
    ) -> Fallible<usize> {
        usize::exact_int_cast(d_in.clone())
    }
}

#[cfg(test)]
mod test_make_cut {
    use crate::domains::{AtomDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_cut() -> Fallible<()> {
        let selected_col = "A";

        let frame_domain =
            LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
                .with_counts(df!["A" => [1, 2, 5], "count" => [1u32, 1, 1]]?.lazy())?;

        let expr_domain = ExprDomain::new(
            frame_domain.clone(),
            LazyFrameContext::Select,
            Some(selected_col.to_string()),
            true,
        );

        let lazy_frame = df!("A" => &[1, 2, 5])?.lazy();

        let transformation = make_cut(
            expr_domain,
            SymmetricDistance,
            vec![2.0, 5.0],
            None,
            false,
            false,
        )?;

        let expr_res = transformation.invoke(&(lazy_frame.into(), col("A")))?.1;
        let expr_exp = col(selected_col).cut(vec![2.0, 5.0], None, false, false);

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }
}
