use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{DatasetMetric, LazyFrameDomain, LazyGroupByDomain};
use crate::error::*;
use polars::prelude::*;

pub fn make_groupby<M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    grouping_columns: Vec<String>,
) -> Fallible<Transformation<LazyFrameDomain, LazyGroupByDomain, M, M>>
where
    M: DatasetMetric + 'static,
    (LazyFrameDomain, M): MetricSpace,
    (LazyGroupByDomain, M): MetricSpace,
{
    for col in &grouping_columns {
        let mut found = false;
        for (keys, _) in &input_domain.margins {
            if keys.contains::<str>(col.as_ref()) {
                found = true;
                break;
            }
        }
        if !found {
            return fallible!(MakeTransformation, "Margins must me known");
        }
    }

    let output_domain = LazyGroupByDomain {
        lazy_frame_domain: input_domain.clone(),
        grouping_columns: grouping_columns.clone(),
    };

    let function = Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyGroupBy> {
        let column_exprs: Vec<_> = grouping_columns.iter().map(|c| col(c.as_ref())).collect();
        Ok(lazy_frame.clone().groupby_stable(&column_exprs))
    });

    Transformation::new(
        input_domain,
        output_domain,
        function,
        input_metric.clone(),
        input_metric,
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
        .with_counts(df!["A" => [1, 2], "B" => [1.0, 2.0], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["C" => [8, 9, 10], "count" => [1, 1, 1]]?.lazy())?;

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 2.0, 2.0],
            "C" => &[8, 9, 10],)?
        .lazy();

        let grouping_columns = vec!["A".to_string(), "B".to_string()];

        let groupby_trans = make_groupby(lf_domain, SymmetricDistance::default(), grouping_columns);

        let lf_res = groupby_trans
            .unwrap_test()
            .invoke(&lazy_frame.clone())
            .unwrap_test()
            .agg([all()])
            .collect()
            .unwrap_test();

        let lf_exp = lazy_frame
            .groupby([col("A"), col("B")])
            .agg([all()])
            .collect()
            .unwrap_test();

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
            make_groupby(lf_domain, SymmetricDistance::default(), grouping_columns)
                .map(|v| v.input_domain.clone())
                .unwrap_err()
                .variant;

        let error_variant_exp = MakeTransformation;

        assert_eq!(error_variant_exp, error_variant_res);

        Ok(())
    }
}
