# type: ignore
def make_stable_group_by(
    input_domain: DslPlanDomain, input_metric: FrameDistance[M], plan: DslPlan
) -> Transformation[DslPlanDomain, DslPlanDomain, FrameDistance[M], FrameDistance[M]]:
    if M.ORDERED:
        raise "Sample-and-aggregate requires an unordered input metric"

    input, keys, aggs, maintain_order = match_ordinary_group_by(plan)

    t_prior = input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()
    stable_input = t_prior(input)

    expr_domain = WildExprDomain(
        columns=middle_domain.series_domains,
        context=Context.RowByRow,
    )

    stable_keys = []
    for key in keys:  # |\label{line:stable-keys}|
        t_key = key.make_stable(expr_domain, L0PInfDistance(middle_metric[0]))
        stable_keys.append(t_key(stable_input).expr)

    if middle_metric[0].identifier().is_some():  # |\label{line:identifier}|
        raise "identifier is not currently supported"

    grouping_keys = set(stable_keys)
    aggs = list(aggs)
    output_domain = prepare_group_by(
        middle_domain, stable_keys, aggs
    )  # |\label{line:group-by-domain}|

    def stability_map(d_in: Bounds) -> Bounds:
        # |\label{line:total-contributions}|
        contributed_rows = d_in.get_bound(set()).per_group
        # |\label{line:total-groups}|
        contributed_groups = d_in.get_bound(grouping_keys).num_groups

        influenced_groups = option_min(contributed_rows, contributed_groups)
        if influenced_groups.is_none():
            return (
                "an upper bound on the number of contributed rows or groups is required"
            )

        per_group = influenced_groups.inf_mul(2)  # |\label{line:double}|

        bound = Bound(by=set(), per_group=per_group, num_groups=None)
        return Bounds([bound])  # |\label{line:bound}|

    t_group_agg = Transformation.new(
        middle_domain,
        output_domain,
        lambda plan: group_by_plan(plan, stable_keys, aggs, maintain_order),
        middle_metric,
        middle_metric,
        StabilityMap.new_fallible(stability_map),
    )

    return t_prior >> t_group_agg
