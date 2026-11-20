# type: ignore
def match_num_groups_predicate(
    ranks: Expr,
    partition_by: Vec[Expr],
    identifier: Expr,
    threshold: u32,
) -> Optional[Bound]:
    # check if is a rank function
    if not isinstance(ranks, Expr.Function) or not isinstance(
        ranks.function, FunctionExpr.Rank
    ):  # |\label{line:check_rank}|
        return None

    input = ranks.input
    options = ranks.function.options

    if partition_by != [identifier]:  # |\label{line:check_identifier}|
        raise "num_groups truncation must use the identifier in the over clause"

    if options.method != RankMethod.Dense:  # |\label{line:check_method}|
        raise "num_groups truncation's rank must be dense"

    if len(input) != 1:  # |\label{line:check_input}|
        raise "rank function must be applied to a single input"

    input_item = input[0]

    # Treat as_struct as a special case that represents multiple columns.
    if isinstance(input_item, Expr.Function) and isinstance(
        input_item.function, FunctionExpr.AsStruct
    ):  # |\label{line:extract_grouping_columns}|
        # If the first field is a hash of the second field,
        # then interpret the grouping columns as the hash input.
        # The second field disambiguates hash collisions when ranking.
        fields = input_item.input
        if (
            fields
            and isinstance(fields[0], Expr.Function)
            and isinstance(fields[0].function, FunctionExpr.Hash)
        ):  # |\label{line:check_hash}|
            hash_inputs = fields[0].input
            second = fields[1] if len(fields) > 1 else None
            if (hash_inputs[0] if hash_inputs else None) == second:
                inner = hash_inputs[0] if hash_inputs else None
                if not isinstance(inner, Expr.Function) or not isinstance(
                    inner.function, FunctionExpr.AsStruct
                ):
                    raise f"expected hash input to be a struct, found {hash_inputs}"
                fields = inner.input

        by = set(fields)
    else:
        by = {input_item}

    return Bound(by=by, per_group=None, num_groups=threshold)  # |\label{line:bound}|
