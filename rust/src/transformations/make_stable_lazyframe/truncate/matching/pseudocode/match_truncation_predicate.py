# type: ignore
def match_truncation_predicate(
    predicate: Expr, identifier: Expr, schema: Optional[Schema]
) -> Optional[Vec[Bound]]:
    if (
        isinstance(predicate, Expr.Function)
        and predicate.function == BooleanFunction.AllHorizontal
    ):
        # propagate errors
        bounds = [
            match_truncation_predicate(expr, identifier, schema)
            for expr in predicate.input
        ]

        # propagate nones
        if any(bound is None for bound in bounds):  # |\label{line:check_all}|
            return None

        return [bound for matched in bounds for bound in matched]

    elif isinstance(predicate, Expr.BinaryExpr) and predicate.op == Operator.And:
        left = match_truncation_predicate(predicate.left, identifier, schema)
        right = match_truncation_predicate(predicate.right, identifier, schema)
        if left is None or right is None:  # |\label{line:check_both}|
            return None
        return left + right

    elif isinstance(predicate, Expr.BinaryExpr):
        left, right = predicate.left, predicate.right
        if predicate.op == Operator.Lt:
            over, threshold, offset = left, right, 0
        elif predicate.op == Operator.LtEq:
            over, threshold, offset = left, right, 1
        elif predicate.op == Operator.Gt:
            over, threshold, offset = right, left, 0
        elif predicate.op == Operator.GtEq:
            over, threshold, offset = right, left, 1
        else:
            return None

        if not groups_to_rows_window(over):  # |\label{line:check_over}|
            return None
        if over.order_by is not None:  # |\label{line:check_order_by}|
            return None

        threshold_value = literal_value_of(threshold, u32)
        if threshold_value is None:
            raise ValueError(
                f"literal value for truncation threshold ({threshold}) must be representable as a u32"
            )

        # account for distinction between gt and ge
        threshold_value = threshold_value.inf_add(
            offset
        )  # |\label{line:threshold_value}|

        per_group = match_per_group_predicate(
            over.function, over.partition_by, identifier, threshold_value, schema
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

        return [num_groups or per_group]  # |\label{line:return}|


def groups_to_rows_window(expr):
    if isinstance(expr, Expr.Over):
        return expr.mapping == WindowMapping.GroupsToRows
    if isinstance(expr, Expr.Window):
        return expr.options == WindowType.Over(WindowMapping.GroupsToRows)
    return False
