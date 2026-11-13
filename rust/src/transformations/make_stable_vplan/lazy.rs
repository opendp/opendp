//! Certified node-level transformations over bound, backend-neutral plans.
//!
//! Each function only rewrites the plan and clones the opaque source handle.
//! Stability is inherited from the matching verified concrete-table operator.
//! The generic denotational lifting lemma is in lazy/proof.rs.

use std::marker::PhantomData;

use crate::{
    combinators::make_chain_tt_certified_symmetric,
    core::{CertifiedTransformationParts, Function, StabilityMap, Transformation},
    domains::{
        FrameDomain,
        verified_lazyframe::{VLazyFrame, VSource},
        verified_plan::VFrameDomain,
    },
    error::*,
    metrics::SymmetricDistance,
    verified_query::{VExpr, VPlan, VTable},
};

use super::{filter, input, project};

type LazyDomain<S> = FrameDomain<VLazyFrame<S>>;
type TableTransformation =
    Transformation<VFrameDomain, SymmetricDistance, VFrameDomain, SymmetricDistance>;

pub(crate) type LazyTransformation<S> =
    Transformation<LazyDomain<S>, SymmetricDistance, LazyDomain<S>, SymmetricDistance>;

enum Node {
    Input,
    Filter(VExpr),
    Project(Vec<VExpr>),
}

impl Node {
    fn apply(&self, input: VPlan) -> VPlan {
        match self {
            Self::Input => input,
            Self::Filter(predicate) => VPlan::Filter {
                input: Box::new(input),
                predicate: predicate.clone(),
            },
            Self::Project(exprs) => VPlan::Project {
                input: Box::new(input),
                exprs: exprs.clone(),
            },
        }
    }
}

/// This adapter binds the plan rewrite and the stability map to the *same*
/// certified VTable node. No arbitrary callbacks can enter this constructor.
/// The native correspondence between Node::apply and the verified evaluator
/// remains a small, explicit boundary, pending direct Verus-to-Arc contracts.
struct CertifiedLazyNode<S: VSource> {
    node: Node,
    table: TableTransformation,
    _source: PhantomData<S>,
}

impl<S: VSource> CertifiedTransformationParts<
    LazyDomain<S>, SymmetricDistance, LazyDomain<S>, SymmetricDistance,
> for CertifiedLazyNode<S> {
    fn into_parts(
        self,
        input_domain: &LazyDomain<S>,
        output_domain: &LazyDomain<S>,
    ) -> Fallible<(
        Function<VLazyFrame<S>, VLazyFrame<S>>,
        StabilityMap<SymmetricDistance, SymmetricDistance>,
    )> {
        if self.table.input_domain != input_domain.cast_carrier::<VTable>()
            || self.table.output_domain != output_domain.cast_carrier::<VTable>()
        {
            return fallible!(MakeTransformation,
                "lazy node certificate does not match realized frame domains");
        }
        let node = self.node;
        Ok((
            Function::new(move |input: &VLazyFrame<S>| VLazyFrame {
                plan: node.apply(input.plan.clone()),
                source: input.source.clone(),
            }),
            self.table.stability_map.clone(),
        ))
    }
}

fn from_table<S: VSource>(
    input_domain: LazyDomain<S>,
    metric: SymmetricDistance,
    node: Node,
    table: TableTransformation,
) -> Fallible<LazyTransformation<S>> {
    let output_domain = table.output_domain.cast_carrier::<VLazyFrame<S>>();
    Transformation::new_certified(
        input_domain, metric.clone(), output_domain, metric,
        CertifiedLazyNode { node, table, _source: PhantomData },
    )
}

fn make_input<S: VSource>(
    domain: LazyDomain<S>, metric: SymmetricDistance,
) -> Fallible<LazyTransformation<S>> {
    let table = input::make_stable_vtable_input(
        domain.cast_carrier::<VTable>(), metric.clone(),
    )?;
    from_table(domain, metric, Node::Input, table)
}

fn make_filter<S: VSource>(
    domain: LazyDomain<S>, metric: SymmetricDistance, predicate: VExpr,
) -> Fallible<LazyTransformation<S>> {
    let table = filter::make_stable_vtable_filter(
        domain.cast_carrier::<VTable>(), metric.clone(), predicate.clone(),
    )?;
    from_table(domain, metric, Node::Filter(predicate), table)
}

fn make_project<S: VSource>(
    domain: LazyDomain<S>, metric: SymmetricDistance, exprs: Vec<VExpr>,
) -> Fallible<LazyTransformation<S>> {
    let table = project::make_stable_vtable_project(
        domain.cast_carrier::<VTable>(), metric.clone(), exprs.clone(),
    )?;
    from_table(domain, metric, Node::Project(exprs), table)
}

/// Compose node transformations without ever collecting the source. The
/// output carrier contains a backend-neutral VPlan and the same source handle.
///
/// The first verified instance is the margin-free scalar symmetric metric.
/// Grouped FrameDistance bounds need a separate verified lifting theorem.
pub(crate) fn make_stable_vlazyframe<S: VSource>(
    domain: LazyDomain<S>, metric: SymmetricDistance, plan: VPlan,
) -> Fallible<LazyTransformation<S>> {
    match plan {
        VPlan::Input => make_input(domain, metric),
        VPlan::Filter { input: child, predicate } => {
            let prior = make_stable_vlazyframe(domain, metric, *child)?;
            let (domain, metric) = prior.output_space();
            let step = make_filter(domain, metric, predicate)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
        VPlan::Project { input: child, exprs } => {
            let prior = make_stable_vlazyframe(domain, metric, *child)?;
            let (domain, metric) = prior.output_space();
            let step = make_project(domain, metric, exprs)?;
            make_chain_tt_certified_symmetric(&step, &prior)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use crate::{
        core::Domain,
        domains::{AtomDomain, ColumnDomain, OptionDomain},
        verified_query::{VDType, VField, VValue},
    };

    #[derive(Clone)]
    struct CountingSource {
        table: Arc<VTable>,
        reads: Arc<AtomicUsize>,
    }

    impl VSource for CountingSource {
        fn snapshot(&self) -> Fallible<VTable> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok((*self.table).clone())
        }
    }

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

    fn domain<S: VSource>() -> Fallible<LazyDomain<S>> {
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
    fn no_materialization_during_construction_or_composition() -> Fallible<()> {
        let reads = Arc::new(AtomicUsize::new(0));
        let source = CountingSource {
            table: Arc::new(source()), reads: reads.clone(),
        };
        let transformation = make_stable_vlazyframe(
            domain::<CountingSource>()?, SymmetricDistance, plan(),
        )?;
        let bound = VLazyFrame::new(source.clone());
        let output = transformation.invoke(&bound)?;
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        let VPlan::Project { input, exprs } = &output.plan else {
            panic!("expected project plan");
        };
        assert_eq!(exprs, &vec![VExpr::Alias {
            expr: Box::new(VExpr::Column(0)),
            name: "selected".into(),
        }]);
        let VPlan::Filter { input, predicate } = &**input else {
            panic!("expected filter child");
        };
        assert_eq!(*predicate, VExpr::Column(1));
        assert!(matches!(&**input, VPlan::Input));
        assert!(Arc::ptr_eq(&output.source.table, &source.table));
        for distance in [0, 1, 3, u32::MAX] {
            assert_eq!(transformation.map(&distance)?, distance);
        }
        let collected = output.collect()?;
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        assert_eq!(collected.rows, vec![vec![Some(VValue::Int(7))]]);
        assert!(transformation.output_domain.member(&output)?);
        Ok(())
    }

    #[test]
    fn realized_domains_and_different_sources() -> Fallible<()> {
        let transformation = make_stable_vlazyframe(
            domain::<VTable>()?, SymmetricDistance, plan(),
        )?;
        let original = VLazyFrame::new(source());
        assert!(transformation.input_domain.member(&original)?);
        let selected = transformation.invoke(&original)?;
        assert!(transformation.output_domain.member(&selected)?);
        assert_eq!(selected.collect()?.rows.len(), 1);
        let mut changed = source();
        changed.rows.push(vec![Some(VValue::Int(4)), Some(VValue::Bool(true))]);
        let other = transformation.invoke(&VLazyFrame::new(changed))?;
        assert_eq!(other.collect()?.rows.len(), 2);
        assert!(transformation.output_domain.member(&other)?);

        let mut invalid = source();
        invalid.rows[0][0] = Some(VValue::Int(11));
        assert!(!transformation.input_domain.member(&VLazyFrame::new(invalid))?);
        Ok(())
    }

    #[test]
    fn validates_each_node_before_building() -> Fallible<()> {
        let bad_filter = VPlan::Filter {
            input: Box::new(VPlan::Input),
            predicate: VExpr::Column(0),
        };
        assert!(make_stable_vlazyframe(
            domain::<VTable>()?, SymmetricDistance, bad_filter,
        ).is_err());
        let bad_project = VPlan::Project {
            input: Box::new(VPlan::Input), exprs: vec![VExpr::Column(99)],
        };
        assert!(make_stable_vlazyframe(
            domain::<VTable>()?, SymmetricDistance, bad_project,
        ).is_err());
        Ok(())
    }
}
