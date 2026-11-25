# type: ignore
def match_group_by_truncation(
    plan: DslPlan, identifier: Expr
) -> Optional[Tuple[DslPlan, GroupByTruncation, Bound]]:
    if not isinstance(plan, DslPlan.GroupBy):  # |\label{line:check_groupby}|
        return None

    input = plan.input
    keys = plan.keys
    aggs = plan.aggs
    apply = plan.apply
    options = plan.options

    if apply is not None or plan.predicates or options != GroupbyOptions.default():
        return None  # |\label{line:check_apply}|

    if identifier not in keys:  # |\label{line:check_ids}|
        return None

    by = {key for key in keys if key != identifier}
    return (
        input,
        GroupByTruncation(keys, aggs),
        Bound(by=by, per_group=1, num_groups=None),  # |\label{line:bound}|
    )
