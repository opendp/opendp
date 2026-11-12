//! Executable margin checks, with an exact Verus model of group counts.
//! The implementation is quadratic in row count; acceptable for verified
//! prototype table membership, but not a substitute for Polars group-by.

use super::*;

verus! {

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
                Ok(prior + if value == key { 1nat } else { 0nat }),
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
        let matches = current == *key;
        proof {
            assert(matches == (current@ == key@));
        }
        Ok(prior + if matches { 1 } else { 0 })
    }
}

/// Number of distinct nonempty groups represented by the first n rows.
pub open spec fn distinct_groups_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>, n: nat,
) -> Result<nat, ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 { Ok(0) }
    else {
        match (
            distinct_groups_spec(rows, by, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok(groups), Ok(key)) =>
                match count_key_spec(rows, by, key, (n - 1) as nat) {
                    Ok(count) => Ok(groups + if count == 0 { 1nat } else { 0nat }),
                    Err(_) => Err(()),
                },
            _ => Err(()),
        }
    }
}

pub fn distinct_groups(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>, n: usize,
) -> (result: Result<usize, ()>)
    requires n <= rows.len(),
    ensures
        match result {
            Ok(count) => distinct_groups_spec(rows@, by@, n as nat) == Ok(count as nat),
            Err(_) => distinct_groups_spec(rows@, by@, n as nat).is_Err(),
        },
        result.is_Ok() ==> result.unwrap() <= n,
    decreases n,
{
    if n == 0 {
        Ok(0)
    } else {
        let prior = match distinct_groups(rows, by, n - 1) {
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
        Ok(prior + if earlier == 0 { 1 } else { 0 })
    }
}

/// Largest number of occurrences of any grouping key.
pub open spec fn largest_group_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>, n: nat,
) -> Result<nat, ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 { Ok(0) }
    else {
        match (
            largest_group_spec(rows, by, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok(prior), Ok(key)) =>
                match count_key_spec(rows, by, key, rows.len()) {
                    Ok(count) => Ok(if count > prior { count } else { prior }),
                    Err(_) => Err(()),
                },
            _ => Err(()),
        }
    }
}

pub fn largest_group(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>, n: usize,
) -> (result: Result<usize, ()>)
    requires n <= rows.len(),
    ensures
        match result {
            Ok(count) => largest_group_spec(rows@, by@, n as nat) == Ok(count as nat),
            Err(_) => largest_group_spec(rows@, by@, n as nat).is_Err(),
        },
        result.is_Ok() ==> result.unwrap() <= rows.len(),
    decreases n,
{
    if n == 0 { Ok(0) }
    else {
        let prior = match largest_group(rows, by, n - 1) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let key = match project::eval_prefix(by, &rows[n - 1], by.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let count = match count_key(rows, by, &key, rows.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        Ok(if count > prior { count } else { prior })
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
    } else {
        match (
            distinct_groups_spec(rows, by, rows.len()),
            largest_group_spec(rows, by, rows.len()),
        ) {
            (Ok(groups), Ok(largest)) => Ok(
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
    } else {
        let groups = match distinct_groups(rows, by, rows.len()) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let largest = match largest_group(rows, by, rows.len()) {
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
