# type: ignore
def make_expr_datetime_component(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Transformation:
    match expr:  # `\label{match-strategy}`
        case Expr.Function(input=inputs, function=FunctionExpr.TemporalExpr(temporal_function)):
            pass
        case _:
            raise ValueError("expected datetime component expression")
    
    to_dtype, _ = match_datetime_component(temporal_function)  # `\label{match-dt}`

    # raises an error if there is not exactly one input
    input, = inputs  # `\label{one-input}`

    t_prior = input.make_stable(input_domain, input_metric)  # `\label{t-prior}`
    middle_domain, middle_metric = t_prior.output_space()

    in_dtype = middle_domain.column.dtype
    if in_dtype not in {DataType.Time, DataType.Datetime, DataType.Date}:  # `\label{check-dtype}`
        raise ValueError("expected a temporal input type")
    
    output_domain = middle_domain.clone()  # `\label{output-domain}`
    output_domain.column.set_dtype(to_dtype) # `\label{domain-dtype}`

    def function(expr: Expr) -> Expr:
        return Expr.Function(
            input=[expr],
            function=FunctionExpr.TemporalExpr(temporal_function),
            options=FunctionOptions(
                collect_groups=ApplyOptions.ElementWise,
            ),
        )

    return t_prior >> Transformation.new(  # `\label{t-component}`
        middle_domain,
        output_domain,
        Function.then_expr(function),
        middle_metric,
        middle_metric,
        StabilityMap.new(lambda d_in: d_in),
    )