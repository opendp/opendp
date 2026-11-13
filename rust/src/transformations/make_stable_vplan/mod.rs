//! Verified, backend-independent OpenDP constructors for row-local VPlans.
//! Each operator constructs and composes its own OpenDP Transformation.

mod input;
mod filter;
mod project;
mod stability;
mod lazy;

// Verus-only executable constructors are colocated with their OpenDP nodes.
#[path = "input/proof.rs"]
mod input_proof;
#[path = "filter/proof.rs"]
mod filter_proof;
#[path = "table/proof.rs"]
mod table_proof;
#[path = "project/proof.rs"]
mod project_proof;

use std::collections::HashSet;
use crate::combinators::make_chain_tt_certified_symmetric;
use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::verified_plan::{VFrameBounds, VFrameDomain, VFrameMetric, VPlanDomain},
    metrics::SymmetricDistance,
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

/// Construct a verified concrete-table transformation for a row-local plan.
///
/// Unlike the symbolic VPlan constructor, this executes the individual
/// certified operators. The output domain of each node is the input domain
/// of the next, so native composition carries the stability maps along.
pub(crate) fn make_stable_vtable(
    input_domain: VFrameDomain,
    metric: SymmetricDistance,
    plan: VPlan,
) -> Fallible<Transformation<VFrameDomain, SymmetricDistance, VFrameDomain, SymmetricDistance>> {
    match plan {
        VPlan::Input => input::make_stable_vtable_input(input_domain, metric),
        VPlan::Filter { input, predicate } => {
            let prior = make_stable_vtable(input_domain, metric, *input)?;
            let (domain, metric) = prior.output_space();
            let step = filter::make_stable_vtable_filter(domain, metric, predicate)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
        VPlan::Project { input, exprs } => {
            let prior = make_stable_vtable(input_domain, metric, *input)?;
            let (domain, metric) = prior.output_space();
            let step = project::make_stable_vtable_project(domain, metric, exprs)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
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
    fn certified_table_identity_preserves_membership_and_distance() -> Fallible<()> {
        let identity = make_stable_vtable(source()?, SymmetricDistance, VPlan::Input)?;
        let original = table();
        let returned = identity.invoke(&original)?;
        assert_eq!(returned.schema, original.schema);
        assert_eq!(returned.rows, original.rows);
        assert_eq!(identity.map(&5)?, 5);
        let mut invalid = original;
        invalid.rows[0][0] = Some(VValue::Int(11));
        assert!(identity.invoke(&invalid).is_err());
        Ok(())
    }

    #[test]
    fn certified_table_filter_builds_valid_transformation() -> Fallible<()> {
        let domain = source()?;
        let trans = filter::make_stable_vtable_filter(
            domain.clone(), SymmetricDistance, VExpr::Column(1),
        )?;
        let original = table();
        assert!(trans.input_domain.member(&original)?);
        let filtered = trans.invoke(&original)?;
        assert!(trans.output_domain.member(&filtered)?);
        assert_eq!(filtered.rows, vec![original.rows[0].clone()]);
        for d in [0, 1, 3, u32::MAX] {
            assert_eq!(trans.map(&d)?, d);
        }
        // Domain membership on this path calls the verified bound checker.
        let mut out_of_bounds = original.clone();
        out_of_bounds.rows[0][0] = Some(VValue::Int(11));
        assert!(!trans.input_domain.member(&out_of_bounds)?);
        assert!(trans.invoke(&out_of_bounds).is_err());
        let mut wrong_nullability = original.clone();
        wrong_nullability.schema[0].nullable = true;
        assert!(!trans.input_domain.member(&wrong_nullability)?);
        assert!(trans.invoke(&wrong_nullability).is_err());
        let mut appended = original.clone();
        appended.rows.push(original.rows[0].clone());
        let more = trans.invoke(&appended)?;
        assert!(trans.output_domain.member(&more)?);
        assert_eq!(more.rows.len(), filtered.rows.len() + 1);
        let mut deleted = original.clone();
        deleted.rows.remove(0);
        let less = trans.invoke(&deleted)?;
        assert!(less.rows.is_empty());
        assert!(trans.output_domain.member(&less)?);
        assert!(filter::make_stable_vtable_filter(
            domain.clone(), SymmetricDistance,
            VExpr::Column(0),
        ).is_err());
        assert!(filter::make_stable_vtable_filter(
            domain.with_margin(Margin::select().with_max_length(3))?,
            SymmetricDistance, VExpr::Column(1),
        ).is_err());
        Ok(())
    }


    #[test]
    fn certified_table_project_preserves_column_bounds_and_duplicates() -> Fallible<()> {
        let source = source()?;
        let selected = vec![
            VExpr::Alias {
                expr: Box::new(VExpr::Column(0)), name: "first".into(),
            },
            VExpr::Alias {
                expr: Box::new(VExpr::Column(0)), name: "second".into(),
            },
            VExpr::Column(1),
        ];
        let trans = project::make_stable_vtable_project(
            source.clone(), SymmetricDistance, selected,
        )?;
        let data = table();
        assert!(trans.input_domain.member(&data)?);
        let output = trans.invoke(&data)?;
        assert!(trans.output_domain.member(&output)?);
        assert_eq!(output.rows, data.rows.iter().map(|row| {
            vec![row[0], row[0], row[1]]
        }).collect::<Vec<_>>());
        assert_eq!(output.schema.len(), 3);
        let cols = &trans.output_domain.series_domains;
        assert_eq!(cols[0].name.as_str(), "first");
        assert_eq!(cols[1].name.as_str(), "second");
        assert_eq!(cols[0].atom_domain::<i64>()?.get_closed_bounds()?, (0, 10));
        assert_eq!(cols[1].atom_domain::<i64>()?.get_closed_bounds()?, (0, 10));
        for d in [0, 1, 3, u32::MAX] {
            assert_eq!(trans.map(&d)?, d);
        }
        let mut bad = data.clone();
        bad.rows[0][0] = Some(VValue::Int(11));
        assert!(trans.invoke(&bad).is_err());
        let derived = project::make_stable_vtable_project(
            source, SymmetricDistance, vec![
                VExpr::Alias {
                    expr: Box::new(VExpr::Binary {
                        left: Box::new(VExpr::Column(0)),
                        op: VBinaryOp::Gt,
                        right: Box::new(VExpr::Literal(VValue::Int(5))),
                    }),
                    name: "above".into(),
                },
                VExpr::Alias {
                    expr: Box::new(VExpr::Clip {
                        expr: Box::new(VExpr::Column(0)),
                        lower: None,
                        upper: Some(Box::new(VExpr::Literal(VValue::Int(5)))),
                    }),
                    name: "clipped".into(),
                },
                VExpr::Alias {
                    expr: Box::new(VExpr::Binary {
                        left: Box::new(VExpr::Column(1)),
                        op: VBinaryOp::Eq,
                        right: Box::new(VExpr::Literal(VValue::Bool(true))),
                    }),
                    name: "flag_is_true".into(),
                },
            ],
        )?;
        let result = derived.invoke(&data)?;
        assert!(derived.output_domain.member(&result)?);
        assert_eq!(result.rows, vec![
            vec![Some(VValue::Bool(true)), Some(VValue::Int(5)), Some(VValue::Bool(true))],
            vec![Some(VValue::Bool(false)), Some(VValue::Int(3)), Some(VValue::Bool(false))],
            vec![Some(VValue::Bool(false)), Some(VValue::Int(2)), None],
        ]);
        let columns = &derived.output_domain.series_domains;
        assert!(columns[0].atom_domain::<bool>()?.bounds.is_none());
        assert!(columns[1].atom_domain::<i64>()?.bounds.is_none());
        assert!(columns[2].nullable);
        assert!(columns[2].atom_domain::<bool>()?.bounds.is_none());
        assert_eq!(derived.map(&u32::MAX)?, u32::MAX);
        Ok(())
    }

    #[test]
    fn concrete_filter_then_project_composes_certificates() -> Fallible<()> {
        let domain = source()?;
        let plan = query();
        let data = table();
        let expected = verified::evaluate(&plan, &data)
            .expect("well-typed filter and projection evaluate");

        let transformation = make_stable_vtable(domain.clone(), SymmetricDistance, plan)?;
        let output = transformation.invoke(&data)?;
        assert!(transformation.input_domain.member(&data)?);
        assert!(transformation.output_domain.member(&output)?);
        assert_eq!(output.rows, expected);
        assert_eq!(output.rows, vec![
            vec![Some(VValue::Int(5)), Some(VValue::Bool(true))],
        ]);
        assert_eq!(output.schema.len(), 2);
        let columns = &transformation.output_domain.series_domains;
        assert_eq!(columns[0].name.as_str(), "clipped");
        assert!(columns[0].atom_domain::<i64>()?.bounds.is_none());
        assert_eq!(columns[1].name.as_str(), "selected");
        for d in [0, 1, 3, u32::MAX] {
            assert_eq!(transformation.map(&d)?, d);
        }

        // Both insertions and removals are accounted for by the composed map.
        let mut inserted = data.clone();
        inserted.rows.push(data.rows[0].clone());
        let more = transformation.invoke(&inserted)?;
        assert_eq!(more.rows.len(), output.rows.len() + 1);
        let mut removed = data.clone();
        removed.rows.remove(0);
        assert!(transformation.invoke(&removed)?.rows.is_empty());

        let mut invalid = data.clone();
        invalid.rows[0][0] = Some(VValue::Int(11));
        assert!(!transformation.input_domain.member(&invalid)?);
        assert!(transformation.invoke(&invalid).is_err());
        Ok(())
    }

    #[test]
    fn concrete_project_then_filter_uses_projected_domain() -> Fallible<()> {
        let plan = VPlan::Filter {
            input: Box::new(VPlan::Project {
                input: Box::new(VPlan::Input),
                exprs: vec![
                    VExpr::Alias {
                        expr: Box::new(VExpr::Column(0)), name: "copied_x".into(),
                    },
                    VExpr::Alias {
                        expr: Box::new(VExpr::Binary {
                            left: Box::new(VExpr::Column(0)),
                            op: VBinaryOp::Gt,
                            right: Box::new(VExpr::Literal(VValue::Int(5))),
                        }),
                        name: "above_five".into(),
                    },
                ],
            }),
            predicate: VExpr::Column(1),
        };
        let data = table();
        let expected = verified::evaluate(&plan, &data)
            .expect("well-typed projection and filter evaluate");
        let transformation = make_stable_vtable(source()?, SymmetricDistance, plan)?;
        let output = transformation.invoke(&data)?;
        assert!(transformation.output_domain.member(&output)?);
        assert_eq!(output.rows, expected);
        assert_eq!(output.rows, vec![
            vec![Some(VValue::Int(8)), Some(VValue::Bool(true))],
        ]);
        assert_eq!(transformation.output_domain.series_domains[0]
            .atom_domain::<i64>()?.get_closed_bounds()?, (0, 10));
        assert!(transformation.output_domain.series_domains[1]
            .atom_domain::<bool>()?.bounds.is_none());
        for d in [0, 1, 3, u32::MAX] {
            assert_eq!(transformation.map(&d)?, d);
        }
        Ok(())
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
    fn filter_map_rejects_untyped_grouping_keys() -> Fallible<()> {
        let t = make_stable_vplan(
            VPlanDomain::new(source()?)?,
            VFrameMetric::new(SymmetricDistance),
            VPlan::Filter { input: Box::new(VPlan::Input), predicate: VExpr::Column(1) },
        )?;
        // Aliases, nullable keys, and empty groupings are all valid.
        let valid = Bounds(vec![
            Bound::by([VExpr::Alias {
                expr: Box::new(VExpr::Column(0)), name: "renamed".into(),
            }]).with_per_group(2),
            Bound::by([VExpr::Column(1)]).with_num_groups(2),
            Bound::by([VExpr::Column(0)]).with_per_group(2).with_num_groups(3),
            Bound::by(Vec::<VExpr>::new()).with_per_group(3),
        ]);
        assert_eq!(t.map(&valid)?, valid);
        let invalid = Bounds(vec![
            Bound::by([VExpr::Column(99)]).with_per_group(1),
        ]);
        assert!(t.map(&invalid).is_err());
        Ok(())
    }

    #[test]
    fn filter_preserves_concrete_grouped_distances() -> Fallible<()> {
        use std::collections::HashMap;

        fn components(left: &VTable, right: &VTable, by: &HashSet<VExpr>) -> (u32, u32) {
            // Fix one ordering for both datasets; group keys are sets in Bounds.
            let keys = by.iter().cloned().collect::<Vec<_>>();
            let mut groups: HashMap<Vec<Option<VValue>>, HashMap<Vec<Option<VValue>>, i64>> =
                HashMap::new();
            for (table, sign) in [(left, 1_i64), (right, -1)] {
                for row in &table.rows {
                    let key = verified::project::eval_prefix(
                        &keys, row, keys.len(),
                    ).unwrap();
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
        let source = source()?
            .with_margin(Margin::by([VExpr::Column(0)]).with_max_length(2))?;
        let domain = VPlanDomain::new(source)?;
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
        // The final projection drops the grouped margin.
        assert!(transformation.output_domain.frame.margins.is_empty());
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
