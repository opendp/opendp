//! Grouped-distance stability lemmas for verified filters.
//!
//! Separate from the completed scalar SymmetricDistance constructor
//! certificate; the symbolic FrameDistance adapter is not yet fully verified.

use super::*;

verus! {

/// Pointwise contraction of the multiplicity difference for two datasets.
/// Each row's grouping key is unchanged by filtering, so this is the
/// per-row algebra underlying every grouped symmetric-distance bound.
pub proof fn lemma_filter_count_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    value: Seq<Option<VValue>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::row_in_schema(value, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) => {
                let a = left.to_multiset().count(value) as int;
                let b = right.to_multiset().count(value) as int;
                let x = out_left.to_multiset().count(value) as int;
                let y = out_right.to_multiset().count(value) as int;
                (if x >= y { x - y } else { y - x })
                    <= (if a >= b { a - b } else { b - a })
            },
            _ => false,
        },
{
    lemma_filter_exact_count(plan, left, schema, value);
    lemma_filter_exact_count(plan, right, schema, value);
}

/// Both insertions and deletions are accounted for by the absolute
/// difference of row multiplicities within each group.
pub proof fn lemma_group_distance_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    group: Seq<Option<VValue>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                frame_distance::group_distance(out_left.to_multiset(), out_right.to_multiset(),
                    values, group_by, group)
                <= frame_distance::group_distance(left.to_multiset(), right.to_multiset(),
                    values, group_by, group),
            _ => false,
        },
    decreases values.len(),
{
    if values.len() > 0 {
        stability::lemma_rows_drop_last_valid(values, schema);
        lemma_group_distance_contraction(plan, left, right, schema,
            values.drop_last(), group_by, group);
        lemma_filter_count_contraction(plan, left, right, schema, values.last());
    } else {
        verified::lemma_apply_prefix_type_sound(plan, left, schema, schema, left.len());
        verified::lemma_apply_prefix_type_sound(plan, right, schema, schema, right.len());
    }
}

/// A filter cannot create a changed group: the support of nonzero group
/// distances is contained in the original support.
pub proof fn lemma_changed_groups_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                frame_distance::changed_groups(out_left.to_multiset(), out_right.to_multiset(),
                    values, group_by, groups)
                <= frame_distance::changed_groups(left.to_multiset(), right.to_multiset(),
                    values, group_by, groups),
            _ => false,
        },
    decreases groups.len(),
{
    if groups.len() > 0 {
        lemma_changed_groups_contraction(plan, left, right, schema,
            values, group_by, groups.drop_last());
        lemma_group_distance_contraction(plan, left, right, schema,
            values, group_by, groups.last());
    } else {
        verified::lemma_apply_prefix_type_sound(plan, left, schema, schema, left.len());
        verified::lemma_apply_prefix_type_sound(plan, right, schema, schema, right.len());
    }
}


/// Specialize the grouped-distance certificate to an executable sequence
/// of well-typed VExpr keys. The kernel's verified project::eval_prefix
/// implements exactly project::row_prefix, which supplies this grouping.
pub proof fn lemma_filter_preserves_expr_group_bound(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    keys: Seq<VExpr>, values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
        verified::project::schema_prefix(keys, schema, keys.len()).is_Some(),
        frame_distance::within_group_bound(left.to_multiset(), right.to_multiset(), values,
            |row: Seq<Option<VValue>>|
                verified::project::row_prefix(keys, row, keys.len()).unwrap(),
            groups, per_group, num_groups),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                frame_distance::within_group_bound(out_left.to_multiset(), out_right.to_multiset(),
                    values,
                    |row: Seq<Option<VValue>>|
                        verified::project::row_prefix(keys, row, keys.len()).unwrap(),
                    groups, per_group, num_groups),
            _ => false,
        },
{
    // Every typed row has a total grouping key. The same kernel function
    // evaluates keys on both sides of the symmetric distance.
    assert forall|row: Seq<Option<VValue>>| verified::row_in_schema(row, schema)
        implies verified::project::row_prefix(keys, row, keys.len()).is_Ok() by {
        verified::project::lemma_row_prefix_type_sound(
            keys, schema, row, keys.len());
    }
    let by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>> =
        |row: Seq<Option<VValue>>|
            verified::project::row_prefix(keys, row, keys.len()).unwrap();
    lemma_changed_groups_contraction(
        plan, left, right, schema, values, by, groups);
    if let Some(limit) = per_group {
        assert forall|key: Seq<Option<VValue>>|
            frame_distance::group_distance(
                verified::apply_spec(plan, left).unwrap().to_multiset(),
                verified::apply_spec(plan, right).unwrap().to_multiset(),
                values, by, key) <= limit as nat by {
            lemma_group_distance_contraction(
                plan, left, right, schema, values, by, key);
        }
    }
}

/// Filtering preserves the concrete *kinds* of constraints retained by
/// filter_domain: column-level row restrictions, max_length, max_groups.
/// The source-relative Rust FrameDomain encoding is checked separately.
pub proof fn lemma_filter_transformation_contract(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    allowed: Set<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
        verified::project::schema_prefix(keys, schema, keys.len()).is_Some(),
        domain::bounded_frame_member(rows.to_multiset(), allowed,
            keys, values, groups, per_group, num_groups),
    ensures
        match verified::apply_spec(plan, rows) {
            Ok(out) => domain::bounded_frame_member(out.to_multiset(), allowed,
                keys, values, groups, per_group, num_groups),
            Err(_) => false,
        },
{
    lemma_filter_submultiset(plan, rows, schema);
    let output = verified::apply_spec(plan, rows).unwrap();
    domain::lemma_allowed_rows_subset(
        output.to_multiset(), rows.to_multiset(), allowed);
    domain::lemma_complete_group_support_subset(
        output.to_multiset(), rows.to_multiset(), keys, values, groups);
    assert(verified::rows_in_schema(Seq::<Seq<Option<VValue>>>::empty(), schema));
    lemma_filter_preserves_expr_group_bound(
        plan, rows, Seq::empty(), schema, keys, values,
        groups, per_group, num_groups);
}


impl CertifiedTableFilter {
    /// Full margin-free frame contract: filtering preserves every row-wise
    /// value restriction, and contracts the global multiset L1 distance.
    /// This proves the identity stability map for arbitrary distances, not
    /// just unit insertions and deletions.
    pub proof fn lemma_domain_and_global_stability(
        &self,
        left: Seq<Seq<Option<VValue>>>,
        right: Seq<Seq<Option<VValue>>>,
        values: Seq<Seq<Option<VValue>>>,
        allowed: Set<Seq<Option<VValue>>>,
    )
        requires
            self.valid(),
            verified::rows_in_schema(left, self.schema_spec()),
            verified::rows_in_schema(right, self.schema_spec()),
            verified::rows_in_schema(values, self.schema_spec()),
            domain::allowed_rows(left.to_multiset(), allowed),
            domain::allowed_rows(right.to_multiset(), allowed),
        ensures
            match (
                self.result_spec(left),
                self.result_spec(right),
            ) {
                (Ok(out_left), Ok(out_right)) => {
                    &&& verified::rows_in_schema(out_left, self.schema_spec())
                    &&& verified::rows_in_schema(out_right, self.schema_spec())
                    &&& domain::allowed_rows(out_left.to_multiset(), allowed)
                    &&& domain::allowed_rows(out_right.to_multiset(), allowed)
                    &&& frame_distance::group_distance(
                        out_left.to_multiset(), out_right.to_multiset(), values,
                        |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                        Seq::<Option<VValue>>::empty(),
                    ) <= frame_distance::group_distance(
                        left.to_multiset(), right.to_multiset(), values,
                        |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                        Seq::<Option<VValue>>::empty(),
                    )
                },
                _ => false,
            },
    {
        lemma_single_filter_relation(&self.plan, left);
        lemma_single_filter_relation(&self.plan, right);
        verified::lemma_relation_type_sound(&self.plan, left, self.schema@, self.schema@);
        verified::lemma_relation_type_sound(&self.plan, right, self.schema@, self.schema@);
        let out_left = verified::relation_spec(&self.plan, left).unwrap();
        let out_right = verified::relation_spec(&self.plan, right).unwrap();
        lemma_filter_submultiset(&self.plan, left, self.schema@);
        lemma_filter_submultiset(&self.plan, right, self.schema@);
        assert(out_left.to_multiset().subset_of(left.to_multiset()));
        assert(out_right.to_multiset().subset_of(right.to_multiset()));
        domain::lemma_allowed_rows_subset(out_left.to_multiset(), left.to_multiset(), allowed);
        domain::lemma_allowed_rows_subset(out_right.to_multiset(), right.to_multiset(), allowed);
        assert(verified::local_schema(&self.plan, self.schema@) == Some(self.schema@));
        lemma_group_distance_contraction(
            &self.plan, left, right, self.schema@, values,
            |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
            Seq::<Option<VValue>>::empty(),
        );
    }

}

} // verus!
