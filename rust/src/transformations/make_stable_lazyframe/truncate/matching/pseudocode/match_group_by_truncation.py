# type: ignore
def match_group_by_truncation(
    plan: DslPlan, identifier: Expr
) -> Optional[Tuple[DslPlan, Truncation, Bound]]:
    if not isinstance(plan, DslPlan.GroupBy): # |\label{line:check_groupby}|
        return None

    input = plan.input
    keys = plan.keys
    aggs = plan.aggs
    apply = plan.apply
    options = plan.options

    if apply is not None or options != GroupbyOptions.default():
        return None # |\label{line:check_apply}|

    ids, by = partition(lambda expr: expr == identifier, keys) # |\label{line:partition}|

    if not ids: # |\label{line:check_ids}|
        return None

    return (
        input,
        Truncation.GroupBy(keys, aggs),
        Bound(by=by, per_group=1, num_groups=None), # |\label{line:bound}|
    )


# part of Rust's standard lib, included for readability, with hardcoded types
def partition(
    predicate, iterable: Iterable[Expr]
) -> Tuple[HashSet[Expr], HashSet[Expr]]:
    true_set = set()
    false_set = set()
    for item in iterable:
        if predicate(item):
            true_set.add(item)
        else:
            false_set.add(item)
    return true_set, false_set
