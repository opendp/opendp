# type: ignore
def match_per_group_predicate(
    enumeration: Expr,
    partition_by: Vec[Expr],
    identifier: Expr,
    threshold: u32,
    schema: Optional[Schema],
) -> Optional[Bound]:
    range = int_range(lit(0), len(), 1, DataType.Int64)
    if enumeration == range:
        raise "physical row order is not invariant under unordered identifier metrics"

    trusted = match_permute(enumeration) == range
    one_based = False
    if not trusted:
        trusted = match_ordinal_rank(
            enumeration, range, schema
        )  # |\label{line:check_enumeration}|
        one_based = trusted
    if not trusted:
        return None

    ids, by = partition(
        lambda expr: expr == identifier, partition_by
    )  # |\label{line:partition}|
    if not ids:
        raise "failed to find identifier column in per_group predicate condition"
    return Bound(
        by=by, per_group=max(0, threshold - int(one_based)), num_groups=None
    )  # |\label{line:bound}|


def match_ordinal_rank(
    enumeration: Expr, range: Expr, schema: Optional[Schema]
) -> bool:
    if not (
        isinstance(enumeration, Expr.Function)
        and isinstance(enumeration.function, FunctionExpr.Rank)
        and enumeration.function.options.method == RankMethod.Ordinal
        and len(enumeration.input) == 1
        and isinstance(enumeration.input[0], Expr.Function)
        and enumeration.input[0].function == FunctionExpr.AsStruct
    ):
        return False

    fields = enumeration.input[0].input
    if not fields:
        return False
    *keys, last = fields
    if not isinstance(last, Expr.Alias) or match_permute(last.expr) != range:
        return False

    if schema is not None:
        for key in keys:  # |\label{line:sort-keys}|
            if contains_categorical(key.to_field(schema).dtype):
                raise "SortBy truncation cannot use Categorical keys"
            if not is_permutation_equivariant(key, schema):
                raise "SortBy truncation keys must be permutation-equivariant"
    return True


def contains_categorical(dtype):
    if isinstance(dtype, DataType.Categorical):
        return True
    if isinstance(dtype, DataType.Struct):
        return any(contains_categorical(field.dtype) for field in dtype.fields)
    if isinstance(dtype, (DataType.Array, DataType.List)):
        return contains_categorical(dtype.inner)
    return False
