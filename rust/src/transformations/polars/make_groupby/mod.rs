use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{LazyFrameDomain, LazyGroupByDomain};
use crate::error::*;
use crate::metrics::{Lp, L1};
use crate::transformations::traits::UnboundedMetric;
use polars::export::ahash::HashSet;
use opendp_derive::bootstrap;
use polars::prelude::*;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(generics(M(suppress)))]
pub fn make_groupby_stable<M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    grouping_columns: Vec<String>,
) -> Fallible<Transformation<LazyFrameDomain, LazyGroupByDomain, M, L1<M>>>
where
    M: UnboundedMetric + 'static,
    (LazyFrameDomain, M): MetricSpace,
    (LazyGroupByDomain, L1<M>): MetricSpace,
{
    let keyset = (input_domain.margins.keys())
        .flat_map(|k| k.iter().cloned())
        .collect::<HashSet<String>>();

    if let Some(missing_margin) =
        (grouping_columns.iter()).find(|c| !keyset.contains::<str>(c.as_ref()))
    {
        return fallible!(
            MakeTransformation,
            "failed to find margin for {}",
            missing_margin
        );
    }

    let output_domain = LazyGroupByDomain {
        lazy_frame_domain: input_domain.clone(),
        grouping_columns: grouping_columns.clone(),
    };

    let column_exprs: Vec<_> = grouping_columns.iter().map(|c| col(c.as_ref())).collect();

    Transformation::new(
        input_domain,
        output_domain,
        Function::new(move |lazy_frame: &LazyFrame| {
            lazy_frame.clone().groupby_stable(&column_exprs)
        }),
        input_metric.clone(),
        Lp(input_metric),
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod test_make_groupby_trans {
    use crate::domains::{AtomDomain, SeriesDomain};
    use crate::error::ErrorVariant::MakeTransformation;
    use crate::error::*;
    use polars::prelude::*;

    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_make_groupby_output() -> Fallible<()> {
        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?
        .with_counts(df!["A" => [1, 2], "B" => [1.0, 2.0], "count" => [1u32, 2]]?.lazy())?
        .with_counts(df!["C" => [8, 9, 10], "count" => [1u32, 1, 1]]?.lazy())?;

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 2.0, 2.0],
            "C" => &[8, 9, 10],)?
        .lazy();

        let grouping_columns = vec!["A".to_string(), "B".to_string()];

        let groupby_trans =
            make_groupby_stable(lf_domain, SymmetricDistance::default(), grouping_columns);

        let lf_res = groupby_trans?
            .invoke(&lazy_frame.clone())?
            .agg([all()])
            .collect()?;

        let lf_exp = lazy_frame
            .groupby_stable([col("A"), col("B")])
            .agg([all()])
            .collect()?;

        assert!(lf_exp.frame_equal(&lf_res));

        Ok(())
    }

    #[test]
    fn test_make_groupby_output_no_margin() -> Fallible<()> {
        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?;

        let grouping_columns = vec!["A".to_string(), "B".to_string()];

        let error_variant_res =
            make_groupby_stable(lf_domain, SymmetricDistance::default(), grouping_columns)
                .map(|v| v.input_domain.clone())
                .unwrap_err()
                .variant;

        let error_variant_exp = MakeTransformation;

        assert_eq!(error_variant_exp, error_variant_res);

        Ok(())
    }
}
