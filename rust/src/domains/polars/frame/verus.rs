//! Verified compatibility of FrameDomain<VTable> and SymmetricDistance.
//! Kept beside the FrameDomain MetricSpace implementations in mod.rs.
//! These laws cover the VTable carrier and the corresponding row sequences;
//! they do not imply verification of every Frame or MicrodataMetric.

use super::*;
use super::domain::*;
use super::{metric_space, symmetric_distance};

verus! {

/// Metric-space interpretation of the domain of typed row sequences.
/// Permuting rows does not change symmetric distance.
pub open spec fn symmetric_rows_space(rules: Seq<VColumnRule>) -> bool {
    metric_space::valid_metric_space::<Seq<Seq<Option<VValue>>>, u32>(
        |rows: Seq<Seq<Option<VValue>>>| rows_member(rows, rules),
        |a: Seq<Seq<Option<VValue>>>, b: Seq<Seq<Option<VValue>>>|
            a.to_multiset() =~= b.to_multiset(),
        |a: Seq<Seq<Option<VValue>>>, b: Seq<Seq<Option<VValue>>>|
            symmetric_distance::distance(a.to_multiset(), b.to_multiset()),
        |a: Seq<Seq<Option<VValue>>>, b: Seq<Seq<Option<VValue>>>, d: u32|
            symmetric_distance::close(a.to_multiset(), b.to_multiset(), d),
        |exact: nat, d: u32| exact <= d as nat,
    )
}

/// Same metric space, this time on the actual OpenDP VTable carrier.
/// Domain membership additionally checks the schema against the rules.
pub open spec fn symmetric_table_space(rules: Seq<VColumnRule>) -> bool {
    metric_space::valid_metric_space::<VTable, u32>(
        |table: VTable| table_member(&table, rules),
        |a: VTable, b: VTable|
            (a.schema@ =~= b.schema@)
                && (rows_view(a.rows@).to_multiset()
                    =~= rows_view(b.rows@).to_multiset()),
        |a: VTable, b: VTable|
            symmetric_distance::distance(
                rows_view(a.rows@).to_multiset(),
                rows_view(b.rows@).to_multiset()),
        |a: VTable, b: VTable, d: u32|
            symmetric_distance::close(
                rows_view(a.rows@).to_multiset(),
                rows_view(b.rows@).to_multiset(), d),
        |exact: nat, d: u32| exact <= d as nat,
    )
}

pub proof fn lemma_symmetric_rows_space(rules: Seq<VColumnRule>)
    ensures symmetric_rows_space(rules),
{
    assert forall|a: Seq<Seq<Option<VValue>>>|
        #[trigger] rows_member(a, rules)
        implies symmetric_distance::distance(
            a.to_multiset(), a.to_multiset()) == 0 by {
        symmetric_distance::lemma_reflexive(a.to_multiset());
    }
    assert forall|a: Seq<Seq<Option<VValue>>>,
                  b: Seq<Seq<Option<VValue>>>|
        #[trigger] rows_member(a, rules)
            && #[trigger] rows_member(b, rules)
        implies
            symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                == symmetric_distance::distance(b.to_multiset(), a.to_multiset())
            && (symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                == 0 <==> a.to_multiset() =~= b.to_multiset()) by {
        symmetric_distance::lemma_symmetric(a.to_multiset(), b.to_multiset());
        symmetric_distance::lemma_separating(a.to_multiset(), b.to_multiset());
    }
    assert forall|a: Seq<Seq<Option<VValue>>>,
                  b: Seq<Seq<Option<VValue>>>,
                  c: Seq<Seq<Option<VValue>>>|
        #[trigger] rows_member(a, rules)
            && #[trigger] rows_member(b, rules)
            && #[trigger] rows_member(c, rules)
        implies symmetric_distance::distance(a.to_multiset(), c.to_multiset())
            <= symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                + symmetric_distance::distance(b.to_multiset(), c.to_multiset()) by {
        symmetric_distance::lemma_triangle(
            a.to_multiset(), b.to_multiset(), c.to_multiset());
    }
}

pub proof fn lemma_symmetric_table_space(rules: Seq<VColumnRule>)
    ensures symmetric_table_space(rules),
{
    assert forall|a: VTable| #[trigger] table_member(&a, rules)
        implies symmetric_distance::distance(
            rows_view(a.rows@).to_multiset(),
            rows_view(a.rows@).to_multiset()) == 0 by {
        symmetric_distance::lemma_reflexive(
            rows_view(a.rows@).to_multiset());
    }
    assert forall|a: VTable, b: VTable|
        #[trigger] table_member(&a, rules)
            && #[trigger] table_member(&b, rules)
        implies {
            &&& symmetric_distance::distance(
                rows_view(a.rows@).to_multiset(),
                rows_view(b.rows@).to_multiset())
                == symmetric_distance::distance(
                    rows_view(b.rows@).to_multiset(),
                    rows_view(a.rows@).to_multiset())
            &&& (symmetric_distance::distance(
                rows_view(a.rows@).to_multiset(),
                rows_view(b.rows@).to_multiset())
                == 0 <==> (a.schema@ =~= b.schema@
                    && rows_view(a.rows@).to_multiset()
                        =~= rows_view(b.rows@).to_multiset()))
        } by {
        lemma_unique_schema(a.schema@, b.schema@, rules);
        symmetric_distance::lemma_symmetric(
            rows_view(a.rows@).to_multiset(),
            rows_view(b.rows@).to_multiset());
        symmetric_distance::lemma_separating(
            rows_view(a.rows@).to_multiset(),
            rows_view(b.rows@).to_multiset());
    }
    assert forall|a: VTable, b: VTable, c: VTable|
        #[trigger] table_member(&a, rules)
            && #[trigger] table_member(&b, rules)
            && #[trigger] table_member(&c, rules)
        implies symmetric_distance::distance(
            rows_view(a.rows@).to_multiset(), rows_view(c.rows@).to_multiset())
            <= symmetric_distance::distance(
                rows_view(a.rows@).to_multiset(), rows_view(b.rows@).to_multiset())
                + symmetric_distance::distance(
                rows_view(b.rows@).to_multiset(), rows_view(c.rows@).to_multiset()) by {
        symmetric_distance::lemma_triangle(
            rows_view(a.rows@).to_multiset(),
            rows_view(b.rows@).to_multiset(),
            rows_view(c.rows@).to_multiset());
    }
}

} // verus!
