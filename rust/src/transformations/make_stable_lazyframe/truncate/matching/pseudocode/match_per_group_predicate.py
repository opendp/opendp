# type: ignore
def match_per_group_predicate(
    enumeration: Expr,
    partition_by: Vec[Expr],
    identifier: Expr,
    threshold: u32,
) -> Optional[MatchedBound]:
    range = int_range(lit(0), len(), 1, DataType.Int64)
    if enumeration == range:
        raise "physical row order is not invariant under unordered identifier metrics"

    if match_permute(enumeration) == range:
        sort_keys = []
    else:
        sort_keys = match_ordinal_rank(
            enumeration, range
        )  # |\label{line:check_enumeration}|
        if sort_keys is None:
            return None

    ids, by = partition(
        lambda expr: expr == identifier, partition_by
    )  # |\label{line:partition}|
    if not ids:
        raise "failed to find identifier column in per_group predicate condition"
    return MatchedBound(
        Bound(by=by, per_group=threshold, num_groups=None),
        sort_keys,
    )  # |\label{line:bound}|
