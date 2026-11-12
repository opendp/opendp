//! Source-relative domains for symbolic verified query plans.
//!
//! VPlan::Input is unbound. Its source-domain assumptions therefore belong
//! in this domain, not in the backend-independent verified AST.


use crate::{
    core::{Domain, MetricSpace},
    domains::{ColumnDomain, FrameDomain},
    error::*,
    metrics::{Bounds, FrameDistanceFor, SymmetricDistance},
    verified_query::{VExpr, VPlan, VTable},
};

use super::verified_query_domain::{check_filter_predicate, kernel_schema, project_columns};

pub(crate) type VFrameDomain = FrameDomain<VTable>;
pub(crate) type VFrameMetric = FrameDistanceFor<SymmetricDistance, VExpr>;
pub(crate) type VFrameBounds = Bounds<VExpr>;

/// A domain of query plans relative to a fixed source data contract.
///
/// `frame` describes the plan's output. `source` describes every allowed
/// input to its `Input` leaf. Neither domain contains private data.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct VPlanDomain {
    pub source: VFrameDomain,
    pub frame: VFrameDomain,
}

impl VPlanDomain {
    pub fn new(source: VFrameDomain) -> Fallible<Self> {
        kernel_schema(&source.series_domains)?;
        Ok(Self { frame: source.clone(), source })
    }

    pub fn with_output(&self, frame: VFrameDomain) -> Self {
        Self { source: self.source.clone(), frame }
    }
}

impl Domain for VPlanDomain {
    type Carrier = VPlan;

    fn member(&self, plan: &VPlan) -> Fallible<bool> {
        // Only plans whose output contract can be derived from the *same*
        // fixed source assumptions are admitted.
        Ok(analyze_plan(plan, &self.source)
            .is_ok_and(|inferred| inferred == self.frame))
    }
}

impl MetricSpace for (VPlanDomain, VFrameMetric) {
    fn check_space(&self) -> Fallible<()> {
        (self.0.frame.clone(), self.1.clone()).check_space()
    }
}

/// Output-domain rule for one projection node. It preserves exact column
/// domains only for identity column expressions, and keeps only global margins.
pub(crate) fn project_domain(
    input: &VFrameDomain, exprs: &[VExpr],
) -> Fallible<VFrameDomain> {
    let columns = project_columns(&input.series_domains, exprs)?;
    let margins = input.margins.iter()
        .filter(|margin| margin.by.is_empty())
        .cloned()
        .collect();
    VFrameDomain::new_with_margins(columns, margins)
}

/// Filtering preserves column constraints and group-size upper bounds,
/// but not dataset-to-dataset key/length invariants.
pub(crate) fn filter_domain(
    input: &VFrameDomain, predicate: &VExpr,
) -> Fallible<VFrameDomain> {
    check_filter_predicate(&input.series_domains, predicate)?;
    let mut output = input.clone();
    for margin in &mut output.margins {
        margin.invariant = None;
    }
    Ok(output)
}

/// Static domain propagation over a complete subtree. Reused by membership
/// and the recursive constructors so they cannot disagree on output domains.
pub(crate) fn analyze_plan(
    plan: &VPlan, source: &VFrameDomain,
) -> Fallible<VFrameDomain> {
    match plan {
        VPlan::Input => {
            kernel_schema(&source.series_domains)?;
            Ok(source.clone())
        },
        VPlan::Project { input, exprs } =>
            project_domain(&analyze_plan(input, source)?, exprs),
        VPlan::Filter { input, predicate } =>
            filter_domain(&analyze_plan(input, source)?, predicate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::DataType;
    use std::collections::HashSet;
    use crate::domains::{AtomDomain, Margin, OptionDomain};
    use crate::verified_query::{VValue, VBinaryOp};

    fn source() -> Fallible<VFrameDomain> {
        VFrameDomain::new(vec![
            ColumnDomain::new("x", AtomDomain::<i64>::new_closed((0, 10))?),
            ColumnDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    #[test]
    fn source_relative_membership() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        assert!(domain.member(&VPlan::Input)?);

        let query = VPlan::Project {
            input: Box::new(VPlan::Input),
            exprs: vec![
                VExpr::Alias {
                    expr: Box::new(VExpr::Column(0)),
                    name: "renamed".into(),
                },
                VExpr::Alias {
                    expr: Box::new(VExpr::Binary {
                        left: Box::new(VExpr::Column(0)),
                        op: VBinaryOp::Gt,
                        right: Box::new(VExpr::Literal(VValue::Int(5))),
                    }),
                    name: "above".into(),
                },
            ],
        };
        let output = analyze_plan(&query, &domain.source)?;
        assert_eq!(output.series_domains[0].name.as_str(), "renamed");
        assert_eq!(output.series_domains[0].atom_domain::<i64>()?.get_closed_bounds()?, (0, 10));
        assert_eq!(output.series_domains[1].dtype(), DataType::Boolean);
        assert!(domain.with_output(output).member(&query)?);
        assert!(!domain.member(&query)?); // different output contract
        Ok(())
    }

    #[test]
    fn rejects_invalid_subtrees_and_duplicate_names() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        let bad = VPlan::Filter {
            input: Box::new(VPlan::Input),
            predicate: VExpr::Column(0),
        };
        assert!(!domain.member(&bad)?);
        assert!(analyze_plan(&bad, &domain.source).is_err());

        let duplicate = VPlan::Project {
            input: Box::new(VPlan::Input),
            exprs: vec![VExpr::Column(0), VExpr::Column(0)],
        };
        assert!(analyze_plan(&duplicate, &domain.source).is_err());
        Ok(())
    }

    #[test]
    fn retains_only_safe_margins() -> Fallible<()> {
        let source = source()?
            .with_margin(Margin::select().with_max_length(5).with_invariant_lengths())?
            .with_margin(Margin::by([VExpr::Column(0)]).with_max_groups(3).with_invariant_keys())?;
        let selected = project_domain(&source, &[VExpr::Column(1)])?;
        assert_eq!(selected.margins.len(), 1);
        assert_eq!(selected.margins[0].max_length, Some(5));
        let filtered = filter_domain(&source, &VExpr::Column(1))?;
        assert_eq!(filtered.margins.len(), 2);
        assert!(filtered.margins.iter().all(|m| m.invariant.is_none()));
        Ok(())
    }

    #[test]
    fn expression_keys_and_metric_space() -> Fallible<()> {
        let domain = VPlanDomain::new(source()?)?;
        let metric = VFrameMetric::new(SymmetricDistance);
        (domain, metric).check_space()?;
        let bounds = VFrameBounds::from(3);
        assert_eq!(bounds.get_bound(&HashSet::new()).per_group, Some(3));
        Ok(())
    }
}
