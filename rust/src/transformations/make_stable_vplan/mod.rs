//! Recursive, backend-independent constructors over the verified query DSL.
//!
//! Every constructor consumes a node (or, later, a recognized subtree), then
//! composes with its recursively constructed child transformation.

use std::collections::HashSet;

mod stability;
mod certified;

use crate::{
    core::{Domain, Function, StabilityMap, Transformation},
    domains::{
        verified_query_domain::kernel_schema,
        verified_plan::{
            filter_domain, project_domain, VFrameBounds, VFrameMetric, VPlanDomain,
        },
    },
    error::*,
    verified_query::{self as verified, VExpr, VField, VPlan},
};

type VTransformation =
    Transformation<VPlanDomain, VFrameMetric, VPlanDomain, VFrameMetric>;

/// Admit only results whose verified schema matches the OpenDP output-domain
/// contract. The executable checker itself is proved in Verus.
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

fn make_input(
    domain: VPlanDomain,
    metric: VFrameMetric,
) -> Fallible<VTransformation> {
    let source = kernel_schema(&domain.source.series_domains)?;
    let expected = kernel_schema(&domain.frame.series_domains)?;
    let input_domain = domain.clone();
    Transformation::new(
        domain.clone(), metric.clone(), domain, metric,
        Function::new_fallible(move |input: &VPlan| {
            // The source-relative contract includes bounds, names, and margins,
            // none of which are described by the Verus type schema alone.
            if !input_domain.member(input)? {
                return fallible!(MakeTransformation,
                    "caller plan does not satisfy the verified input domain");
            }
            checked_result(
                certified::make_input(input.clone(), &source),
                &expected,
            )
        }),
        StabilityMap::new(Clone::clone),
    )
}

fn global_bound(bounds: &VFrameBounds) -> VFrameBounds {
    // The current Verus stability theorem proves symmetric row distance,
    // not preservation of arbitrary grouping-dependent bounds.
    crate::metrics::Bounds(vec![bounds.get_bound(&HashSet::new())])
}

fn make_project(
    domain: VPlanDomain,
    metric: VFrameMetric,
    exprs: Vec<VExpr>,
) -> Fallible<VTransformation> {
    let output = domain.with_output(project_domain(&domain.frame, &exprs)?);
    let source = kernel_schema(&domain.source.series_domains)?;
    let child = kernel_schema(&domain.frame.series_domains)?;
    let expected = kernel_schema(&output.frame.series_domains)?;
    Transformation::new(
        domain, metric.clone(), output, metric,
        Function::new_fallible(move |input: &VPlan| {
            checked_result(
                certified::make_project(input.clone(), exprs.clone(), &child, &source),
                &expected,
            )
        }),
        StabilityMap::new(global_bound),
    )
}

fn make_filter(
    domain: VPlanDomain,
    metric: VFrameMetric,
    predicate: VExpr,
) -> Fallible<VTransformation> {
    let output = domain.with_output(filter_domain(&domain.frame, &predicate)?);
    let source = kernel_schema(&domain.source.series_domains)?;
    let child = kernel_schema(&domain.frame.series_domains)?;
    let expected = kernel_schema(&output.frame.series_domains)?;
    Transformation::new(
        domain, metric.clone(), output, metric,
        Function::new_fallible(move |input: &VPlan| {
            checked_result(
                certified::make_filter(input.clone(), predicate.clone(), &child, &source),
                &expected,
            )
        }),
        StabilityMap::new(global_bound),
    )
}

/// Construct a stable VPlan-to-VPlan transformation from an AST subtree.
///
/// The leaf is the identity on the caller's plan. Project and Filter extend
/// that plan; source data is neither accessed nor embedded in the AST.
pub(crate) fn make_stable_vplan(
    input_domain: VPlanDomain,
    input_metric: VFrameMetric,
    plan: VPlan,
) -> Fallible<VTransformation> {
    match plan {
        VPlan::Input => make_input(input_domain, input_metric),
        VPlan::Project { input, exprs } => {
            let parent = make_stable_vplan(input_domain, input_metric, *input)?;
            let (domain, metric) = parent.output_space();
            parent >> make_project(domain, metric, exprs)?
        },
        VPlan::Filter { input, predicate } => {
            let parent = make_stable_vplan(input_domain, input_metric, *input)?;
            let (domain, metric) = parent.output_space();
            parent >> make_filter(domain, metric, predicate)?
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
