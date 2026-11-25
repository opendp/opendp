# type: ignore
def make_stable_group_by(
    input_domain: DslPlanDomain, input_metric: FrameDistance[M], plan: DslPlan
) -> Transformation[DslPlanDomain, DslPlanDomain, FrameDistance[M], FrameDistance[M]]:
    input, keys, aggs, maintain_order = match_ordinary_group_by(plan)
    if maintain_order:
        raise "maintain_order is wasted compute because row ordering is protected information"

    t_prior = input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()

    expr_domain = WildExprDomain(
        columns=middle_domain.series_domains,
        context=Context.RowByRow,
    )

    stable_keys = []
    for key in keys:  # |\label{line:stable-keys}|
        t_key = key.make_stable(expr_domain, L0PInfDistance(middle_metric[0]))
        if not has_canonical_representatives(t_key.output_domain.column.dtype()):
            raise ValueError("noncanonical grouping key")
        stable_keys.append(t_key(input).expr)

    if middle_metric[0].identifier().is_some():  # |\label{line:identifier}|
        raise "identifier is not currently supported"

    grouping_keys = set(keys)
    output_domain = group_by_output_domain(
        middle_domain, grouping_keys, stable_keys, aggs
    )

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


def group_by_plan(plan, keys, aggs, maintain_order):
    return DslPlan.GroupBy(
        input=plan,
        keys=keys,
        aggs=aggs,
        apply=None,
        maintain_order=maintain_order,
        options=GroupbyOptions.default(),
    )


def match_ordinary_group_by(plan):
    if not isinstance(plan, DslPlan.GroupBy):
        raise ValueError("Expected group-by")
    if (
        plan.apply is not None
        or plan.predicates
        or plan.options != GroupbyOptions.default()
    ):
        raise ValueError("Unsupported group-by options")
    return plan.input, plan.keys, plan.aggs, plan.maintain_order
