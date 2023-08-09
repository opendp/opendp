use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation, ExprFunction};
use crate::domains::{Bounds, NumericDataType, DynSeriesAtomDomain, ExprDomain, OuterMetric};
use crate::error::*;
use crate::traits::{CheckAtom, TotalOrd};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    generics(
        M(suppress), 
        TA(suppress)
    ),
    derived_types(TA = "$get_active_column_type(input_domain)")
)]
/// Make a Transformation that returns a `clip(bounds)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `bounds` - bounds to be applied in clamp operation.
pub fn make_clamp_expr<
    M,
    TA: 'static + Clone + TotalOrd + CheckAtom + NumericNative + NumericDataType,
>(
    input_domain: ExprDomain<M::LazyDomain>,
    input_metric: M,
    bounds: (TA, TA),
) -> Fallible<Transformation<ExprDomain<M::LazyDomain>, ExprDomain<M::LazyDomain>, M, M>>
where
    M: OuterMetric,
    M::Distance: 'static + Clone,
    (ExprDomain<M::LazyDomain>, M): MetricSpace,
{
    let mut output_domain = input_domain.clone();

    let active_column = input_domain.active_column()?;
    // retrieve active series
    let active_series = output_domain
        .lazy_frame_domain
        .try_column_mut(active_column.clone())?;

    // add bounds to atom domain
    let mut atom_domain = active_series.atom_domain()?.clone();
    atom_domain.bounds = Some(Bounds::new_closed(bounds)?);

    // update element domain in active series
    active_series.element_domain = Arc::new(atom_domain) as Arc<dyn DynSeriesAtomDomain>;

    // Margins on the active_column could be preserved but this functionality has not been implemented yet
    let margins = output_domain
        .lazy_frame_domain
        .margins
        .clone()
        .into_iter()
        .filter(|(s, _)| !s.contains(&*active_column))
        .collect();

    output_domain.lazy_frame_domain.margins = margins;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_expr(move |expr| expr.clip(AnyValue::from(bounds.0), AnyValue::from(bounds.1))),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

#[cfg(test)]
mod test_make_clamp {
    use crate::domains::{AtomDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain};
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_clamp_expr() -> Fallible<()> {
        let active_col = "A";

        let frame_domain =
            LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
                .with_counts(df!["A" => [1, 2, 5], "count" => [1u32, 1, 1]]?.lazy())?;

        let expr_domain = ExprDomain::new(
            frame_domain.clone(),
            LazyFrameContext::Select,
            Some(active_col.to_string())
        );

        let lazy_frame = Arc::new(df!("A" => &[1, 2, 5])?.lazy());

        let frame_domain_clip = LazyFrameDomain::new(vec![SeriesDomain::new(
            "A",
            AtomDomain::<i32>::new_closed((1, 3))?,
        )])?;

        let expr_domain_clip = ExprDomain::new(
            frame_domain_clip.clone(),
            LazyFrameContext::Select,
            Some(active_col.to_string())
        );

        let transformation = make_clamp_expr(expr_domain, SymmetricDistance, (1, 3))?;

        let expr_res = transformation.invoke(&(lazy_frame, col("A")))?.1;
        let expr_exp = col(active_col).clip(AnyValue::from(1), AnyValue::from(3));

        assert_eq!(expr_res, expr_exp);
        assert_eq!(transformation.output_domain, expr_domain_clip);

        Ok(())
    }
}
