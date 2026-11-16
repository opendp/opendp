# type: ignore
def make_expr_strptime(
    input_domain: WildExprDomain, input_metric: M, expr: Expr
) -> Transformation:
    match expr:  # `\label{match-strategy}`
        case Expr.Function(
            inputs,
            function=FunctionExpr.StringExpr(
                StringFunction.Strptime(to_type, strptime_options)
            ),
        ):
            pass
        case _:
            raise ValueError("expected str.strptime expression")
    input, _ = inputs

    t_prior = input.make_stable(input_domain, input_metric)  # `\label{t-prior}`
    middle_domain, middle_metric = t_prior.output_space()

    if strptime_options.format is None:  # `\label{format}`
        raise ValueError("format must be specified")

    if to_type == DataType.Time and not strptime_options.exact:
        raise ValueError("non-exact not implemented for Time data type")

    # never raise on error
    strptime_options.strict = False  # `\label{non-strict}`

    ambiguous = lit("null")  # `\label{ambiguous}`

    output_domain = middle_domain.clone()
    series_domain = output_domain.column

    # check input and output types
    if series_domain.dtype() != DataType.String:  # `\label{string-check}`
        raise ValueError("str.strptime input dtype must be String")

    # Reject nanoseconds, timezone-aware datetimes, and other output types.
    if to_type not in {
        DataType.Time,
        DataType.Date,
        DataType.Datetime("ms", None),
        DataType.Datetime("us", None),
    }:  # `\label{to-type-check}`
        raise ValueError("str.strptime output dtype is unsupported")

    inferred_dtype = (
        col(series_domain.name)
        .str.strptime(to_type, strptime_options, ambiguous)
        .to_field(Schema({"input": DataType.String}))
        .dtype
    )  # `\label{inferred-dtype}`
    if inferred_dtype != to_type:
        raise ValueError("str.strptime inferred a different output dtype")

    # in Rust, this assigns to the series domain in output_domain
    series_domain.set_dtype(to_type)  # `\label{out-series}`
    series_domain.nullable = True

    def function(expr: Expr) -> Expr:
        return expr.str.strptime(to_type, strptime_options, ambiguous)

    return t_prior >> Transformation.new(  # `\label{strp-trans}`
        middle_domain,
        output_domain,
        Function.then_expr(function),
        middle_metric,
        middle_metric,
        StabilityMap.new(lambda d_in: d_in),
    )
