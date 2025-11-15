# type: ignore
def truncate_id_bound(
    id_bound: Bound,
    truncation: Bound,
    total_ids: Optional[int],
) -> Bound:
    # Once truncated, max contributions when grouped by "over" are bounded
    row_bound = Bound.by(truncation.by)

    # In each group, the worst-case row contributions is the
    # the number of ids contributed (known from id_bound)
    # times the number of rows contributed under each id (known from truncation),
    num_ids, num_rows = id_bound.per_group, truncation.per_group
    if num_ids is not None and num_rows is not None:
        row_bound = row_bound.with_per_group(num_ids.inf_mul(num_rows)) # |\label{line:per-group}|

    # Worst case number of groups contributed is the
    # total number of ids contributed (total_ids)
    # times the number of groups contributed under each id (known from truncation).
    num_groups_via_truncation = None  # |\label{line:num-groups-via-truncation}|
    if total_ids is not None and truncation.num_groups is not None:
        num_groups_via_truncation = total_ids.inf_mul(truncation.num_groups)

    # Alternatively, the number of groups contributed may be known outright from id_bound.
    # Use the smaller of the two if both are known.
    num_groups = option_min(num_groups_via_truncation, id_bound.num_groups)
    if num_groups is not None:
        row_bound = row_bound.with_num_groups(num_groups) # |\label{line:num-groups}|

    return row_bound
