//! Verified grouped-margin checks using the standard hash map.
//! One grouping-expression evaluation and expected O(1) map update per row.

use super::*;
use vstd::hash_map::HashMapWithView;
use vstd::map::Map;
#[cfg(verus_keep_ghost)]
use vstd::std_specs::hash::obeys_key_model;

verus! {

type Key = Vec<Option<VValue>>;
type GroupMap = HashMapWithView<Key, usize>;

/// Count grouping keys in a row prefix. This is also the mathematical
/// specification for the executable standard-library hash map.
pub open spec fn group_state_spec(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>, n: nat,
) -> Result<(Map<Seq<Option<VValue>>, usize>, usize), ()>
    recommends n <= rows.len(),
    decreases n,
{
    if n == 0 {
        Ok((Map::empty(), 0))
    } else {
        match (
            group_state_spec(rows, by, (n - 1) as nat),
            project::row_prefix(by, rows[(n - 1) as int]@, by.len()),
        ) {
            (Ok((counts, largest)), Ok(key)) => {
                let prior = counts.get(key).unwrap_or(0);
                if prior == usize::MAX {
                    Err(())
                } else {
                    let next = (prior as nat + 1) as usize;
                    Ok((counts.insert(key, next),
                        if next > largest { next } else { largest }))
                }
            },
            _ => Err(()),
        }
    }
}

/// Once a prefix fails to evaluate or overflows, extending it cannot
/// restore a valid grouped count.
pub proof fn lemma_group_error_persists(
    rows: Seq<Vec<Option<VValue>>>, by: Seq<VExpr>,
    prefix: nat, total: nat,
)
    requires
        prefix <= total <= rows.len(),
        group_state_spec(rows, by, prefix).is_Err(),
    ensures group_state_spec(rows, by, total).is_Err(),
    decreases total - prefix,
{
    if prefix < total {
        lemma_group_error_persists(rows, by, prefix, (total - 1) as nat);
    }
}

/// Interpret grouped margins as bounds on the number of groups and
/// the largest group. An empty grouping has one group even for zero rows.
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
        // The caller separately validates every grouping expression.
        Ok(true)
    } else {
        match group_state_spec(rows, by, rows.len()) {
            Ok((counts, largest)) => Ok(
                (match max_groups {
                    Some(limit) => counts.len() <= limit as nat,
                    None => true,
                }) && (match max_length {
                    Some(limit) => largest <= limit as usize,
                    None => true,
                })
            ),
            Err(_) => Err(()),
        }
    }
}

/// Count with the standard HashMap implementation, verified through vstd.
/// Key-model assumptions are isolated here: vstd currently axiomatizes
/// primitive keys but not derived Hash/Eq for Vec<Option<VValue>>.
pub fn group_stats(
    rows: &Vec<Vec<Option<VValue>>>, by: &Vec<VExpr>,
) -> (result: Result<(usize, usize), ()>)
    ensures match result {
        Ok(stats) => match group_state_spec(rows@, by@, rows.len() as nat) {
            Ok((counts, largest)) =>
                stats.0 as nat == counts.len() && stats.1 == largest,
            Err(_) => false,
        },
        Err(_) => group_state_spec(rows@, by@, rows.len() as nat).is_Err(),
    },
{
    proof {
        // These properties hold for the derived standard Rust Hash and Eq
        // implementations, but are not yet established generically by vstd.
        assume(obeys_key_model::<Key>());
        assume(forall|k1: Key, k2: Key| k1@ == k2@ ==> k1 == k2);
    }
    let mut counts = GroupMap::new();
    let mut largest: usize = 0;
    let mut i: usize = 0;
    while i < rows.len()
        invariant
            i <= rows.len(),
            group_state_spec(rows@, by@, i as nat) == Ok((counts@, largest)),
        decreases rows.len() - i
    {
        let key = match project::eval_prefix(by, &rows[i], by.len()) {
            Ok(value) => value,
            Err(_) => {
                proof {
                    assert(group_state_spec(rows@, by@, (i + 1) as nat).is_Err());
                    lemma_group_error_persists(rows@, by@,
                        (i + 1) as nat, rows.len() as nat);
                }
                return Err(());
            },
        };
        let ghost key_view = key@;
        let ghost old_counts = counts@;
        let ghost old_largest = largest;
        let prior = match counts.get(&key) {
            Some(value) => *value,
            None => 0,
        };
        if prior == usize::MAX {
            proof {
                assert(group_state_spec(rows@, by@, (i + 1) as nat).is_Err());
                lemma_group_error_persists(rows@, by@,
                    (i + 1) as nat, rows.len() as nat);
            }
            return Err(());
        }
        let next = prior + 1;
        counts.insert(key, next);
        if next > largest {
            largest = next;
        }
        proof {
            assert(group_state_spec(rows@, by@, (i + 1) as nat)
                == Ok((old_counts.insert(key_view, next), largest)));
        }
        i += 1;
    }
    Ok((counts.len(), largest))
}

/// Verus verifies the checker invoked by FrameDomain<VTable>::member.
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
        let (groups, largest) = match group_stats(rows, by) {
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
