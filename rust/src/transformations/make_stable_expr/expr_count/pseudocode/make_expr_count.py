# type: ignore
def make_expr_count(
    input_domain: WildExprDomain, input_metric: L01InfDistance[MI], expr: Expr
) -> Transformation:
    match expr:  # `\label{match-strategy}`
        case Agg(Count(input, include_nulls)):
            if include_nulls:
                strategy = Strategy.Len
            else:
                strategy = Strategy.Count

        case Function(inputs, function=FunctionExpr.NullCount):
            (input,) = inputs
            strategy = Strategy.NullCount

        case Agg(NUnique(input)):
            strategy = Strategy.NUnique

        case _:
            raise ValueError("expected count, null_count, len, or n_unique expression")

    # check if input is row-by-row
    is_row_by_row = input.make_stable(
        input_domain.as_row_by_row(), input_metric
    ).is_ok()  # `\label{is-row-by-row}`

    # construct prior transformation
    t_prior = input.make_stable(input_domain, input_metric)  # `\label{t-prior}`
    middle_domain, middle_metric = t_prior.output_space()

    margin = middle_domain.context.aggregation(str(strategy))  # `\label{alignment}`

    output_domain = ExprDomain(  # `\label{output-domain}`
        column=SeriesDomain.new(  # `\label{output-series}`
            middle_domain.column.name,
            AtomDomain.default(u32),
        ),
        context=Context.Aggregation(
            margin=Margin(
                by=margin.by,
                max_length=1,
                max_groups=margin.max_groups,
                invariant=margin.invariant,
            ),
        ),
    )

    match strategy:  # `\label{will-count-all}`
        case Strategy.Len:
            will_count_all = is_row_by_row
        case Strategy.Count:
            will_count_all = is_row_by_row and not middle_domain.column.nullable
        case _:
            will_count_all = False

    invariant = margin.invariant if will_count_all else None  # `\label{invariant}`

    def function(e: Expr) -> Expr:  # `\label{function}`
        match strategy:
            case Strategy.Count:
                return e.count()
            case Strategy.NullCount:
                return e.null_count()
            case Strategy.Len:
                return e.len()
            case Strategy.NUnique:
                return e.n_unique()

    return t_prior >> Transformation.new(  # `\label{count-trans}`
        middle_domain,
        middle_metric,
        output_domain,  # `\label{output-domain-usage}`
        LpDistance.default(),
        Function.then_expr(function).fill_with(typed_lit(u32(0))),
        counting_query_stability_map(invariant),  # `\label{stability-map}`
    )
