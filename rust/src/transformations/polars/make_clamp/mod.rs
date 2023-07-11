use num::One;
use polars::prelude::*;
use std::rc::Rc;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Bounds, Context, DataTypeFrom, DynSeriesAtomDomain, ExprDomain, ExprMetric};
use crate::error::*;
use crate::traits::{CheckAtom, DistanceConstant, TotalOrd};

/// Make a Transformation that returns a clip(<bounds>) expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `bounds` - bounds to be applied in clamp operation.
pub fn make_clamp_expr<
    M,
    C: Context,
    TA: 'static + Clone + TotalOrd + CheckAtom + NumericNative + DataTypeFrom,
>(
    input_domain: ExprDomain<C>,
    input_metric: M,
    bounds: (TA, TA),
) -> Fallible<Transformation<ExprDomain<C>, ExprDomain<C>, M, M>>
where
    M: ExprMetric<C>,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
    (ExprDomain<C>, M): MetricSpace,
    i32: From<TA>,
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
    active_series.element_domain = Rc::new(atom_domain) as Rc<dyn DynSeriesAtomDomain>;

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
        Function::new_fallible(
            move |(frame, expr): &(C::Value, Expr)| -> Fallible<(C::Value, Expr)> {
                Ok((
                    frame.clone(),
                    expr.clone()
                        .clip(AnyValue::from(bounds.0), AnyValue::from(bounds.1)),
                ))
            },
        ),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(M::Distance::one()),
    )
}

#[cfg(test)]
mod test_make_clamp {
    use crate::domains::{LazyFrameContext, LazyFrameDomain, SeriesDomain, AtomDomain};
    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let selected_col = "A";

        let frame_domain =
            LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
                .with_counts(df!["A" => [1, 2, 5], "count" => [1, 1, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain.clone(),
            context: LazyFrameContext::Select,
            active_column: Some(selected_col.to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 5],)?
        .lazy();

        let frame_domain_clip = LazyFrameDomain::new(vec![SeriesDomain::new(
            "A",
            AtomDomain::<i32>::new_closed((1, 3))?,
        )])?;

        let expr_domain_clip = ExprDomain {
            lazy_frame_domain: frame_domain_clip.clone(),
            context: LazyFrameContext::Select,
            active_column: Some(selected_col.to_string()),
        };

        let transformation = make_clamp_expr(expr_domain, SymmetricDistance, (1, 3))?;

        let expr_res = transformation.invoke(&(lazy_frame, col("A")))?.1;
        let expr_exp = col(selected_col).clip(AnyValue::from(1), AnyValue::from(3));

        assert_eq!(expr_res, expr_exp);
        assert_eq!(transformation.output_domain, expr_domain_clip);

        Ok(())
    }
}
