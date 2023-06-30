use polars::prelude::*;

use crate::{
    core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation},
    domains::{
        AtomDomain, ExprDomain, OuterMetric, PrimitiveDataType, VectorDomain,
    },
    error::Fallible,
    metrics::{InsertDeleteDistance, IntDistance, LInfDiffDistance, SymmetricDistance, L1},
    traits::{CheckAtom, ExactIntCast, InfCast, Number},
    transformations::{make_quantile_score_candidates, ARDatasetMetric, IntoFrac, ToVec}, measurements::DExpOuterMetric,
};

type Score = u64;

/// Makes a Transformation to scores how similar each candidate is to the given
/// `alpha`-quantile on the input dataset with Polars.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `candidates` - Potential quantile to score
/// * `alpha` - Quantile value in [0, 1]. Choose 0.5 for median
///
/// # Generics
/// * `MI` - Input Metric.
/// * `TIA` - Atomic Input Type. Type of elements in the candidates input vector.
/// * `A` - Alpha type. Can be a (numer, denom) tuple, or float.
pub fn make_quantile_score_candidates_expr<
    MI,
    TIA: CheckAtom + Number + PrimitiveDataType,
    A: Clone + IntoFrac,
>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    candidates: Vec<TIA>,
    alpha: A,
) -> Fallible<
    Transformation<ExprDomain<MI::LazyDomain>, ExprDomain<MI::LazyDomain>, MI, MI::ScoreMetric>,
>
where
    MI: DQuantileOuterMetric,
    MI::InnerMetric: 'static + ARDatasetMetric,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

    ChunkedArray<TIA::Polars>: ToVec<TIA>,
{
    let meas = make_quantile_score_candidates(
        VectorDomain::default(),
        input_metric.inner_metric(),
        candidates.clone(),
        alpha,
    )?;

    let function = meas.function.clone();

    Transformation::new(
        input_domain.clone(),
        input_domain.clone(),
        Function::new_expr(move |expr| {
            expr.apply(
                enclose!(function, move |s: Series| {
                    let vec = (s.unpack::<TIA::Polars>()?.clone())
                        .to_option_vec()?
                        .into_iter()
                        .map(|v| v.ok_or_else(|| err!(FailedFunction, "expected non-null data")))
                        .collect::<Fallible<Vec<TIA>>>()?;

                    let score_vec = function
                        .eval(&vec)?
                        .into_iter()
                        .map(Score::exact_int_cast)
                        .collect::<Fallible<Vec<Score>>>()?;
                    
                    Ok(Some(Series::from_vec(&s.name(), score_vec)))
                }),
                GetOutput::from_type(Score::dtype()),
            )
        }),
        input_metric,
        MI::ScoreMetric::default(),
        // TODO: prove that the same bound holds under grouping
        StabilityMap::new_fallible(move |d_in| Score::inf_cast(meas.map(d_in)?)),
    )
}

/// Requirements for a metric that can be used in a sum transformation.
pub trait DQuantileOuterMetric: OuterMetric<Distance = IntDistance> {
    type ScoreMetric: DExpOuterMetric<
        // the inner metric will always be the L_{\infty}-difference distance
        InnerMetric = LInfDiffDistance<Score>,
        // regardless the choice of metric, the distance type will always be QO
        Distance = Score,
        // the type of the domain will never change
        LazyDomain = Self::LazyDomain,
    >;
}

impl DQuantileOuterMetric for InsertDeleteDistance {
    type ScoreMetric = LInfDiffDistance<Score>;
}
impl DQuantileOuterMetric for L1<InsertDeleteDistance> {
    type ScoreMetric = L1<LInfDiffDistance<Score>>;
}
impl DQuantileOuterMetric for SymmetricDistance {
    type ScoreMetric = LInfDiffDistance<Score>;
}
impl DQuantileOuterMetric for L1<SymmetricDistance> {
    type ScoreMetric = L1<LInfDiffDistance<Score>>;
}

#[cfg(test)]
pub mod test_quantile_make_score_candidates {

    use super::*;
    use crate::error::Fallible;
    use crate::metrics::{InsertDeleteDistance, Lp};
    use crate::transformations::polars_test::{get_grouped_test_data, get_select_test_data};

    #[test]
    fn test_make_quantile_score_candidates_expr_select() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;

        // Get resulting sum (expression result)
        let candidates = vec![2.0, 4.0, 5.0];
        let trans = make_quantile_score_candidates_expr(
            expr_domain,
            InsertDeleteDistance,
            candidates,
            0.8,
        )?;
        let expr_trans = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = (*lazy_frame).clone().select([expr_trans]).collect()?;

        // Get expected scoring
        let cell: Vec<u64> = vec![4_000, 6_000, 6_000];
        let frame_expected = DataFrame::new(vec![Series::new("B", &cell)])?;

        assert_eq!(frame_actual, frame_expected);
        Ok(())
    }

    #[test]
    fn test_make_quantile_score_candidates_expr_grouppy() -> Fallible<()> {
        let (expr_domain, group_by) = get_grouped_test_data()?;

        // Get resulting scores (expression result)
        let candidates = vec![1.0, 2.0, 3.0];
        let trans = make_quantile_score_candidates_expr(
            expr_domain,
            Lp(InsertDeleteDistance),
            candidates,
            0.5,
        )?;
        let expr_trans = trans.invoke(&(group_by.clone(), col("B")))?.1;

        let frame_actual = (*group_by).clone()
            .agg([expr_trans])
            .sort("A", Default::default())
            .collect()?;

        // Get expected scoring
        let cell_1: Vec<u64> = vec![0, 5_000, 5_000];
        let cell_2: Vec<u64> = vec![5_000, 5_000, 10_000];
        let frame_expected = DataFrame::new(vec![
            Series::new("A", &[1, 2]),
            Series::new("B", &[Series::new("B", cell_1), Series::new("B", cell_2)]),
        ])?;

        assert_eq!(frame_actual, frame_expected);
        println!("{:?}", frame_actual);
        Ok(())
    }
}
