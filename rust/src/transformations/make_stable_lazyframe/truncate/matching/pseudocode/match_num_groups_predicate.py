# type: ignore
def match_num_groups_predicate(
    ranks: Expr,
    partition_by: Vec[Expr],
    identifier: Expr,
    threshold: u32,
    schema: Optional[Schema],
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

    def strip_alias(expr):
        while isinstance(expr, Expr.Alias):
            expr = expr.expr
        return expr

    # Treat as_struct as a special case that represents multiple columns.
    if isinstance(input_item, Expr.Function) and isinstance(
        input_item.function, FunctionExpr.AsStruct
    ):  # |\label{line:extract_grouping_columns}|
        fields = input_item.input
        # Hash and key fields may have distinct aliases to avoid name collisions.
        if fields:
            first = strip_alias(fields[0])
            second = strip_alias(fields[1]) if len(fields) > 1 else None
            if isinstance(first, Expr.Function) and isinstance(
                first.function, FunctionExpr.Hash
            ):  # |\label{line:check_hash}|
                if (first.input[0] if first.input else None) == second:
                    # |\label{line:check_hash_shape}|
                    if len(fields) != 2 or len(first.input) != 1:
                        raise "hashed rank has unexpected struct fields or hash inputs"
                    if not isinstance(second, Expr.Function) or not isinstance(
                        second.function, FunctionExpr.AsStruct
                    ):
                        raise f"expected hash input to be a struct, found {first.input}"

                    fields = second.input
        by = set(fields)
    else:
        by = {input_item}

    # All rank keys, not only hashed keys, must have permutation-stable codes.
    if schema is not None:  # |\label{line:check-categorical-hash}|
        for key in by:
            if contains_categorical(key.to_field(schema).dtype):
                raise "num_groups truncation cannot use Categorical keys"

    return Bound(by=by, per_group=None, num_groups=threshold)


def contains_categorical(dtype):
    if isinstance(dtype, DataType.Categorical):
        return True
    if isinstance(dtype, DataType.Struct):
        return any(contains_categorical(field.dtype) for field in dtype.fields)
    if isinstance(dtype, (DataType.Array, DataType.List)):
        return contains_categorical(dtype.inner)
    return False  # |\label{line:bound}|
