use std::collections::HashSet;

use super::super::group_by::match_ordinary_group_by;
use super::super::permutation::is_permutation_equivariant;
use crate::polars::contains_categorical;
use crate::{error::Fallible, metrics::Bound, polars::literal_value_of, traits::InfAdd};
use opendp_derive::proven;
#[cfg(not(patch_polars))]
use polars_plan::dsl::WindowType;

use polars::prelude::{
    BooleanFunction, DataType, DslPlan, Expr, FunctionExpr, Operator, RankMethod, Schema,
    WindowMapping, int_range, len, lit,
};

#[cfg(test)]
mod test;

#[derive(PartialEq, Debug)]
pub(crate) struct GroupByTruncation {
    pub(super) keys: Vec<Expr>,
    pub(super) aggs: Vec<Expr>,
}

/// A structurally matched contribution bound and its deferred sort-key checks.
#[derive(PartialEq, Debug)]
pub(crate) struct MatchedBound {
    pub(super) bound: Bound,
    sort_keys: Vec<Expr>,
}

impl MatchedBound {
    fn new(bound: Bound) -> Self {
        Self {
            bound,
            sort_keys: vec![],
        }
    }

    /// Validate ranking; the constructor separately checks grouping expressions.
    pub(super) fn validate_ranking(self, schema: &Schema) -> Fallible<Bound> {
        for key in &self.sort_keys {
            let dtype = key.clone().to_field(schema)?.dtype;
            if contains_categorical(&dtype) {
                return fallible!(
                    MakeTransformation,
                    "SortBy truncation cannot use Categorical keys; cast to String or Enum first."
                );
            }
            if !rank_key_dtype_supported(&dtype) {
                return fallible!(
                    MakeTransformation,
                    "Unsupported SortBy truncation key dtype: {dtype}"
                );
            }
            if !is_permutation_equivariant(key, schema) {
                return fallible!(
                    MakeTransformation,
                    "SortBy truncation keys must be permutation-equivariant expressions."
                );
            }
        }
        Ok(self.bound)
    }
}

// Ranking is only certified for scalar dtypes. Structs used by the
// truncation grammar are checked field by field, not as arbitrary rank keys.
fn rank_key_dtype_supported(dtype: &DataType) -> bool {
    dtype.is_primitive_numeric()
        || matches!(
            dtype,
            DataType::Boolean
                | DataType::String
                | DataType::Enum(..)
                | DataType::Date
                | DataType::Datetime(..)
                | DataType::Time
                | DataType::Duration(_)
                | DataType::Null
        )
}

#[derive(Debug)]
pub(crate) struct MatchedTruncations {
    pub(super) input: DslPlan,
    pub(super) filters: Vec<Expr>,
    pub(super) group_by: Option<GroupByTruncation>,
    pub(super) bounds: Vec<MatchedBound>,
}

/// Matches multiple truncations in a compute plan.
/// Errors only when the plan is unambiguously a mis-specified truncation.
///
/// # Proof Definition
/// For any choice of LazyFrame plan,
/// returns the plan with the truncations removed,
/// the truncations that were removed,
/// and candidate per-id bounds with their deferred validation obligations.
/// The caller must check that all filter grouping keys are retained by the final group-by.
/// Returns None if no truncations match. The caller must validate every matched bound before using it.
#[proven]
pub(crate) fn match_truncations(
    mut plan: DslPlan,
    identifier: &Expr,
) -> Fallible<Option<MatchedTruncations>> {
    let mut filters = vec![];
    let mut bounds = vec![];
    let group_by = match_group_by_truncation(&plan, identifier).map(|(input, group_by, bound)| {
        plan = input;
        (group_by, bound)
    });

    // match until not a filter truncation
    while let DslPlan::Filter { input, predicate } = plan.clone() {
        let Some(new_bounds) = match_truncation_predicate(&predicate, identifier)? else {
            break;
        };

        plan = input.as_ref().clone();
        filters.push(predicate);
        bounds.extend(new_bounds);
    }

    // just for better error messages, no privacy implications
    if match_group_by_truncation(&plan, identifier).is_some() {
        return fallible!(
            MakeTransformation,
            "Groupby truncation must be the last truncation in the plan. Otherwise the groupby truncation may invalidate later truncations."
        );
    }
    // since the parse descends to the source,
    // filters and bounds are in reverse order
    filters.reverse();
    bounds.reverse();

    let group_by = if let Some((group_by, bound)) = group_by {
        bounds.push(MatchedBound::new(bound));
        Some(group_by)
    } else {
        None
    };

    if filters.is_empty() && group_by.is_none() {
        return Ok(None);
    }
    Ok(Some(MatchedTruncations {
        input: plan,
        filters,
        group_by,
        bounds,
    }))
}

/// Matches a truncation via a groupby.
///
/// # Proof Definition
/// For a given query plan and user identifier expression,
/// if the query plan bounds row contributions per-identifier via a group-by,
/// returns a triple containing the input to the truncation,
/// the truncation itself, and the per-id bound on user contribution.
#[proven]
fn match_group_by_truncation(
    plan: &DslPlan,
    identifier: &Expr,
) -> Option<(DslPlan, GroupByTruncation, Bound)> {
    let (input, keys, aggs, _) = match_ordinary_group_by(plan).ok()?;

    if !keys.contains(identifier) {
        return None;
    }

    Some((
        input.clone(),
        GroupByTruncation {
            keys: keys.to_vec(),
            aggs: aggs.to_vec(),
        },
        Bound {
            by: keys
                .iter()
                .filter(|key| *key != identifier)
                .cloned()
                .collect(),
            per_group: Some(1),
            num_groups: None,
        },
    ))
}

/// Match user identifier truncations in a predicate.
///
/// # Proof Definition
/// For a given filter predicate and identifier expression,
/// returns an error if the predicate contains a mis-specified truncation,
/// none if the predicate is not a truncation,
/// otherwise candidate per-identifier contribution bounds with all deferred checks.
#[proven]
fn match_truncation_predicate(
    predicate: &Expr,
    identifier: &Expr,
) -> Fallible<Option<Vec<MatchedBound>>> {
    let children: Vec<&Expr> = match predicate {
        Expr::Function {
            input,
            function: FunctionExpr::Boolean(BooleanFunction::AllHorizontal),
            ..
        } => input.iter().collect(),
        Expr::BinaryExpr {
            left,
            op: Operator::And,
            right,
        } => vec![left.as_ref(), right.as_ref()],

        // conditions that limit contributions per partition or influenced partitions
        Expr::BinaryExpr { left, op, right } => {
            let (over, threshold, inclusive) = match op {
                Operator::Lt => (left, right, false),
                Operator::LtEq => (left, right, true),
                Operator::Gt => (right, left, false),
                Operator::GtEq => (right, left, true),
                // don't throw an error as non-truncation filters may still be valid
                _ => return Ok(None),
            };
            #[cfg(patch_polars)]
            let Expr::Over {
                function,
                partition_by,
                order_by: None,
                mapping: WindowMapping::GroupsToRows,
            } = over.as_ref()
            else {
                return Ok(None);
            };
            #[cfg(not(patch_polars))]
            let Expr::Window {
                function,
                partition_by,
                order_by: None,
                options: WindowType::Over(WindowMapping::GroupsToRows),
            } = over.as_ref()
            else {
                return Ok(None);
            };

            // Filters that aren't truncations don't support window functions,
            // so from here on, we can safely assume the predicate is meant to be a truncation,
            // and raise informative errors when the predicate is mis-shapen.

            // Numeric extraction alone may truncate a fractional literal.
            if !matches!(threshold.as_ref(), Expr::Literal(value)
                if value.is_scalar() && value.to_any_value().is_some_and(|v| v.dtype().is_integer()))
            {
                return fallible!(
                    MakeTransformation,
                    "Truncation threshold must be an integer scalar literal."
                );
            }
            let Some(threshold) = literal_value_of::<u32>(&threshold)? else {
                return fallible!(
                    MakeTransformation,
                    "literal value for truncation threshold ({:?}) must be representable as a u32",
                    threshold
                );
            };

            // Ranks start at one; permuted indices start at zero.
            let one_based = matches!(
                function.as_ref(),
                Expr::Function {
                    function: FunctionExpr::Rank { .. },
                    ..
                }
            );
            let threshold = match (one_based, inclusive) {
                (true, false) => threshold.saturating_sub(1),
                (false, true) => threshold.inf_add(&1)?,
                _ => threshold,
            };
            let per_group =
                match_per_group_predicate(function.as_ref(), partition_by, identifier, threshold)?;
            let num_groups = if per_group.is_none() {
                match_num_groups_predicate(function.as_ref(), partition_by, identifier, threshold)?
            } else {
                None
            };

            let Some(bound) = num_groups.map(MatchedBound::new).or(per_group) else {
                return fallible!(
                    MakeTransformation,
                    "expected per_group truncation via a trusted permutation or sort-by ranks, or num_groups truncation via dense rank. Found {:?}",
                    function
                );
            };

            return Ok(Some(vec![bound]));
        }
        _ => return Ok(None),
    };

    let mut bounds = vec![];
    let mut all_matched = true;
    for child in children {
        match match_truncation_predicate(child, identifier)? {
            Some(matched) => bounds.extend(matched),
            None => all_matched = false,
        }
    }
    Ok(all_matched.then_some(bounds))
}

/// # Proof Definition
/// If `ranks` is a dense ranking of grouping columns,
/// and `partition_by` is a singleton of `identifier`,
/// then returns the bound on per-identifier contributions,
/// or an error if the truncation is mis-specified.
#[proven]
fn match_num_groups_predicate(
    ranks: &Expr,
    partition_by: &[Expr],
    identifier: &Expr,
    threshold: u32,
) -> Fallible<Option<Bound>> {
    // check if the function is limiting num_groups
    let Expr::Function {
        input,
        function: FunctionExpr::Rank { options, .. },
        ..
    } = ranks
    else {
        return Ok(None);
    };
    if partition_by != std::slice::from_ref(identifier) {
        return fallible!(
            MakeTransformation,
            "num_groups truncation must use the identifier in the over clause"
        );
    }

    if !matches!(options.method, RankMethod::Dense) {
        return fallible!(
            MakeTransformation,
            "num_groups truncation's rank must be dense"
        );
    }

    let Ok([input_item]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeTransformation,
            "rank function must be applied to a single input, found {:?}",
            input.len()
        );
    };

    let by = num_groups_keys(input_item)?.iter().cloned().collect();

    Ok(Some(Bound {
        by,
        per_group: None,
        num_groups: Some(threshold),
    }))
}

/// Grouping expressions in the single input to a dense rank.
pub(super) fn num_groups_keys(input_item: &Expr) -> Fallible<&[Expr]> {
    Ok(match input_item {
        // Treat as_struct as a special case that represents multiple columns.
        // Could still actually have a struct column via to_struct(to_struct(...)).
        Expr::Function {
            function: FunctionExpr::AsStruct,
            input,
            ..
        } => {
            // If the first field is a hash of the second field,
            // then interpret the grouping columns as the hash input.
            // The second field disambiguates hash collisions when ranking.
            if let Some(Expr::Function {
                input: hash_input,
                function: FunctionExpr::Hash(_, _, _, _),
                ..
            }) = input.get(0)
            {
                if hash_input.get(0) == input.get(1) {
                    let Some(Expr::Function {
                        input: true_input,
                        function: FunctionExpr::AsStruct,
                        ..
                    }) = hash_input.get(0)
                    else {
                        return fallible!(
                            MakeTransformation,
                            "expected hash input to be a struct, found {:?}",
                            hash_input
                        );
                    };
                    return Ok(true_input);
                }
            }
            input
        }
        input => std::slice::from_ref(input),
    })
}

/// # Proof Definition
/// If `enumeration` is a trusted uniform permutation of row indices or
/// ordinal ranks of permutation-equivariant sort keys with trusted uniform
/// tie-breaking, and `partition_by` includes `identifier`, then returns
/// a valid bound on per-group contributions when grouped by the remaining
/// expressions in `partition_by`, provided the returned sort keys are validated.
#[proven]
fn match_per_group_predicate(
    enumeration: &Expr,
    partition_by: &[Expr],
    identifier: &Expr,
    threshold: u32,
) -> Fallible<Option<MatchedBound>> {
    let range = int_range(lit(0), len(), 1, DataType::Int64);
    if enumeration == &range {
        return fallible!(
            MakeTransformation,
            "per-group truncation requires a full trusted random permutation; physical first/last are not stable under unordered identifier metrics."
        );
    }
    let sort_keys =
        if crate::polars::permute::match_permute(enumeration)?.is_some_and(|data| data == &range) {
            vec![]
        } else if let Some(keys) = match_ordinal_rank(enumeration, &range)? {
            keys
        } else {
            return Ok(None);
        };
    // we now know this is a per group predicate,
    // and can raise more informative error messages

    // check if the function is limiting partition contributions
    let (ids, by) = partition_by
        .iter()
        .cloned()
        .partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.is_empty() {
        return fallible!(
            MakeTransformation,
            "failed to find identifier column in per_group predicate condition"
        );
    }

    Ok(Some(MatchedBound {
        bound: Bound {
            by,
            per_group: Some(threshold),
            num_groups: None,
        },
        sort_keys,
    }))
}

fn match_ordinal_rank(enumeration: &Expr, range: &Expr) -> Fallible<Option<Vec<Expr>>> {
    let Expr::Function {
        input,
        function: FunctionExpr::Rank { options, .. },
    } = enumeration
    else {
        return Ok(None);
    };
    if options.method != RankMethod::Ordinal {
        return Ok(None);
    }
    let [
        Expr::Function {
            input: fields,
            function: FunctionExpr::AsStruct,
        },
    ] = input.as_slice()
    else {
        return Ok(None);
    };
    let Some((random_priority, sort_keys)) = fields.split_last() else {
        return Ok(None);
    };
    let Expr::Alias(random_priority, _) = random_priority else {
        return Ok(None);
    };
    if !crate::polars::permute::match_permute(random_priority)?.is_some_and(|data| data == range) {
        return Ok(None);
    }
    Ok(Some(sort_keys.to_vec()))
}
