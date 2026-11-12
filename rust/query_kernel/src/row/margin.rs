//! Executable margin checks, with an exact Verus model of group counts.
//! Grouping expressions are evaluated once per row. Key comparisons remain
//! quadratic in the worst case; this is a verified reference implementation.

use super::*;

verus! {

/// Structural comparison of the kernel's optional integer/Boolean cells.
pub fn same_cell(a: Option<VValue>, b: Option<VValue>) -> (equal: bool)
    ensures equal == (a == b),
{
    match (a, b) {
        (None, None) => true,
        (Some(VValue::Int(x)), Some(VValue::Int(y))) => x == y,
        (Some(VValue::Bool(x)), Some(VValue::Bool(y))) => x == y,
        _ => false,
    }
}

pub open spec fn key_equal(
    a: Seq<Option<VValue>>, b: Seq<Option<VValue>>,
) -> bool {
    a.len() == b.len()
        && forall|i: int| 0 <= i < a.len() ==> a[i] == b[i]
}

/// Compare one prefix without relying on a generic Rust PartialEq contract.
pub fn same_key_prefix(
    a: &Vec<Option<VValue>>, b: &Vec<Option<VValue>>, n: usize,
) -> (equal: bool)
    requires n <= a.len() && n <= b.len(),
    ensures equal == (forall|i: int| 0 <= i < n ==> a@[i] == b@[i]),
    decreases n,
{
    if n == 0 {
        true
    } else {
        let prior = same_key_prefix(a, b, n - 1);
        let last = same_cell(a[n - 1], b[n - 1]);
        proof {
            assert((forall|i: int| 0 <= i < n ==> a@[i] == b@[i])
                == ((forall|i: int| 0 <= i < n - 1 ==> a@[i] == b@[i])
                    && a@[(n - 1) as int] == b@[(n - 1) as int]));
        }
        prior && last
    }
}

pub fn same_key(a: &Vec<Option<VValue>>, b: &Vec<Option<VValue>>) -> (equal: bool)
    ensures equal == key_equal(a@, b@),
{
    if a.len() != b.len() { false }
    else { same_key_prefix(a, b, a.len()) }
}

/// Evaluate grouping expressions exactly once per input row.
pub open spec fn keys_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>, n: nat,
) -> Result<Seq<Seq<Option<VValue>>>, ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 { Ok(Seq::empty()) }
    else {
        match (
            keys_spec(rows, by, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok(prefix), Ok(key)) => Ok(prefix.push(key)),
            _ => Err(()),
        }
    }
}

pub fn eval_keys(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>, n: usize,
) -> (result: Result<Vec<Vec<Option<VValue>>>, ()>)
    requires n <= rows.len(),
    ensures
        match result {
            Ok(keys) => keys_spec(rows@, by@, n as nat)
                == Ok(rows_view(keys@)),
            Err(_) => keys_spec(rows@, by@, n as nat).is_Err(),
        },
        result.is_Ok() ==> result.unwrap().len() == n,
    decreases n,
{
    if n == 0 {
        Ok(Vec::new())
    } else {
        let mut prior = match eval_keys(rows, by, n - 1) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let key = match project::eval_prefix(by, &rows[n - 1], by.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let ghost old_keys = prior@;
        proof { lemma_rows_view_push(old_keys, key); }
        prior.push(key);
        Ok(prior)
    }
}

/// Count a grouping key in a prefix of already evaluated keys.
pub open spec fn count_key_spec(
    keys: Seq<Seq<Option<VValue>>>, key: Seq<Option<VValue>>, n: nat,
) -> nat
    recommends n <= keys.len(),
    decreases n,
{
    if n == 0 { 0 }
    else {
        count_key_spec(keys, key, (n - 1) as nat)
            + if key_equal(keys[(n - 1) as int], key) { 1nat } else { 0nat }
    }
}

pub fn count_key(
    keys: &Vec<Vec<Option<VValue>>>, key: &Vec<Option<VValue>>, n: usize,
) -> (count: usize)
    requires n <= keys.len(),
    ensures
        count as nat == count_key_spec(rows_view(keys@), key@, n as nat),
        count <= n,
    decreases n,
{
    if n == 0 { 0 }
    else {
        let prior = count_key(keys, key, n - 1);
        let matches = same_key(&keys[n - 1], key);
        prior + if matches { 1 } else { 0 }
    }
}

/// One scan gives both cardinality margins. Counting uses cached keys.
pub open spec fn group_stats_spec(
    keys: Seq<Seq<Option<VValue>>>, n: nat,
) -> (nat, nat)
    recommends n <= keys.len(),
    decreases n,
{
    if n == 0 { (0, 0) }
    else {
        let (groups, largest) = group_stats_spec(keys, (n - 1) as nat);
        let earlier = count_key_spec(keys, keys[(n - 1) as int], (n - 1) as nat);
        (
            groups + if earlier == 0 { 1nat } else { 0nat },
            if earlier + 1 > largest { earlier + 1 } else { largest },
        )
    }
}

pub fn group_stats(
    keys: &Vec<Vec<Option<VValue>>>, n: usize,
) -> (stats: (usize, usize))
    requires n <= keys.len(),
    ensures
        (stats.0 as nat, stats.1 as nat)
            == group_stats_spec(rows_view(keys@), n as nat),
        stats.0 <= n && stats.1 <= n,
    decreases n,
{
    if n == 0 { (0, 0) }
    else {
        let (groups, largest) = group_stats(keys, n - 1);
        let earlier = count_key(keys, &keys[n - 1], n - 1);
        (
            groups + if earlier == 0 { 1 } else { 0 },
            if earlier + 1 > largest { earlier + 1 } else { largest },
        )
    }
}

/// The membership predicate implemented by check_margin below.
pub open spec fn margin_member_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>,
    max_length: Option<u32>, max_groups: Option<u32>,
) -> Result<bool, ()> {
    if by.len() == 0 {
        Ok(max_groups != Some(0u32)
            && match max_length {
                Some(limit) => rows.len() <= limit as nat,
                None => true,
            })
    } else if max_length.is_none() && max_groups.is_none() {
        // The domain separately verifies that every grouping expression is typed.
        Ok(true)
    } else {
        match keys_spec(rows, by, rows.len()) {
            Ok(keys) => {
                let (groups, largest) = group_stats_spec(keys, keys.len());
                Ok((match max_groups {
                    Some(limit) => groups <= limit as nat,
                    None => true,
                }) && (match max_length {
                    Some(limit) => largest <= limit as nat,
                    None => true,
                }))
            },
            Err(_) => Err(()),
        }
    }
}

/// Verus verifies the actual checker invoked by FrameDomain<VTable>::member.
pub fn check_margin(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>,
    max_length: Option<u32>, max_groups: Option<u32>,
) -> (result: Result<bool, ()>)
    ensures result == margin_member_spec(rows@, by@, max_length, max_groups),
{
    if by.len() == 0 {
        Ok(max_groups != Some(0u32)
            && match max_length {
                Some(limit) => rows.len() <= limit as usize,
                None => true,
            })
    } else if max_length.is_none() && max_groups.is_none() {
        Ok(true)
    } else {
        let keys = match eval_keys(rows, by, rows.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let (groups, largest) = group_stats(&keys, keys.len());
        Ok((match max_groups {
            Some(limit) => groups <= limit as usize,
            None => true,
        }) && (match max_length {
            Some(limit) => largest <= limit as usize,
            None => true,
        }))
    }
}

} // verus!
