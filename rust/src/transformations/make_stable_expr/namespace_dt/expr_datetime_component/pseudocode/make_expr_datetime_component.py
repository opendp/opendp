# type: ignore
def make_expr_datetime_component(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Transformation:
    match expr:  # `\label{match-strategy}`
        case Expr.Function(
            input=inputs, function=FunctionExpr.TemporalExpr(temporal_function)
        ):
            pass
        case _:
            raise ValueError("expected datetime component expression")

    to_dtype = match_datetime_component(temporal_function)  # `\label{match-dt}`

    if to_dtype is None:
        raise ValueError("expected datetime component")

    # raises an error if there is not exactly one input
    (input,) = inputs  # `\label{one-input}`

    t_prior = input.make_stable(input_domain, input_metric)  # `\label{t-prior}`
    middle_domain, middle_metric = t_prior.output_space()

    in_dtype = middle_domain.column.dtype
    if not (
        in_dtype in {DataType.Time, DataType.Date}
        or (in_dtype.is_datetime() and in_dtype.time_unit == TimeUnit.Nanoseconds)
    ):  # `\label{check-dtype}`
        raise ValueError("expected Time, Date, or nanosecond Datetime")

    is_clock = temporal_function in {
        Hour,
        Minute,
        Second,
        Millisecond,
        Microsecond,
        Nanosecond,
    }
    if (in_dtype == DataType.Date and is_clock) or (
        in_dtype == DataType.Time and not is_clock
    ):  # `\label{check-component}`
        raise ValueError("incompatible temporal component")

    output_domain = middle_domain.clone()  # `\label{output-domain}`
    output_domain.column.set_dtype(to_dtype)  # `\label{domain-dtype}`

    output_domain.column.nullable = True  # `\label{nullable}`

    def function(expr: Expr) -> Expr:
        return Expr.Function(
            input=[expr],
            function=FunctionExpr.TemporalExpr(temporal_function),
        )

    return t_prior >> Transformation.new(  # `\label{t-component}`
        middle_domain,
        middle_metric,
        output_domain,
        middle_metric,
        Function.then_expr(function),
        StabilityMap.new(lambda d_in: d_in),
    )
