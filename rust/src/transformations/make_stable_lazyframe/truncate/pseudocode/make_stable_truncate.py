# type: ignore
def make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance[SymmetricIdDistance],
    plan: DslPlan,
) -> Transformation[
    DslPlanDomain,
    DslPlanDomain,
    FrameDistance[SymmetricIdDistance],
    FrameDistance[SymmetricDistance],
]:
    # the identifier is protected from changes, so we can use the identifier from the input metric
    # instead of the identifier from the middle_metric to match truncations
    input, truncations, truncation_bounds = match_truncations(
        plan, input_metric[0].identifier
    )

    if truncations.is_empty():
        return ValueError("failed to match truncation")

    t_prior = input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()

    for bound in truncation_bounds:
        for key in bound.by:
            # raises if the key is not infallible row-by-row
            make_stable_expr(  # |\label{line:stable-keys}|
                WildExprDomain(
                    columns=middle_domain.series_domains,
                    context=Context.RowByRow,
                ),
                L0PInfDistance(middle_metric[0]),
                key,
            )

    output_domain = middle_domain.clone()
    for truncation in truncations:  # |\label{line:truncation}|
        output_domain = truncate_domain(output_domain, truncation)

    # The matcher permits only filters followed by an optional final group-by.
    has_group_by = isinstance(truncations[-1], Truncation.GroupBy)
    output_bys = [  # |\label{line:output-bys}|
        materialize_grouping_keys(bound.by) if has_group_by else bound.by
        for bound in truncation_bounds
    ]

    def function(plan: DslPlan) -> DslPlan:
        for truncation in truncations:
            match truncation:
                case Truncation.Filter(predicate):
                    plan = DslPlan.Filter(
                        input=plan,
                        predicate=predicate,
                    )

                case Truncation.GroupBy(keys, aggs):
                    plan = DslPlan.GroupBy(
                        input=plan,
                        keys=keys,
                        aggs=aggs,
                        apply=None,
                        maintain_order=False,
                        options=GroupbyOptions.default(),
                    )
        return plan

    def stability_map(id_bounds: Bounds) -> Bounds:
        # |\label{line:total-ids}|
        total_num_ids = id_bounds.get_bound({}).per_group

        # each truncation is used to derive row bounds
        new_bounds = []
        for truncation_bound, output_by in zip(  # |\label{line:truncation-bound}|
            truncation_bounds, output_bys
        ):
            # Look up the per-id bound using the original input expression.
            row_bound = truncate_id_bound(  # |\label{line:truncate-id-bound}|
                id_bounds.get_bound(truncation_bound.by),  # |\label{line:id-bound-by}|
                truncation_bound,
                total_num_ids,
            )
            # Describe the resulting rows using materialized output columns.
            row_bound.by = output_by  # |\label{line:output-bound-by}|
            new_bounds.append(row_bound)
        return Bounds(new_bounds)

    t_truncate = Transformation.new(
        middle_domain,
        output_domain,
        Function.new(function),
        middle_metric,
        FrameDistance(SymmetricDistance),
        StabilityMap.new_fallible(stability_map),
    )
    return t_prior >> t_truncate
