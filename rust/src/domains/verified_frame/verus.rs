//! Executable, verified row-wise membership for the first table transformation.
//! Bounds here are inclusive. The Rust adapter rejects other bound shapes.

use super::*;
use vstd::multiset::Multiset;

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq, StructuralEq)]
pub enum VCellRange {
    Int { lower: Option<i64>, upper: Option<i64> },
    Bool { min: Option<bool>, max: Option<bool> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, StructuralEq)]
pub struct VColumnRule {
    pub nullable: bool,
    pub range: VCellRange,
}

pub open spec fn rule_field(rule: VColumnRule) -> VField {
    VField {
        dtype: match rule.range {
            VCellRange::Int { .. } => VDType::Int64,
            VCellRange::Bool { .. } => VDType::Boolean,
        },
        nullable: rule.nullable,
    }
}

pub open spec fn cell_member(cell: Option<VValue>, rule: VColumnRule) -> bool {
    match (cell, rule.range) {
        (None, _) => rule.nullable,
        (Some(VValue::Int(x)), VCellRange::Int { lower, upper }) =>
            (lower.is_None() || x >= lower.unwrap())
                && (upper.is_None() || x <= upper.unwrap()),
        (Some(VValue::Bool(x)), VCellRange::Bool { min, max }) =>
            (min.is_None() || !min.unwrap() || x)
                && (max.is_None() || max.unwrap() || !x),
        _ => false,
    }
}

/// A derived expression has no value-range promise beyond its dtype and nullability.
pub open spec fn unbounded_rule(rule: VColumnRule) -> bool {
    match rule.range {
        VCellRange::Int { lower, upper } => lower.is_None() && upper.is_None(),
        VCellRange::Bool { min, max } => min.is_None() && max.is_None(),
    }
}

pub fn check_unbounded_rule(rule: VColumnRule) -> (valid: bool)
    ensures valid == unbounded_rule(rule),
{
    match rule.range {
        VCellRange::Int { lower, upper } => lower.is_none() && upper.is_none(),
        VCellRange::Bool { min, max } => min.is_none() && max.is_none(),
    }
}

/// Type correctness is sufficient for membership in an unbounded column.
pub proof fn lemma_unbounded_cell(cell: Option<VValue>, rule: VColumnRule)
    requires
        unbounded_rule(rule),
        cell_matches_field(cell, rule_field(rule)),
    ensures cell_member(cell, rule),
{
    match (cell, rule.range) {
        (None, _) => {},
        (Some(VValue::Int(_)), VCellRange::Int { .. }) => {},
        (Some(VValue::Bool(_)), VCellRange::Bool { .. }) => {},
        _ => { assert(false); },
    }
}

pub open spec fn row_member(
    row: Seq<Option<VValue>>, rules: Seq<VColumnRule>,
) -> bool {
    row.len() == rules.len()
        && forall|i: int| 0 <= i < rules.len() ==> cell_member(row[i], rules[i])
}

pub open spec fn schema_member(
    schema: Seq<VField>, rules: Seq<VColumnRule>,
) -> bool {
    schema.len() == rules.len()
        && forall|i: int| 0 <= i < rules.len() ==> schema[i] == rule_field(rules[i])
}

pub open spec fn table_member(
    table: &VTable, rules: Seq<VColumnRule>,
) -> bool {
    schema_member(table.schema@, rules)
        && forall|i: int| 0 <= i < table.rows@.len()
            ==> row_member(table.rows@[i]@, rules)
}

/// A rule set determines exactly one schema.
pub proof fn lemma_unique_schema(
    first: Seq<VField>, second: Seq<VField>, rules: Seq<VColumnRule>,
)
    requires schema_member(first, rules), schema_member(second, rules),
    ensures first =~= second,
{
    assert(first.len() == second.len());
    assert forall|i: int| 0 <= i < first.len()
        implies first[i] == second[i] by {
        assert(first[i] == rule_field(rules[i]));
        assert(second[i] == rule_field(rules[i]));
    }
}

/// This is the actual membership predicate used by the OpenDP domain bridge.
pub fn check_cell(cell: Option<VValue>, rule: VColumnRule) -> (valid: bool)
    ensures valid == cell_member(cell, rule),
{
    match (cell, rule.range) {
        (None, _) => rule.nullable,
        (Some(VValue::Int(x)), VCellRange::Int { lower, upper }) =>
            (lower.is_none() || x >= lower.unwrap())
                && (upper.is_none() || x <= upper.unwrap()),
        (Some(VValue::Bool(x)), VCellRange::Bool { min, max }) =>
            (min.is_none() || !min.unwrap() || x)
                && (max.is_none() || max.unwrap() || !x),
        _ => false,
    }
}

pub fn check_row(row: &Vec<Option<VValue>>, rules: &Vec<VColumnRule>)
    -> (valid: bool)
    ensures valid == row_member(row@, rules@),
{
    if row.len() != rules.len() { return false; }
    let mut i = 0;
    while i < row.len()
        invariant
            0 <= i <= row.len(),
            row.len() == rules.len(),
            forall|j: int| 0 <= j < i ==> cell_member(row@[j], rules@[j]),
        decreases row.len() - i,
    {
        if !check_cell(row[i], rules[i]) { return false; }
        i += 1;
    }
    true
}

pub fn check_table(table: &VTable, rules: &Vec<VColumnRule>)
    -> (valid: bool)
    ensures valid == table_member(table, rules@),
{
    if table.schema.len() != rules.len() { return false; }
    let mut j = 0;
    while j < rules.len()
        invariant
            0 <= j <= rules.len(),
            rules.len() == table.schema.len(),
            forall|k: int| 0 <= k < j
                ==> table.schema@[k] == rule_field(rules@[k]),
        decreases rules.len() - j,
    {
        let rule = rules[j];
        let field = VField {
            dtype: match rule.range {
                VCellRange::Int { .. } => VDType::Int64,
                VCellRange::Bool { .. } => VDType::Boolean,
            },
            nullable: rule.nullable,
        };
        if table.schema[j] != field { return false; }
        proof { assert(field == rule_field(rules@[j as int])); }
        j += 1;
    }
    let mut i = 0;
    while i < table.rows.len()
        invariant
            0 <= i <= table.rows.len(),
            schema_member(table.schema@, rules@),
            forall|k: int| 0 <= k < i
                ==> row_member(table.rows@[k]@, rules@),
        decreases table.rows.len() - i,
    {
        if !check_row(&table.rows[i], rules) { return false; }
        i += 1;
    }
    true
}

/// Build a schema from the exact rule metadata used to check membership.
pub fn rule_schema(rules: &Vec<VColumnRule>) -> (schema: Vec<VField>)
    ensures schema_member(schema@, rules@),
{
    let mut schema = Vec::new();
    let mut i: usize = 0;
    while i < rules.len()
        invariant
            0 <= i <= rules.len(),
            schema@.len() == i,
            forall|j: int| 0 <= j < i
                ==> schema@[j] == rule_field(rules@[j]),
        decreases rules.len() - i,
    {
        let rule = rules[i];
        schema.push(VField {
            dtype: match rule.range {
                VCellRange::Int { .. } => VDType::Int64,
                VCellRange::Bool { .. } => VDType::Boolean,
            },
            nullable: rule.nullable,
        });
        i += 1;
    }
    schema
}

/// Membership in a column rule entails the kernel's type and nullability rule.
pub proof fn lemma_cell_typed(cell: Option<VValue>, rule: VColumnRule)
    requires cell_member(cell, rule),
    ensures cell_matches_field(cell, rule_field(rule)),
{
    match (cell, rule.range) {
        (None, _) => {},
        (Some(VValue::Int(_)), VCellRange::Int { .. }) => {},
        (Some(VValue::Bool(_)), VCellRange::Bool { .. }) => {},
        _ => { assert(false); },
    }
}

pub proof fn lemma_row_typed(
    row: Seq<Option<VValue>>, schema: Seq<VField>, rules: Seq<VColumnRule>,
)
    requires row_member(row, rules), schema_member(schema, rules),
    ensures row_in_schema(row, schema),
{
    assert(row.len() == schema.len());
    assert forall|i: int| 0 <= i < row.len()
        implies cell_matches_field(row[i], schema[i]) by {
        lemma_cell_typed(row[i], rules[i]);
        assert(schema[i] == rule_field(rules[i]));
    }
}

/// Membership of a sequence of rows under common column rules.
pub open spec fn rows_member(
    rows: Seq<Seq<Option<VValue>>>, rules: Seq<VColumnRule>,
) -> bool {
    forall|i: int| 0 <= i < rows.len() ==> row_member(rows[i], rules)
}

/// A VTable belongs to its frame domain exactly when its schema matches
/// and its row sequence belongs to the corresponding row domain.
pub proof fn lemma_table_rows_domain(
    table: &VTable, rules: Seq<VColumnRule>,
)
    ensures table_member(table, rules)
        <==> (schema_member(table.schema@, rules)
            && rows_member(rows_view(table.rows@), rules)),
{
    let rows = rows_view(table.rows@);
    assert(rows.len() == table.rows@.len());
    assert forall|i: int| 0 <= i < table.rows@.len()
        implies rows[i] == table.rows@[i]@ by { }
    if table_member(table, rules) {
        assert forall|i: int| 0 <= i < rows.len()
            implies row_member(rows[i], rules) by {
            assert(row_member(table.rows@[i]@, rules));
        }
    } else if schema_member(table.schema@, rules)
        && rows_member(rows, rules)
    {
        assert forall|i: int| 0 <= i < table.rows@.len()
            implies row_member(table.rows@[i]@, rules) by {
            assert(row_member(rows[i], rules));
        }
    }
}

pub proof fn lemma_rows_member_typed(
    rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>,
    rules: Seq<VColumnRule>,
)
    requires schema_member(schema, rules), rows_member(rows, rules),
    ensures rows_in_schema(rows, schema),
{
    assert forall|i: int| 0 <= i < rows.len()
        implies row_in_schema(rows[i], schema) by {
        lemma_row_typed(rows[i], schema, rules);
    }
}

pub proof fn lemma_table_typed(table: &VTable, rules: Seq<VColumnRule>)
    requires table_member(table, rules),
    ensures table_valid(table),
{
    assert forall|i: int| 0 <= i < table.rows@.len()
        implies row_in_schema(rows_view(table.rows@)[i], table.schema@) by {
        lemma_row_typed(table.rows@[i]@, table.schema@, rules);
    }
}

pub proof fn lemma_rows_subset(
    smaller: Multiset<Seq<Option<VValue>>>,
    larger: Multiset<Seq<Option<VValue>>>,
    rules: Seq<VColumnRule>,
)
    requires
        smaller.subset_of(larger),
        forall|row: Seq<Option<VValue>>|
            larger.count(row) > 0 ==> row_member(row, rules),
    ensures
        forall|row: Seq<Option<VValue>>|
            smaller.count(row) > 0 ==> row_member(row, rules),
{
    assert forall|row: Seq<Option<VValue>>|
        smaller.count(row) > 0 implies row_member(row, rules) by {
        assert(smaller.count(row) <= larger.count(row));
    }
}


/// Filtering a table cannot violate a row-local domain: every output row
/// must already occur in its input, and schema is unchanged.
pub proof fn lemma_table_submultiset(
    input: &VTable, output: &VTable, rules: Seq<VColumnRule>,
)
    requires
        table_member(input, rules),
        output.schema@ == input.schema@,
        rows_view(output.rows@).to_multiset()
            .subset_of(rows_view(input.rows@).to_multiset()),
    ensures table_member(output, rules),
{
    let before = rows_view(input.rows@);
    let after = rows_view(output.rows@);
    before.to_multiset_ensures();
    after.to_multiset_ensures();
    assert(schema_member(output.schema@, rules));
    assert forall|i: int| 0 <= i < output.rows@.len()
        implies row_member(output.rows@[i]@, rules) by {
        let row = output.rows@[i]@;
        assert(after[i] == row);
        assert(after.contains(row));
        assert(after.to_multiset().count(row) > 0);
        assert(before.to_multiset().count(row) > 0);
        assert(before.contains(row));
        let index = choose|j: int| 0 <= j < before.len() && before[j] == row;
        assert(before[index] == input.rows@[index]@);
        assert(row_member(input.rows@[index]@, rules));
    }
}

/// Row-wise column restrictions can be represented by an allowed set of
/// complete rows. Removing rows preserves every such restriction.
pub open spec fn allowed_rows(
    rows: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
) -> bool {
    forall|row: Seq<Option<VValue>>|
        rows.count(row) > 0 ==> allowed.contains(row)
}

pub proof fn lemma_allowed_rows_subset(
    smaller: Multiset<Seq<Option<VValue>>>,
    larger: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
)
    requires
        smaller.subset_of(larger),
        allowed_rows(larger, allowed),
    ensures allowed_rows(smaller, allowed),
{
    assert forall|row: Seq<Option<VValue>>|
        smaller.count(row) > 0 implies allowed.contains(row) by {
        assert(smaller.count(row) <= larger.count(row));
    }
}

/// A fixed, distinct enumeration of row values and their grouping keys.
pub open spec fn group_universe(
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> bool {
    &&& forall|i: int, j: int|
        0 <= i < j < values.len() ==> values[i] != values[j]
    &&& forall|i: int| 0 <= i < values.len() ==> exists|j: int|
        0 <= j < groups.len()
        && groups[j] == project::row_prefix(
            keys, #[trigger] values[i], keys.len()).unwrap()
    &&& forall|i: int, j: int|
        0 <= i < j < groups.len() ==> groups[i] != groups[j]
}

/// The table-dependent part of completeness: every present row is enumerated.
pub open spec fn rows_covered(
    rows: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
) -> bool {
    forall|row: Seq<Option<VValue>>|
        rows.count(row) > 0 ==> exists|i: int|
            0 <= i < values.len() && values[i] == row
}

pub open spec fn complete_group_support(
    rows: Multiset<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> bool {
    group_universe(keys, values, groups) && rows_covered(rows, values)
}

pub proof fn lemma_complete_group_support_subset(
    smaller: Multiset<Seq<Option<VValue>>>,
    larger: Multiset<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
)
    requires smaller.subset_of(larger),
        complete_group_support(larger, keys, values, groups),
    ensures complete_group_support(smaller, keys, values, groups),
{
    assert forall|row: Seq<Option<VValue>>|
        smaller.count(row) > 0 implies exists|i: int|
            0 <= i < values.len() && values[i] == row by {
        assert(smaller.count(row) <= larger.count(row));
    }
    assert(rows_covered(smaller, values));
    assert(group_universe(keys, values, groups));
}

/// The portion of concrete FrameDomain<VTable>::member that can constrain
/// a filtered table: row-wise column domains and upper-bound margins.
/// For an empty set of keys, the margin has one group even on empty data.
/// The finite row/key enumerations must cover the support to interpret
/// the numerical bounds as actual group counts.
pub open spec fn bounded_frame_member(
    rows: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
) -> bool {
    &&& allowed_rows(rows, allowed)
    &&& complete_group_support(rows, keys, values, groups)
    &&& (keys.len() != 0 || num_groups != Some(0u32))
    &&& super::frame_distance::within_group_bound(rows,
        Seq::<Seq<Option<VValue>>>::empty().to_multiset(),
        values,
        |row: Seq<Option<VValue>>|
            project::row_prefix(keys, row, keys.len()).unwrap(),
        groups, per_group, num_groups)
}


} // verus!
