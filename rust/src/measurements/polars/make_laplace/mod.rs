use crate::core::ExprFunction;
use crate::domains::NumericDataType;
use crate::measurements::{make_laplace, MakeLaplace};
use crate::metrics::{AbsoluteDistance, L1Distance};
use crate::traits::{Float, Number};
use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, ExprDomain, OuterMetric, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
};
use opendp_derive::bootstrap;
use polars::prelude::*;

#[bootstrap(ffi = false)]
/// Polars operator to make the Laplace noise measurement
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
pub fn make_laplace_expr<MI, QO: 'static>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    scale: QO,
) -> Fallible<Measurement<ExprDomain<MI::LazyDomain>, Expr, MI, MaxDivergence<QO>>>
where
    MI: LaplaceOuterMetric<QO>,
    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
{
    if input_domain.active_series()?.nullable {
        return fallible!(
            MakeTransformation,
            "Laplace mechanism requires non-nullable input"
        );
    }

    MI::make_laplace_expr(input_domain, input_metric, scale)
}

pub trait LaplaceOuterMetric<QO>: OuterMetric {
    fn make_laplace_expr(
        input_domain: ExprDomain<Self::LazyDomain>,
        input_metric: Self,
        scale: QO,
    ) -> Fallible<Measurement<ExprDomain<Self::LazyDomain>, Expr, Self, MaxDivergence<QO>>>;
}

impl<T: Number + NumericDataType, QO: Float> LaplaceOuterMetric<QO> for AbsoluteDistance<T>
where
    AtomDomain<T>: MakeLaplace<QO, InputMetric = Self, Carrier = T>,
    (AtomDomain<T>, Self): MetricSpace,

    Series: NamedFromOwned<Vec<T>>,
    (ExprDomain<Self::LazyDomain>, Self): MetricSpace,
{
    fn make_laplace_expr(
        input_domain: ExprDomain<Self::LazyDomain>,
        input_metric: Self,
        scale: QO,
    ) -> Fallible<Measurement<ExprDomain<Self::LazyDomain>, Expr, Self, MaxDivergence<QO>>> {
        let atom_domain = input_domain
            .active_series()?
            .atom_domain::<Self::Distance>()?
            .clone();

        let (_, function, input_metric, output_measure, privacy_map) =
            make_laplace(atom_domain, input_metric, scale)?.decompose();

        Measurement::new(
            input_domain,
            Function::new_expr(new_function(Arc::new(move |vals: &Vec<Self::Distance>| {
                vals.into_iter().map(|v| function.eval(v)).collect()
            }))),
            input_metric,
            output_measure,
            privacy_map,
        )
    }
}

impl<T: Number + NumericDataType, QO: Float> LaplaceOuterMetric<QO> for L1Distance<T>
where
    VectorDomain<AtomDomain<T>>: MakeLaplace<QO, InputMetric = Self, Carrier = Vec<T>>,
    (VectorDomain<AtomDomain<T>>, Self): MetricSpace,

    Series: NamedFromOwned<Vec<T>>,
    (ExprDomain<Self::LazyDomain>, Self): MetricSpace,
{
    fn make_laplace_expr(
        input_domain: ExprDomain<Self::LazyDomain>,
        input_metric: Self,
        scale: QO,
    ) -> Fallible<Measurement<ExprDomain<Self::LazyDomain>, Expr, Self, MaxDivergence<QO>>> {
        let atom_domain = input_domain
            .active_series()?
            .atom_domain::<Self::Distance>()?
            .clone();
        let vector_domain = VectorDomain::new(atom_domain.clone());

        let (_, function, input_metric, output_measure, privacy_map) =
            make_laplace(vector_domain, input_metric, scale)?.decompose();

        Measurement::new(
            input_domain,
            Function::new_expr(new_function(function.function.clone())),
            input_metric,
            output_measure,
            privacy_map,
        )
    }
}

fn new_function<T: NumericDataType>(
    function: Arc<dyn Fn(&Vec<T>) -> Fallible<Vec<T>> + Send + Sync + 'static>,
) -> impl Fn(Expr) -> Expr
where
    Series: NamedFromOwned<Vec<T>>,
{
    let closure = move |s: Series| {
        let vec: Vec<T> = s
            .unpack::<T::NumericPolars>()?
            .into_no_null_iter()
            .collect::<Vec<T>>();
        let noisy_vec =
            function(&vec).map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        Ok(Some(Series::from_vec(&s.name(), noisy_vec)))
    };

    move |expr| expr.map(closure.clone(), GetOutput::same_type())
}

#[cfg(test)]
mod test_make_laplace_expr {
    use super::*;

    use crate::metrics::{InsertDeleteDistance, L1Distance, Lp};
    use crate::transformations::polars_test::{get_grouped_test_data, get_select_test_data};
    use crate::transformations::then_sum_expr;
    use crate::{domains::VectorDomain, measurements::make_base_laplace};

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_laplace_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;
        let space = (expr_domain, InsertDeleteDistance);
        let scale: f64 = 1.0;

        let meas = (space >> then_sum_expr::<_, f64>() >> then_laplace_expr(scale))?;
        let meas_res = meas.invoke(&(lazy_frame.clone(), col("B")))?;

        let df_actual = (*lazy_frame).clone().select([meas_res]).collect()?;
        let df_exact = (*lazy_frame).clone().select([sum("B")]).collect()?;

        assert_ne!(df_actual, df_exact);
        Ok(())
    }

    #[test]
    fn test_make_laplace_grouped() -> Fallible<()> {
        let (expr_domain, lazy_groupby) = get_grouped_test_data()?;
        let scale: f64 = 1.0;

        let meas = make_laplace_expr(expr_domain, Lp(AbsoluteDistance::<f64>::default()), scale)?;
        let meas_res = meas.invoke(&(lazy_groupby.clone(), col("B").sum()))?;
        let series_res = (*lazy_groupby)
            .clone()
            .agg([meas_res])
            .collect()?
            .column("B")?
            .clone();

        let chain = make_base_laplace(VectorDomain::default(), L1Distance::default(), scale, None)?;
        let result = chain.invoke(&vec![1.0, 2.0])?;
        let series_exp = Series::new("B", result);

        assert_ne!(series_res, series_exp);
        Ok(())
    }
}
