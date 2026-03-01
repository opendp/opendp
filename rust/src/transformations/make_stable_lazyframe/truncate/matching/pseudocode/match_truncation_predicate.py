# type: ignore
def match_truncation_predicate(
    predicate: Expr, identifier: Expr
) -> Optional[Vec[Bound]]:
    if (
        isinstance(predicate, FunctionExpr)
        and predicate.function == BooleanFunction.AllHorizontal
    ):
        # propagate errors
        bounds = [
            match_truncation_predicate(expr, identifier) for expr in predicate.input
        ]

        # propagate nones
        if not all(bounds): # |\label{line:check_all}|
            return None
        
        # appears to differ from Rust, but is equivalent 
        # because options don't need to be flattened in Python
        return bounds

    elif isinstance(predicate, BinaryExpr) and predicate.op == Operator.And:
        left = match_truncation_predicate(predicate.left, identifier)
        right = match_truncation_predicate(predicate.right, identifier)
        if left is None or right is None:  # |\label{line:check_both}|
            return None
        return left + right

    elif isinstance(predicate, BinaryExpr):
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

        if not isinstance(over, Expr.Window): # |\label{line:check_over}|
            return None

        threshold_value = literal_value_of(threshold, u32)
        if threshold_value is None:
            raise ValueError(
                f"literal value for truncation threshold ({threshold}) must be representable as a u32"
            )

        # account for distinction between gt and ge
        threshold_value = threshold_value.inf_add(offset) # |\label{line:threshold_value}|

        num_groups = match_num_groups_predicate(
            over.function, over.partition_by, identifier, threshold_value
        )
        per_group = match_per_group_predicate(
            over.function, over.partition_by, identifier, threshold_value
        )

        if num_groups is None and per_group is None: # |\label{line:check_bounds}|
            raise ValueError(
                f"expected a predicate that limits per_group contributions (via int_range) or num_groups contributions (via rank). Found {over.function}"
            )

        return [num_groups or per_group] # |\label{line:return}|
