# type: ignore
def make_stable_group_by(
    input_domain: DslPlanDomain, input_metric: FrameDistance[M], plan: DslPlan
) -> Transformation[DslPlanDomain, DslPlanDomain, FrameDistance[M], FrameDistance[M]]:
    match plan:
        case DslPlan.GroupBy(input, keys, aggs, apply, maintain_order, options):
            pass
        case _:
            raise "Expected group by in logical plan"

    if apply is not None:
        raise "apply is not currently supported"

    if maintain_order:
        raise "maintain_order is wasted compute because row ordering is protected information"

    if options != GroupbyOptions.default():
        raise "options is not currently supported"

    t_prior = input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()

    expr_domain = WildExprDomain(
        columns=middle_domain.series_domains,
        context=Context.RowByRow,
    )

    for key in keys:  # |\label{line:stable-keys}|
        key.make_stable(expr_domain, PartitionDistance(middle_metric[0]))

    for agg in aggs:  # |\label{line:infallible-aggs}|
        check_infallible(agg, Resize.Allow)

    if middle_metric[0].identifier().is_some():  # |\label{line:identifier}|
        raise "identifier is not currently supported"

    # prepare output domain series
    output_schema = middle_domain.simulate_schema(  # |\label{line:simulate-schema}|
        lambda lf: lf.group_by(keys).agg(aggs)
    )
    series_domains = [SeriesDomain.new_from_field(f) for f in output_schema]

    # prepare output domain margins
    h_keys = list(keys)

    def without_invariant(m: Margin) -> Margin:
        m.invariant = None
        return m

    margins = [
        without_invariant(m) for m in middle_domain.margins if m.by.is_subset(h_keys)
    ]

    output_domain = FrameDomain.new_with_margins(series_domains, margins)

    def stability_map(d_in: Bounds) -> Bounds:
        # |\label{line:total-contributions}|
        contributed_rows = d_in.get_bound(set()).per_group
        # |\label{line:total-groups}|
        contributed_groups = d_in.get_bound(h_keys).num_groups

        influenced_groups = option_min(contributed_rows, contributed_groups)
        if influenced_groups.is_none():
            return "an upper bound on the number of contributed rows or groups is required"

        if per_group is not None: # |\label{line:double}|
            per_group = per_group.inf_mul(2)

        bound = Bound(by=set(), per_group=per_group, num_groups=None)
        return Bounds([bound]) # |\label{line:bound}|

    t_group_agg = Transformation.new(
        middle_domain,
        output_domain,
        lambda plan: DslPlan.GroupBy(
            input=plan,
            keys=keys,
            aggs=aggs,
            apply=None,
            maintain_order=False,
            options=options,
        ),
        middle_metric,
        middle_metric,
        StabilityMap.new_fallible(stability_map),
    )

    return t_prior >> t_group_agg
