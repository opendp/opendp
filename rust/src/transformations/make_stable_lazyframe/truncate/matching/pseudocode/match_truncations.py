# type: ignore
def match_truncations(plan: DslPlan, identifier: Expr) -> Optional[MatchedTruncations]:
    filters = []
    group_by = None
    bounds = []

    matched_group_by = match_group_by_truncation(
        plan, identifier
    )  # |\label{line:match_group_by_truncation}|
    if matched_group_by is not None:
        plan, group_by, group_bound = matched_group_by

    # match until not a filter truncation
    while isinstance(plan, DslPlan.Filter):  # |\label{line:match_filter_truncation}|
        input, predicate = plan.input, plan.predicate
        new_bounds = match_truncation_predicate(
            predicate, identifier
        )  # |\label{line:match_truncation_predicate}|
        if new_bounds is None:
            break

        plan = input  # |\label{line:match_filter_truncation_start}|
        filters.append(predicate)
        bounds.extend(new_bounds)  # |\label{line:match_filter_truncation_end}|

    # just for better error messages, no privacy implications
    if (
        match_group_by_truncation(plan, identifier) is not None
    ):  # |\label{line:match_group_by_truncation_check}|
        raise Exception("groupby truncation must be the last truncation in the plan.")

    # since the parse descends to the source,
    # filters and bounds are in reverse order
    filters.reverse()  # |\label{line:truncations_reverse}|
    bounds.reverse()  # |\label{line:bounds_reverse}|

    if group_by is not None:
        bounds.append(MatchedBound(group_bound, []))

    if not filters and group_by is None:
        return None
    return MatchedTruncations(plan, filters, group_by, bounds)
