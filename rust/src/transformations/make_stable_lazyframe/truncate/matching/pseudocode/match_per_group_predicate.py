# type: ignore
def match_per_group_predicate(
    enumeration: Expr,
    partition_by: Vec[Expr],
    identifier: Expr,
    threshold: u32,
) -> Optional[Bound]:
    # reorderings of an enumeration are still enumerations
    if isinstance(enumeration, Expr.Function) and isinstance( # |\label{line:reordering_start}|
        enumeration.options.collect_groups, ApplyOptions.GroupWise
    ):
        input = enumeration.input
        function = enumeration.function

        # FunctionExprs that may reorder data
        if function == FunctionExpr.Reverse:
            is_reorder = True
        elif isinstance(function, FunctionExpr.Random):
            method = str(function.method)
            is_reorder = method == "shuffle"
        else:
            is_reorder = False
        
        if is_reorder:
            enumeration = input[0]

    elif isinstance(enumeration, Expr.SortBy):
        for key in enumeration.by:
            check_infallible(key, Resize.Ban)
        enumeration = enumeration.expr# |\label{line:reordering_end}|

    # in Rust, the != results in a boolean comparison, not a "ne" expression
    if enumeration != int_range(lit(0), len(partition_by), 1, DataType.Int64): # |\label{line:check_enumeration}|
        return None

    # we now know this is a per group predicate,
    # and can raise more informative error messages

    # check if the function is limiting partition contributions
    ids, by = partition(lambda expr: expr == identifier, partition_by) # |\label{line:partition}|

    if not ids:
        raise "failed to find identifier column in per_group predicate condition"

    return Bound(by=by, per_group=threshold, num_groups=None) # |\label{line:bound}|
