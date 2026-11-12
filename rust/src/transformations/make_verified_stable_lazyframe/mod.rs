//! Private bridge from supported Polars queries to the verified row kernel.
//! The Verus proofs cover the normalized IR, not Polars execution itself.

use std::collections::HashSet;

use crate::verified_query as verified;
mod frontend;
mod backend;
use frontend::normalize_plan;
use backend::render_plan;
use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{LazyFrameDomain, SeriesDomain},
    error::*,
    metrics::{Bounds, FrameDistance, SymmetricDistance},
};

/// Experimental private constructor. Unsupported plans fail closed.
pub(crate) fn make_verified_stable_lazyframe(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance<SymmetricDistance>,
    lazyframe: LazyFrame,
) -> Fallible<Transformation<
    LazyFrameDomain, FrameDistance<SymmetricDistance>,
    LazyFrameDomain, FrameDistance<SymmetricDistance>,
>> {
    let (plan, columns) =
        normalize_plan(&lazyframe.logical_plan, &input_domain.series_domains)?;
    let source_columns = input_domain.series_domains.iter()
        .map(|column| column.name.clone()).collect::<Vec<_>>();
    // Conservatively discard margins and grouped distance bounds.
    let output_domain = LazyFrameDomain::new(columns)?;
    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |input: &LazyFrame| {
            let (output, _) = render_plan(
                &plan, input.logical_plan.clone(), &source_columns,
            )?;
            Ok(LazyFrame::from(output)
                .with_optimizations(input.get_current_optimizations()))
        }),
        StabilityMap::new(|bounds: &Bounds| {
            Bounds(vec![bounds.get_bound(&HashSet::new())])
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::{AtomDomain, OptionDomain};
    use crate::core::Domain;

    fn domain() -> Fallible<LazyFrameDomain> {
        LazyFrameDomain::new(vec![
            SeriesDomain::new("x", OptionDomain::new(AtomDomain::<i64>::default())),
            SeriesDomain::new("lower", OptionDomain::new(AtomDomain::<i64>::default())),
            SeriesDomain::new("upper", OptionDomain::new(AtomDomain::<i64>::default())),
            SeriesDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    #[test]
    fn projects_and_filters_a_different_input() -> Fallible<()> {
        let first = df!(
            "x" => [Some(2i64), None],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), None],
        )?.lazy();
        let query = first.clone()
            .filter(col("flag").eq(lit(true)))
            .select([
                col("x").clip(col("lower"), col("upper")).alias("clipped"),
                col("flag"),
            ]);
        let trans = make_verified_stable_lazyframe(
            domain()?, FrameDistance(SymmetricDistance), query)?;
        let second = df!(
            "x" => [Some(-2i64), Some(8), None],
            "lower" => [0i64, 1, 0],
            "upper" => [5i64, 6, 5],
            "flag" => [Some(true), Some(true), None],
        )?.lazy();
        let actual = trans.invoke(&second)?.collect()?;
        let expected = second
            .filter(col("flag").eq(lit(true)))
            .select([
                col("x").clip(col("lower"), col("upper")).alias("clipped"),
                col("flag"),
            ]).collect()?;
        assert_eq!(actual, expected);
        assert_eq!(trans.output_domain.series_domains.len(), 2);
        assert_eq!(trans.output_domain.series_domains[0].name.as_str(), "clipped");
        assert_eq!(
            trans.map(&Bounds::from(2))?.get_bound(&HashSet::new()).per_group,
            Some(2),
        );
        Ok(())
    }

    #[test]
    fn aliases_survive_nested_projections() -> Fallible<()> {
        let source = df!(
            "x" => [Some(1i64), Some(2)],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), Some(false)],
        )?.lazy();
        let query = source.clone()
            .select([col("x").alias("renamed")])
            .select([col("renamed").alias("result")])
            .filter(col("result").gt(lit(1i64)));
        let trans = make_verified_stable_lazyframe(
            domain()?, FrameDistance(SymmetricDistance), query,
        )?;
        let other = df!(
            "x" => [Some(3i64), Some(1), None],
            "lower" => [0i64, 0, 0],
            "upper" => [5i64, 5, 5],
            "flag" => [Some(true), Some(true), Some(false)],
        )?.lazy();
        let actual = trans.invoke(&other)?.collect()?;
        let expected = other
            .select([col("x").alias("renamed")])
            .select([col("renamed").alias("result")])
            .filter(col("result").gt(lit(1i64)))
            .collect()?;
        assert_eq!(actual, expected);
        assert_eq!(trans.output_domain.series_domains[0].name.as_str(), "result");
        Ok(())
    }

    fn to_cells(frame: &DataFrame) -> Fallible<Vec<Vec<Option<verified::VValue>>>> {
        (0..frame.height())
            .map(|i| {
                frame.get_row(i)?.0.into_iter().map(|value| {
                    Ok(match value {
                        AnyValue::Null => None,
                        AnyValue::Int64(v) => Some(verified::VValue::Int(v)),
                        AnyValue::Boolean(v) => Some(verified::VValue::Bool(v)),
                        value => return fallible!(MakeTransformation,
                            "unsupported reference-cell type: {:?}", value),
                    })
                }).collect::<Fallible<Vec<_>>>()
            })
            .collect()
    }

    fn check_polars_semantics(
        source: DataFrame, query: LazyFrame, compare_original: bool,
    ) -> Fallible<()> {
        let input_domain = domain()?;
        assert!(input_domain.member(&source.clone().lazy())?);
        let (plan, expected_columns) =
            frontend::normalize_plan(&query.logical_plan, &input_domain.series_domains)?;
        let input_names = input_domain.series_domains.iter()
            .map(|column| column.name.clone()).collect::<Vec<_>>();
        let (lowered, _) = backend::render_plan(
            &plan, source.clone().lazy().logical_plan.clone(), &input_names,
        )?;
        let reference = verified::VTable {
            schema: crate::domains::verified_frame::kernel_schema(&input_domain.series_domains)?,
            rows: to_cells(&source)?,
        };
        let expected = verified::evaluate(&plan, &reference)
            .expect("well-typed reference evaluation");
        let actual = LazyFrame::from(lowered).collect()?;
        if compare_original {
            assert_eq!(actual, query.collect()?);
        }
        assert!(to_cells(&actual)? == expected, "backend disagrees with reference evaluator");
        assert_eq!(actual.width(), expected_columns.len());
        for (column, expected) in actual.get_columns().iter().zip(&expected_columns) {
            assert_eq!(column.name().as_str(), expected.name.as_str());
            assert_eq!(column.dtype(), &expected.dtype());
        }
        Ok(())
    }

    #[test]
    fn polars_agrees_with_row_semantics() -> Fallible<()> {
        let input = df!(
            "x" => [Some(0i64), Some(5), Some(8), None, Some(7), Some(5)],
            "lower" => [Some(2i64), None, Some(9), Some(0), Some(8), None],
            "upper" => [Some(10i64), Some(3), Some(6), Some(3), Some(3), None],
            "flag" => [Some(true), None, Some(false), Some(true), None, Some(true)],
        )?;
        let projection = |lf: LazyFrame| lf.select([
            col("x").clip(col("lower"), col("upper")).alias("clipped"),
            col("flag"),
        ]);
        check_polars_semantics(input.clone(), projection(input.clone().lazy()), false)?;
        check_polars_semantics(input.slice(0, 0), projection(input.slice(0, 0).lazy()), false)?;
        check_polars_semantics(input.clone(), input.clone().lazy()
            .select([
                col("x").alias("renamed"),
                col("flag").and(col("x").gt(lit(2i64))).alias("keep"),
            ])
            .filter(col("keep")), true)?;
        Ok(())
    }

    #[test]
    fn rejects_unmodeled_columns() -> Fallible<()> {
        let lf = df!(
            "text" => ["a", "b"],
            "x" => [1i64, 2],
        )?.lazy();
        let domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("text", AtomDomain::<String>::default()),
            SeriesDomain::new("x", AtomDomain::<i64>::default()),
        ])?;
        assert!(make_verified_stable_lazyframe(
            domain,
            FrameDistance(SymmetricDistance),
            lf.select([col("x")]),
        ).is_err());
        Ok(())
    }

    #[test]
    fn rejects_unsupported_plans() -> Fallible<()> {
        let lf = df!(
            "x" => [Some(2i64), Some(3i64)],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), Some(false)],
        )?.lazy();
        let metric = FrameDistance(SymmetricDistance);
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().select([lit(5i64)]),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().select([col("x").sum()]),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().filter(col("x")),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric, lf.sort(["x"], SortMultipleOptions::default()),
        ).is_err());
        Ok(())
    }
}
