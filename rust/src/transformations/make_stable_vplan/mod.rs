//! Verified, backend-independent OpenDP constructors for row-local VPlans.
//! Each operator constructs and composes its own OpenDP Transformation.

mod input;
mod filter;
mod project;
mod stability;

// Verus-only executable constructors are colocated with their OpenDP nodes.
#[path = "input/proof.rs"]
mod input_proof;
#[path = "filter/proof.rs"]
mod filter_proof;
#[path = "project/proof.rs"]
mod project_proof;

use std::collections::HashSet;
use crate::{
    core::Transformation,
    domains::verified_plan::{VFrameBounds, VFrameMetric, VPlanDomain},
    error::*,
    verified_query::{self as verified, VExpr, VField, VPlan},
};

type VTransformation =
    Transformation<VPlanDomain, VFrameMetric, VPlanDomain, VFrameMetric>;

fn checked_result(
    result: Option<(VPlan, Vec<VField>)>,
    expected: &Vec<VField>,
) -> Fallible<VPlan> {
    match result {
        Some((plan, fields)) if &fields == expected => Ok(plan),
        _ => fallible!(MakeTransformation,
            "verified plan construction does not satisfy its declared schema"),
    }
}

fn global_bound(bounds: &VFrameBounds) -> VFrameBounds {
    // Without a grouping-specific proof, retain only the global bound.
    crate::metrics::Bounds(vec![bounds.get_bound(&HashSet::new())])
}

/// Compose node-level transformations, preserving their OpenDP contracts.
pub(crate) fn make_stable_vplan(
    input_domain: VPlanDomain,
    input_metric: VFrameMetric,
    plan: VPlan,
) -> Fallible<VTransformation> {
    match plan {
        VPlan::Input => input::make_input(input_domain, input_metric),
        VPlan::Project { input, exprs } =>
            project::make_project(input_domain, input_metric, *input, exprs),
        VPlan::Filter { input, predicate } =>
            filter::make_filter(input_domain, input_metric, *input, predicate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::Domain,
        domains::{
            verified_plan::{analyze_plan, VFrameDomain},
            AtomDomain, ColumnDomain, Margin, OptionDomain,
        },
        metrics::{Bound, Bounds, SymmetricDistance},
        verified_query::{self as verified, VField, VTable, VValue, VDType, VBinaryOp},
    };

    fn source() -> Fallible<VFrameDomain> {
        VFrameDomain::new(vec![
            ColumnDomain::new("x", AtomDomain::<i64>::new_closed((0, 10))?),
            ColumnDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    fn table() -> VTable {
        VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: false },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(3)), Some(VValue::Bool(false))],
                vec![Some(VValue::Int(2)), None],
            ],
        }
    }

    fn query() -> VPlan {
        VPlan::Project {
            input: Box::new(VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            }),
            exprs: vec![
                VExpr::Alias {
                    expr: Box::new(VExpr::Clip {
                        expr: Box::new(VExpr::Column(0)),
                        lower: None,
                        upper: Some(Box::new(VExpr::Literal(VValue::Int(5)))),
                    }),
                    name: "clipped".into(),
                },
                VExpr::Alias {
                    expr: Box::new(VExpr::Column(1)),
                    name: "selected".into(),
                },
            ],
        }
    }

    #[test]
    fn recursive_project_filter_domain_and_execution() -> Fallible<()> {
        let input = source()?;
        let domain = VPlanDomain::new(input.clone())?;
        let metric = VFrameMetric::new(SymmetricDistance);
        let transformation = make_stable_vplan(domain.clone(), metric, query())?;
        assert_eq!(transformation.output_domain.frame.series_domains.len(), 2);
        assert_eq!(transformation.output_domain.frame.series_domains[0].name.as_str(), "clipped");
        assert_eq!(transformation.output_domain.frame.series_domains[1].name.as_str(), "selected");

        let result = transformation.invoke(&VPlan::Input)?;
        assert!(transformation.output_domain.member(&result)?);
        assert_eq!(transformation.output_domain.frame,
                   analyze_plan(&result, &domain.source)?);

        let data = table();
        assert!(input.member(&data)?);
        let values = verified::evaluate(&result, &data).expect("typed plan evaluates");
        assert_eq!(values, vec![vec![Some(VValue::Int(5)), Some(VValue::Bool(true))]]);
        let schema = verified::check_plan(&result, &data.schema).unwrap();
        assert!(transformation.output_domain.frame.member(
            &VTable { schema, rows: values },
        )?);
        Ok(())
    }

    #[test]
    fn insertion_and_deletion_have_unit_symmetric_distance() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        let transformation = make_stable_vplan(
            domain, VFrameMetric::new(SymmetricDistance), query(),
        )?;
        let compiled = transformation.invoke(&VPlan::Input)?;
        let original = verified::evaluate(&compiled, &table()).unwrap();

        let mut inserted = table();
        inserted.rows.push(vec![
            Some(VValue::Int(7)), Some(VValue::Bool(true)),
        ]);
        let with_insert = verified::evaluate(&compiled, &inserted).unwrap();
        assert_eq!(with_insert, vec![original[0].clone(), original[0].clone()]);

        let mut deleted = table();
        deleted.rows.remove(0);
        let with_delete = verified::evaluate(&compiled, &deleted).unwrap();
        assert!(with_delete.is_empty());

        assert_eq!(
            transformation.map(&Bounds::from(1))?
                .get_bound(&HashSet::new()).per_group, Some(1),
        );
        Ok(())
    }

    #[test]
    fn preserves_identity_column_bounds_but_not_derived_bounds() -> Fallible<()> {
        let exprs = vec![
            VExpr::Alias { expr: Box::new(VExpr::Column(0)), name: "x2".into() },
            VExpr::Alias {
                expr: Box::new(VExpr::Binary {
                    left: Box::new(VExpr::Column(0)),
                    op: VBinaryOp::Gt,
                    right: Box::new(VExpr::Literal(VValue::Int(5))),
                }),
                name: "large".into(),
            },
        ];
        let transformation = make_stable_vplan(
            VPlanDomain::new(source()?)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Project { input: Box::new(VPlan::Input), exprs },
        )?;
        let columns = &transformation.output_domain.frame.series_domains;
        assert_eq!(columns[0].atom_domain::<i64>()?.get_closed_bounds()?, (0, 10));
        assert_eq!(columns[1].dtype(), polars::prelude::DataType::Boolean);
        Ok(())
    }

    #[test]
    fn filter_respects_group_margins_with_duplicate_rows() -> Fallible<()> {
        let source = source()?
            .with_margin(Margin::select().with_max_length(3))?
            .with_margin(Margin::by([VExpr::Column(0)])
                .with_max_length(2).with_max_groups(2))?;
        let data = VTable {
            schema: table().schema,
            rows: vec![
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(3)), Some(VValue::Bool(false))],
            ],
        };
        assert!(source.member(&data)?);
        let filter = make_stable_vplan(
            VPlanDomain::new(source)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            },
        )?;
        let plan = filter.invoke(&VPlan::Input)?;
        let output = VTable {
            schema: verified::check_plan(&plan, &data.schema).unwrap(),
            rows: verified::evaluate(&plan, &data).unwrap(),
        };
        assert_eq!(output.rows, data.rows[..2]);
        assert!(filter.output_domain.frame.member(&output)?);
        let mut extra = output;
        extra.rows.push(data.rows[0].clone());
        assert!(!filter.output_domain.frame.member(&extra)?);
        Ok(())
    }

    #[test]
    fn filter_preserves_grouped_distance_bounds() -> Fallible<()> {
        let t = make_stable_vplan(
            VPlanDomain::new(source()?)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            },
        )?;
        let input: Bounds<VExpr> = Bounds::from(5)
            .with_bound(Bound::by([VExpr::Column(0)]).with_per_group(2).with_num_groups(3));
        assert_eq!(t.map(&input)?, input);
        let grouped = HashSet::from([VExpr::Column(0)]);
        assert_eq!(t.map(&input)?.get_bound(&grouped).per_group, Some(2));
        assert_eq!(t.map(&input)?.get_bound(&grouped).num_groups, Some(3));
        Ok(())
    }

    #[test]
    fn filter_preserves_concrete_grouped_distances() -> Fallible<()> {
        use std::collections::HashMap;

        fn components(left: &VTable, right: &VTable, by: &HashSet<VExpr>) -> (u32, u32) {
            // Fix one ordering for both datasets; group keys are sets in Bounds.
            let keys = by.iter().collect::<Vec<_>>();
            let mut groups: HashMap<Vec<Option<VValue>>, HashMap<Vec<Option<VValue>>, i64>> =
                HashMap::new();
            for (table, sign) in [(left, 1_i64), (right, -1)] {
                for row in &table.rows {
                    let key = keys.iter()
                        .map(|expr| verified::eval_expr_row(expr, row).unwrap())
                        .collect::<Vec<_>>();
                    *groups.entry(key).or_default().entry(row.clone()).or_default() += sign;
                }
            }
            let distances = groups.values()
                .map(|rows| rows.values().map(|n| n.unsigned_abs() as u32).sum::<u32>())
                .collect::<Vec<_>>();
            (distances.iter().copied().max().unwrap_or(0),
                distances.iter().filter(|d| **d > 0).count() as u32)
        }

        let left = VTable {
            schema: table().schema,
            rows: vec![
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(3)), Some(VValue::Bool(false))],
            ],
        };
        let right = VTable {
            schema: left.schema.clone(),
            rows: vec![
                vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(4)), Some(VValue::Bool(false))],
                vec![Some(VValue::Int(4)), Some(VValue::Bool(false))],
            ],
        };
        let transformation = make_stable_vplan(
            VPlanDomain::new(source()?)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter { input: Box::new(VPlan::Input), predicate: VExpr::Column(1) },
        )?;
        let plan = transformation.invoke(&VPlan::Input)?;
        let filter_table = |input: &VTable| VTable {
            schema: input.schema.clone(),
            rows: verified::evaluate(&plan, input).unwrap(),
        };
        let (out_left, out_right) = (filter_table(&left), filter_table(&right));
        let bounds: Bounds<VExpr> = Bounds(vec![
            Bound::by(Vec::<VExpr>::new()).with_per_group(4).with_num_groups(1),
            Bound::by([VExpr::Column(0)]).with_per_group(2).with_num_groups(3),
            Bound::by([VExpr::Column(1)]).with_per_group(3).with_num_groups(2),
        ]);
        let output_bounds = transformation.map(&bounds)?;
        assert_eq!(output_bounds, bounds);
        for bound in &output_bounds.0 {
            let (before_max, before_groups) = components(&left, &right, &bound.by);
            let (after_max, after_groups) = components(&out_left, &out_right, &bound.by);
            assert!(after_max <= before_max && after_groups <= before_groups);
            assert!(before_max <= bound.per_group.unwrap());
            assert!(before_groups <= bound.num_groups.unwrap());
            assert!(after_max <= bound.per_group.unwrap());
            assert!(after_groups <= bound.num_groups.unwrap());
        }
        Ok(())
    }

    #[test]
    fn filter_clears_invariants_and_projection_drops_grouped_margins() -> Fallible<()> {
        let input = source()?
            .with_margin(Margin::select().with_max_length(5).with_invariant_lengths())?
            .with_margin(Margin::by([VExpr::Column(0)]).with_max_groups(3).with_invariant_keys())?;
        let filtered = make_stable_vplan(
            VPlanDomain::new(input.clone())?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter { input: Box::new(VPlan::Input), predicate: VExpr::Column(1) },
        )?;
        assert_eq!(filtered.output_domain.frame.margins.len(), 2);
        assert!(filtered.output_domain.frame.margins.iter().all(|m| m.invariant.is_none()));

        let projected = make_stable_vplan(
            VPlanDomain::new(input)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Project {
                input: Box::new(VPlan::Input),
                exprs: vec![VExpr::Column(1)],
            },
        )?;
        assert_eq!(projected.output_domain.frame.margins.len(), 1);
        assert!(projected.output_domain.frame.margins[0].by.is_empty());
        Ok(())
    }

    #[test]
    fn conservatively_forgets_grouped_distance_bounds() -> Fallible<()> {
        let t = make_stable_vplan(
            VPlanDomain::new(source()?)?,
            VFrameMetric::new(SymmetricDistance),
            query(),
        )?;
        let d_in: Bounds<VExpr> = Bounds::from(3)
            .with_bound(Bound::by([VExpr::Column(0)]).with_per_group(1));
        let d_out = t.map(&d_in)?;
        assert_eq!(d_out.0.len(), 1);
        assert!(d_out.0[0].by.is_empty());
        assert_eq!(d_out.0[0].per_group, Some(3));
        Ok(())
    }

    #[test]
    fn composes_on_a_nontrivial_caller_plan() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        let metric = VFrameMetric::new(SymmetricDistance);
        let transformation = make_stable_vplan(domain.clone(), metric, query())?;
        // This filter is a member of the input domain: it preserves the
        // column constraints, and the domain has no grouping invariants.
        let caller = VPlan::Filter {
            input: Box::new(VPlan::Input),
            predicate: VExpr::Column(1),
        };
        assert!(domain.member(&caller)?);
        let composed = transformation.invoke(&caller)?;
        assert!(transformation.output_domain.member(&composed)?);
        let rows = verified::evaluate(&composed, &table()).unwrap();
        assert_eq!(rows, vec![vec![
            Some(VValue::Int(5)), Some(VValue::Bool(true)),
        ]]);
        Ok(())
    }

    #[test]
    fn rejects_typed_caller_with_incompatible_value_domains() -> Fallible<()> {
        let input = source()?;
        let domain = VPlanDomain::new(input)?;
        let transformation = make_stable_vplan(
            domain.clone(),
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            },
        )?;
        // Same names, dtypes and nullability as the source, but the clipped
        // column has a broad domain rather than the source's bounded domain.
        let caller = VPlan::Project {
            input: Box::new(VPlan::Input),
            exprs: vec![
                VExpr::Alias {
                    expr: Box::new(VExpr::Clip {
                        expr: Box::new(VExpr::Column(0)),
                        lower: Some(Box::new(VExpr::Literal(VValue::Int(-100)))),
                        upper: None,
                    }),
                    name: "x".into(),
                },
                VExpr::Column(1),
            ],
        };
        assert!(!domain.member(&caller)?);
        assert!(transformation.invoke(&caller).is_err());
        Ok(())
    }

    #[test]
    fn rejects_invalid_child_and_projection_nodes() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        let metric = VFrameMetric::new(SymmetricDistance);
        let invalid_filter = VPlan::Filter {
            input: Box::new(VPlan::Input),
            predicate: VExpr::Column(0),
        };
        assert!(make_stable_vplan(domain.clone(), metric.clone(), invalid_filter).is_err());
        let duplicate = VPlan::Project {
            input: Box::new(VPlan::Input),
            exprs: vec![VExpr::Column(0), VExpr::Column(0)],
        };
        assert!(make_stable_vplan(domain.clone(), metric.clone(), duplicate).is_err());
        let invalid_child = VPlan::Filter {
            input: Box::new(VPlan::Project {
                input: Box::new(VPlan::Input),
                exprs: vec![VExpr::Column(0)],
            }),
            predicate: VExpr::Column(1),
        };
        assert!(make_stable_vplan(domain, metric, invalid_child).is_err());
        Ok(())
    }
}
