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
    domains::{
        verified_frame::VColumn,
        verified_plan::VFrameDomain,
        LazyFrameDomain, SeriesDomain,
    },
    error::*,
    metrics::{Bound, Bounds, FrameDistance, SymmetricDistance},
    transformations::make_stable_vplan::make_stable_vtable,
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
    // Certify execution on concrete tables, not construction of query syntax:
    // the FrameDistance metric applies to datasets, not to pairs of VPlans.
    // Margins are discarded conservatively because their Polars grouping
    // expressions cannot be represented in the verified DSL.
    let source = VFrameDomain::new(input_domain.series_domains.iter()
        .map(|column| column.cast_carrier::<VColumn>()).collect())?;
    let certified = make_stable_vtable(source, SymmetricDistance, plan.clone())?;
    let verified_map = certified.stability_map.clone();
    let verified_columns = certified.output_domain.series_domains.iter()
        .map(|column| column.cast_carrier::<Series>()).collect::<Vec<_>>();
    if columns != verified_columns {
        return fallible!(MakeTransformation,
            "Polars normalization and verified plan domains disagree");
    }
    let output_domain = LazyFrameDomain::new(verified_columns)?;
    let source_columns = input_domain.series_domains.iter()
        .map(|column| column.name.clone()).collect::<Vec<_>>();
    // Only the global distance crosses the Polars/VExpr boundary. The
    // verified table transformation supplies its 1-Lipschitz stability map.
    // The Polars backend equivalence remains a separate adapter obligation.
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
        StabilityMap::new_fallible(move |bounds: &Bounds| {
            let global = bounds.get_bound(&HashSet::new());
            Ok(Bounds(vec![Bound {
                by: HashSet::new(),
                per_group: global.per_group
                    .map(|d| verified_map.eval(&d)).transpose()?,
                num_groups: global.num_groups
                    .map(|d| verified_map.eval(&d)).transpose()?,
            }]))
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
    fn comparisons_match_polars_with_nulls() -> Fallible<()> {
        let input = df!(
            "x" => [Some(-1i64), Some(2), None, Some(5)],
            "lower" => [Some(0i64), Some(2), Some(1), None],
            "upper" => [Some(10i64), Some(4), None, Some(9)],
            "flag" => [Some(true), Some(false), None, Some(true)],
        )?;
        let query = input.clone().lazy().select([
            col("x").lt_eq(col("lower")).alias("le"),
            col("x").gt_eq(lit(2i64)).alias("ge"),
            col("flag").neq(lit(true)).alias("ne"),
        ]);
        check_polars_semantics(input.clone(), query, true)?;
        let query = input.clone().lazy()
            .filter(col("x").gt_eq(col("lower")))
            .select([
                col("x").neq(col("upper")).alias("different"),
                col("lower").lt_eq(col("upper")).alias("ordered"),
            ]);
        check_polars_semantics(input, query, true)
    }

    #[test]
    fn native_map_uses_verified_global_bound() -> Fallible<()> {
        use crate::metrics::Bound;

        let query = df!("x" => [1i64], "lower" => [0i64],
            "upper" => [2i64], "flag" => [true])?.lazy()
            .filter(col("flag"))
            .select([col("x").alias("value")]);
        let trans = make_verified_stable_lazyframe(
            domain()?, FrameDistance(SymmetricDistance), query,
        )?;
        let bounds = Bounds::from(5)
            .with_bound(Bound::by([col("x")]).with_per_group(2));
        let mapped = trans.map(&bounds)?;
        assert_eq!(mapped.0.len(), 1);
        assert!(mapped.0[0].by.is_empty());
        assert_eq!(mapped.get_bound(&HashSet::new()).per_group, Some(5));
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
        let transformation = make_verified_stable_lazyframe(
            input_domain.clone(), FrameDistance(SymmetricDistance), query.clone(),
        )?;
        let reference = verified::VTable {
            schema: crate::domains::verified_frame::kernel_schema(&input_domain.series_domains)?,
            rows: to_cells(&source)?,
        };
        let physical = make_stable_vtable(
            VFrameDomain::new(input_domain.series_domains.iter()
                .map(|column| column.cast_carrier::<VColumn>()).collect())?,
            SymmetricDistance, plan,
        )?;
        let expected = physical.invoke(&reference)?;
        let actual = transformation.invoke(&source.clone().lazy())?.collect()?;
        if compare_original {
            assert_eq!(actual, query.collect()?);
        }
        assert!(to_cells(&actual)? == expected.rows,
            "backend disagrees with certified table evaluator");
        assert_eq!(expected.schema,
            crate::domains::verified_frame::kernel_schema(
                &physical.output_domain.series_domains)?);
        assert_eq!(transformation.output_domain.series_domains, expected_columns);
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
    fn polars_matches_verified_on_nullable_slices() -> Fallible<()> {
        let full = df!(
            "x" => [Some(-5i64), Some(8), None, Some(3)],
            "lower" => [Some(0i64), Some(10), Some(3), None],
            "upper" => [Some(6i64), Some(2), None, Some(2)],
            "flag" => [Some(true), None, Some(false), Some(true)],
        )?;
        for start in 0..=full.height() {
            for length in 0..=full.height() - start {
                let input = full.slice(start as i64, length);
                let query = input.clone().lazy()
                    .select([
                        col("x").clip(col("lower"), col("upper")).alias("clipped"),
                        col("flag").and(col("x").gt(lit(1i64))).alias("keep"),
                    ])
                    .filter(col("keep"));
                check_polars_semantics(input.clone(), query, false)?;

                let query = input.clone().lazy()
                    .filter(col("flag").and(col("x").gt(lit(0i64))))
                    .select([
                        col("lower").clip(col("x"), col("upper")).alias("chosen"),
                        col("flag"),
                    ]);
                check_polars_semantics(input, query, false)?;
            }
        }
        Ok(())
    }

    #[test]
    fn native_and_verified_projection_domains_agree() -> Fallible<()> {
        let query = df!(
            "x" => [Some(2i64), Some(7)],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), None],
        )?.lazy()
            .select([
                col("x").alias("renamed"),
                col("flag").and(col("x").gt(lit(3i64))).alias("derived"),
            ])
            .filter(col("derived"));
        let mut input = domain()?;
        input.series_domains[0] =
            SeriesDomain::new("x", AtomDomain::<i64>::new_closed((0, 10))?);
        let (plan, columns) =
            frontend::normalize_plan(&query.logical_plan, &input.series_domains)?;
        let v_source = crate::domains::verified_plan::VFrameDomain::new(
            input.series_domains.iter().map(|column|
                column.cast_carrier::<crate::domains::verified_frame::VColumn>()
            ).collect(),
        )?;
        let v_output = crate::domains::verified_plan::analyze_plan(&plan, &v_source)?;
        let inferred = v_output.series_domains.iter()
            .map(|column| column.cast_carrier::<Series>())
            .collect::<Vec<_>>();
        assert_eq!(columns, inferred);
        Ok(())
    }

    #[test]
    fn verified_constructor_preserves_forwarded_bounds() -> Fallible<()> {
        let input_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("x", AtomDomain::<i64>::new_closed((0, 10))?),
            SeriesDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])?;
        let source = df!(
            "x" => [1i64, 8, 3],
            "flag" => [Some(true), Some(false), None],
        )?.lazy();
        let query = source.clone().filter(col("flag"))
            .select([col("x").alias("bounded")]);
        let trans = make_verified_stable_lazyframe(
            input_domain, FrameDistance(SymmetricDistance), query,
        )?;
        assert_eq!(
            trans.output_domain.series_domains[0]
                .atom_domain::<i64>()?.get_closed_bounds()?, (0, 10),
        );
        assert_eq!(
            trans.invoke(&source)?.collect()?,
            df!("bounded" => [1i64])?,
        );
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
