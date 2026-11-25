# type: ignore
def match_truncation_predicate(
    predicate: Expr, identifier: Expr
) -> Optional[Vec[MatchedBound]]:
    if (
        isinstance(predicate, Expr.Function)
        and predicate.function == BooleanFunction.AllHorizontal
    ):
        children = predicate.input
    elif isinstance(predicate, Expr.BinaryExpr) and predicate.op == Operator.And:
        children = [predicate.left, predicate.right]
    elif isinstance(predicate, Expr.BinaryExpr):
        left, right = predicate.left, predicate.right
        if predicate.op == Operator.Lt:
            over, threshold, inclusive = left, right, False
        elif predicate.op == Operator.LtEq:
            over, threshold, inclusive = left, right, True
        elif predicate.op == Operator.Gt:
            over, threshold, inclusive = right, left, False
        elif predicate.op == Operator.GtEq:
            over, threshold, inclusive = right, left, True
        else:
            return None

        if not (
            (isinstance(over, Expr.Over) and over.mapping == WindowMapping.GroupsToRows)
            or (
                isinstance(over, Expr.Window)
                and over.options == WindowType.Over(WindowMapping.GroupsToRows)
            )
        ):  # |\label{line:check_over}|
            return None
        if over.order_by is not None:  # |\label{line:check_order_by}|
            return None

        if not is_integer_scalar_literal(threshold):
            raise ValueError("Truncation threshold must be an integer scalar literal.")
        threshold_value = literal_value_of(threshold, u32)
        if threshold_value is None:
            raise ValueError(
                f"literal value for truncation threshold ({threshold}) must be representable as a u32"
            )

        one_based = isinstance(over.function, Expr.Function) and isinstance(
            over.function.function, FunctionExpr.Rank
        )
        if one_based and not inclusive:  # |\label{line:threshold_value}|
            threshold_value = threshold_value.saturating_sub(1)
        elif not one_based and inclusive:
            threshold_value = threshold_value.inf_add(1)

        per_group = match_per_group_predicate(
            over.function, over.partition_by, identifier, threshold_value
        )
        num_groups = (
            match_num_groups_predicate(
                over.function, over.partition_by, identifier, threshold_value
            )
            if per_group is None
            else None
        )

        if num_groups is None and per_group is None:  # |\label{line:check_bounds}|
            raise ValueError(
                f"expected a trusted per_group enumeration or dense-rank num_groups truncation. Found {over.function}"
            )

        return [
            MatchedBound(num_groups, []) if num_groups is not None else per_group
        ]  # |\label{line:return}|

    else:
        return None

    bounds = [match_truncation_predicate(child, identifier) for child in children]
    if any(bound is None for bound in bounds):  # |\label{line:check_all}|
        return None
    return [bound for matched in bounds for bound in matched]
