# type: ignore
def match_truncations(
    plan: DslPlan, identifier: Expr
) -> tuple[DslPlan, Vec[Truncation], Vec[Bound]]:
    truncations = []
    bounds = []

    allowed_keys = match_group_by_truncation(plan, identifier) # |\label{line:match_group_by_truncation}|
    if allowed_keys:
        input, truncate, new_bound = allowed_keys
        plan = input # |\label{line:match_group_by_truncation_start}|
        truncations.append(truncate)
        bounds.append(new_bound) # |\label{line:match_group_by_truncation_end}|
        allowed_keys = new_bound.by # |\label{line:allowed_keys}|

    # match until not a filter truncation
    while isinstance(plan, Truncation.Filter): # |\label{line:match_filter_truncation}|
        input, predicate = plan.input, plan.predicate
        new_bounds = match_truncation_predicate(predicate, identifier) # |\label{line:match_truncation_predicate}|
        if new_bounds is None:
            break

        # When filter truncation is behind a groupby truncation,
        # if the groupby group keys don't cover the filter truncation keys,
        # then the groupby aggs can overwrite the filter truncation keys,
        # invalidating the filter truncation bounds.
        if allowed_keys is not None: # |\label{line:allowed_keys_check}|
            for bound in new_bounds:
                if not bound.by.is_subset(allowed_keys):
                    raise f"Filter truncation keys ({bound.by}) must be a subset of groupby truncation keys ({allowed_keys})."

        plan = input # |\label{line:match_filter_truncation_start}|
        truncations.append(Truncation.Filter(predicate))
        bounds.extend(new_bounds) # |\label{line:match_filter_truncation_end}|

    # just for better error messages, no privacy implications
    if match_group_by_truncation(plan, identifier) is not None: # |\label{line:match_group_by_truncation_check}|
        raise Exception("groupby truncation must be the last truncation in the plan.")

    # since the parse descends to the source,
    # truncations and bounds are in reverse order
    truncations.reverse() # |\label{line:truncations_reverse}|
    bounds.reverse() # |\label{line:bounds_reverse}|

    return plan, truncations, bounds
