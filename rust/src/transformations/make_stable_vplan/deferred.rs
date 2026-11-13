//! OpenDP transformations over executable, bound reference plans.
//!
//! The generic Deferred<S> and AST builders live in the standalone kernel.
//! Certification here is restricted to VTable bindings, whose realized
//! semantics are verified. External backends need their own correspondence
//! obligations before receiving certified transformations.

use crate::{
    combinators::make_chain_tt_certified_symmetric,
    core::{CertifiedTransformationParts, Function, StabilityMap, Transformation},
    domains::{FrameDomain, verified_plan::VFrameDomain},
    error::*,
    metrics::SymmetricDistance,
    verified_query::{VExpr, Deferred, VPlan, VTable},
};

use super::{filter, input, project};

type DeferredDomain = FrameDomain<Deferred<VTable>>;
type TableTransformation =
    Transformation<VFrameDomain, SymmetricDistance, VFrameDomain, SymmetricDistance>;

pub(crate) type DeferredTransformation =
    Transformation<DeferredDomain, SymmetricDistance, DeferredDomain, SymmetricDistance>;

/// A single local VPlan node and its corresponding table certificate.
/// The node must have Input as its child, so applying it only appends one
/// operator to an existing bound query. This is the native erasure boundary.
struct CertifiedDeferredNode {
    step: VPlan,
    table: TableTransformation,
}

impl CertifiedTransformationParts<
    DeferredDomain, SymmetricDistance, DeferredDomain, SymmetricDistance,
> for CertifiedDeferredNode {
    fn into_parts(
        self, input_domain: &DeferredDomain, output_domain: &DeferredDomain,
    ) -> Fallible<(
        Function<Deferred<VTable>, Deferred<VTable>>,
        StabilityMap<SymmetricDistance, SymmetricDistance>,
    )> {
        if self.table.input_domain != input_domain.cast_carrier::<VTable>()
            || self.table.output_domain != output_domain.cast_carrier::<VTable>()
        {
            return fallible!(MakeTransformation,
                "deferred node certificate does not match realized frame domains");
        }

        // Reuse kernel plan builders rather than defining a second Node enum.
        // The reference interpreter and contraction theorem concern these
        // exact VPlan variants; arbitrary subtrees are rejected here.
        let function = match self.step {
            VPlan::Input => Function::new(|query: &Deferred<VTable>| query.clone()),
            VPlan::Filter { input, predicate } if matches!(*input, VPlan::Input) =>
                Function::new(move |query: &Deferred<VTable>| {
                    query.clone().filter(predicate.clone())
                }),
            VPlan::Project { input, exprs } if matches!(*input, VPlan::Input) =>
                Function::new(move |query: &Deferred<VTable>| {
                    query.clone().project(exprs.clone())
                }),
            _ => return fallible!(MakeTransformation,
                "deferred node must be an individual operator"),
        };
        Ok((function, self.table.stability_map.clone()))
    }
}

fn from_table(
    input_domain: DeferredDomain,
    metric: SymmetricDistance,
    step: VPlan,
    table: TableTransformation,
) -> Fallible<DeferredTransformation> {
    let output_domain = table.output_domain.cast_carrier::<Deferred<VTable>>();
    Transformation::new_certified(
        input_domain, metric.clone(), output_domain, metric,
        CertifiedDeferredNode { step, table },
    )
}

fn make_input(
    domain: DeferredDomain, metric: SymmetricDistance,
) -> Fallible<DeferredTransformation> {
    let table = input::make_stable_vtable_input(
        domain.cast_carrier::<VTable>(), metric.clone(),
    )?;
    from_table(domain, metric, VPlan::Input, table)
}

fn make_filter(
    domain: DeferredDomain, metric: SymmetricDistance, predicate: VExpr,
) -> Fallible<DeferredTransformation> {
    let table = filter::make_stable_vtable_filter(
        domain.cast_carrier::<VTable>(), metric.clone(), predicate.clone(),
    )?;
    let step = VPlan::Filter { input: Box::new(VPlan::Input), predicate };
    from_table(domain, metric, step, table)
}

fn make_project(
    domain: DeferredDomain, metric: SymmetricDistance, exprs: Vec<VExpr>,
) -> Fallible<DeferredTransformation> {
    let table = project::make_stable_vtable_project(
        domain.cast_carrier::<VTable>(), metric.clone(), exprs.clone(),
    )?;
    let step = VPlan::Project { input: Box::new(VPlan::Input), exprs };
    from_table(domain, metric, step, table)
}

/// Compose real OpenDP transformations; invocation only extends the VPlan.
/// Certified here for the reference VTable source and scalar SymmetricDistance.
pub(crate) fn make_stable_deferred(
    domain: DeferredDomain, metric: SymmetricDistance, plan: VPlan,
) -> Fallible<DeferredTransformation> {
    match plan {
        VPlan::Input => make_input(domain, metric),
        VPlan::Filter { input: child, predicate } => {
            let prior = make_stable_deferred(domain, metric, *child)?;
            let (domain, metric) = prior.output_space();
            let step = make_filter(domain, metric, predicate)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
        VPlan::Project { input: child, exprs } => {
            let prior = make_stable_deferred(domain, metric, *child)?;
            let (domain, metric) = prior.output_space();
            let step = make_project(domain, metric, exprs)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::Domain,
        domains::{AtomDomain, ColumnDomain, OptionDomain},
        verified_query::{VDType, VField, VValue},
    };

    fn source() -> VTable {
        VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: false },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(7)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(3)), Some(VValue::Bool(false))],
                vec![Some(VValue::Int(1)), None],
            ],
        }
    }

    fn domain() -> Fallible<DeferredDomain> {
        FrameDomain::new(vec![
            ColumnDomain::new("x", AtomDomain::<i64>::new_closed((0, 10))?),
            ColumnDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    fn plan() -> VPlan {
        VPlan::Project {
            input: Box::new(VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            }),
            exprs: vec![VExpr::Alias {
                expr: Box::new(VExpr::Column(0)),
                name: "selected".into(),
            }],
        }
    }

    #[test]
    fn stable_composition_builds_lazy_plan() -> Fallible<()> {
        let transformation = make_stable_deferred(domain()?, SymmetricDistance, plan())?;
        let bound = Deferred::new(source());
        let output = transformation.invoke(&bound)?;
        let VPlan::Project { input, exprs } = &output.plan else {
            panic!("expected project");
        };
        assert_eq!(exprs, &vec![VExpr::Alias {
            expr: Box::new(VExpr::Column(0)),
            name: "selected".into(),
        }]);
        let VPlan::Filter { input, predicate } = &**input else {
            panic!("expected filter");
        };
        assert_eq!(*predicate, VExpr::Column(1));
        assert!(matches!(&**input, VPlan::Input));
        for distance in [0, 1, 3, u32::MAX] {
            assert_eq!(transformation.map(&distance)?, distance);
        }
        let collected = output.apply()
            .map_err(|_| err!(FailedFunction, "reference collection failed"))?;
        assert_eq!(collected.rows, vec![vec![Some(VValue::Int(7))]]);
        assert!(transformation.output_domain.member(&output)?);
        Ok(())
    }

    #[test]
    fn no_eager_execution_even_for_malformed_reference_input() -> Fallible<()> {
        let transformation = make_stable_deferred(domain()?, SymmetricDistance, plan())?;
        let bad = VTable {
            schema: vec![VField { dtype: VDType::Boolean, nullable: false }],
            rows: vec![vec![Some(VValue::Int(4))]],
        };
        // Applying a lazy operator need not read or validate the source.
        // A malformed source is outside the domain, so apply() rejects it.
        let output = transformation.invoke(&Deferred::new(bad))?;
        assert!(matches!(output.plan, VPlan::Project { .. }));
        assert!(output.apply().is_err());
        Ok(())
    }

    #[test]
    fn realized_domains_and_different_sources() -> Fallible<()> {
        let transformation = make_stable_deferred(domain()?, SymmetricDistance, plan())?;
        let original = Deferred::new(source());
        assert!(transformation.input_domain.member(&original)?);
        let selected = transformation.invoke(&original)?;
        assert!(transformation.output_domain.member(&selected)?);
        assert_eq!(selected.apply().unwrap().rows.len(), 1);
        let mut changed = source();
        changed.rows.push(vec![Some(VValue::Int(4)), Some(VValue::Bool(true))]);
        let other = transformation.invoke(&Deferred::new(changed))?;
        assert_eq!(other.apply().unwrap().rows.len(), 2);
        assert!(transformation.output_domain.member(&other)?);

        let mut invalid = source();
        invalid.rows[0][0] = Some(VValue::Int(11));
        assert!(!transformation.input_domain.member(&Deferred::new(invalid))?);
        Ok(())
    }

    #[test]
    fn validates_each_node_before_building() -> Fallible<()> {
        let bad_filter = VPlan::Filter {
            input: Box::new(VPlan::Input), predicate: VExpr::Column(0),
        };
        assert!(make_stable_deferred(domain()?, SymmetricDistance, bad_filter).is_err());
        let bad_project = VPlan::Project {
            input: Box::new(VPlan::Input), exprs: vec![VExpr::Column(99)],
        };
        assert!(make_stable_deferred(domain()?, SymmetricDistance, bad_project).is_err());
        Ok(())
    }
}
