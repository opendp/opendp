use crate::{
    core::{Measurement, MetricSpace},
    domains::ExprDomain,
    error::Fallible,
    measures::MaxDivergence,
    traits::Float,
    transformations::{make_sum_expr, traits::UnboundedMetric, SumOuterMetric, Summand},
};
use opendp_derive::bootstrap;
use polars::lazy::dsl::Expr;

use super::{then_laplace_expr, LaplaceOuterMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *")),
    generics(MI(suppress), TI(suppress), QO(default = "float"))
)]
/// Polars operator to compute the private sum of a column in a LazyFrame or LazyGroupBy
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `MI` - Input Metric
/// * `TI` - Data type of the input data
/// * `QO` - Output data type of the scale and epsilon
pub fn make_private_sum_expr<MI, TI, QO>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    scale: QO,
) -> Fallible<Measurement<ExprDomain<MI::LazyDomain>, Expr, MI, MaxDivergence<QO>>>
where
    MI: SumOuterMetric<TI>,
    MI::InnerMetric: UnboundedMetric,
    MI::SumMetric: LaplaceOuterMetric<QO>,

    TI: Summand,
    QO: Float,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::SumMetric): MetricSpace,
{
    make_sum_expr(input_domain, input_metric)? >> then_laplace_expr(scale)
}

#[cfg(test)]
mod test_make_mean_expr {
    use super::*;
    use crate::{
        metrics::{InsertDeleteDistance, Lp},
        transformations::polars_test::{get_grouped_test_data, get_select_test_data},
    };
    use polars::prelude::*;

    #[test]
    fn test_mean_expr_select() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;
        let scale: f64 = 0.1;

        let meas = make_private_sum_expr::<_, f64, _>(expr_domain, InsertDeleteDistance, scale)?;
        let expr_meas = meas.invoke(&(lazy_frame.clone(), col("B")))?;

        let release = (*lazy_frame).clone().select([expr_meas]).collect()?;
        println!("{:?}", release);

        let epsilon = meas.map(&2)?;
        println!("sens: {:?}", epsilon);
        assert!(epsilon > 6.66);
        assert!(epsilon < 6.67);

        Ok(())
    }

    #[test]
    fn test_mean_expr_groupby() -> Fallible<()> {
        let (expr_domain, group_by) = get_grouped_test_data()?;
        let scale: f64 = 0.1;

        let meas =
            make_private_sum_expr::<_, f64, _>(expr_domain, Lp(InsertDeleteDistance), scale)?;
        let expr_meas = meas.invoke(&(group_by.clone(), col("B")))?;

        let release = (*group_by)
            .clone()
            .agg([expr_meas])
            .sort("A", Default::default())
            .collect()?;
        println!("{:?}", release);

        let epsilon = meas.map(&2)?;
        println!("epsilon: {:?}", epsilon);
        assert!(epsilon > 20.0);
        assert!(epsilon < 20.00001);

        Ok(())
    }
}
