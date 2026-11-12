//! Executable margin checks, with an exact Verus model of group counts.
//! The worst case is quadratic in row count, but group counts now share one pass;
//! prototype table membership, not a substitute for Polars group-by.

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

/// Count occurrences of one grouping key within a prefix of input rows.
pub open spec fn count_key_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>,
    key: Seq<Option<VValue>>, n: nat,
) -> Result<nat, ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 {
        Ok(0)
    } else {
        match (
            count_key_spec(rows, by, key, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok(prior), Ok(value)) =>
                Ok(prior + if key_equal(value, key) { 1nat } else { 0nat }),
            _ => Err(()),
        }
    }
}

pub fn count_key(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>,
    key: &Vec<Option<VValue>>, n: usize,
) -> (result: Result<usize, ()>)
    requires n <= rows.len(),
    ensures
        match result {
            Ok(count) => count_key_spec(rows@, by@, key@, n as nat) == Ok(count as nat),
            Err(_) => count_key_spec(rows@, by@, key@, n as nat).is_Err(),
        },
        result.is_Ok() ==> result.unwrap() <= n,
    decreases n,
{
    if n == 0 {
        Ok(0)
    } else {
        let prior = match count_key(rows, by, key, n - 1) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let current = match project::eval_prefix(by, &rows[n - 1], by.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let matches = same_key(&current, key);
        Ok(prior + if matches { 1 } else { 0 })
    }
}

/// Compute both the number of groups and their maximum size in one
/// traversal. For each row, only the preceding rows with the same key
/// matter: a new group adds one, otherwise that group's size increases.
pub open spec fn group_stats_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>, n: nat,
) -> Result<(nat, nat), ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 { Ok((0, 0)) }
    else {
        match (
            group_stats_spec(rows, by, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok((groups, largest)), Ok(key)) =>
                match count_key_spec(rows, by, key, (n - 1) as nat) {
                    Ok(earlier) => Ok((
                        groups + if earlier == 0 { 1nat } else { 0nat },
                        if earlier + 1 > largest { earlier + 1 } else { largest },
                    )),
                    Err(_) => Err(()),
                },
            _ => Err(()),
        }
    }
}

pub fn group_stats(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>, n: usize,
) -> (result: Result<(usize, usize), ()>)
    requires n <= rows.len(),
    ensures
        match result {
            Ok(stats) =>
                group_stats_spec(rows@, by@, n as nat)
                    == Ok((stats.0 as nat, stats.1 as nat)),
            Err(_) => group_stats_spec(rows@, by@, n as nat).is_Err(),
        },
        result.is_Ok() ==>
            result.unwrap().0 <= n && result.unwrap().1 <= n,
    decreases n,
{
    if n == 0 {
        Ok((0, 0))
    } else {
        let (groups, largest) = match group_stats(rows, by, n - 1) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let key = match project::eval_prefix(by, &rows[n - 1], by.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let earlier = match count_key(rows, by, &key, n - 1) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        Ok((
            groups + if earlier == 0 { 1 } else { 0 },
            if earlier + 1 > largest { earlier + 1 } else { largest },
        ))
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
        match group_stats_spec(rows, by, rows.len()) {
            Ok((groups, largest)) => Ok(
                (match max_groups {
                    Some(limit) => groups <= limit as nat,
                    None => true,
                }) && (match max_length {
                    Some(limit) => largest <= limit as nat,
                    None => true,
                })
            ),
            _ => Err(()),
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
        let (groups, largest) = match group_stats(rows, by, rows.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
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
