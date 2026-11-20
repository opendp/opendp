# type: ignore
def make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance[SymmetricIdDistance],
    input: DslPlan,
    truncations: Vec[Truncation],
    truncation_bounds: Vec[Bound],
) -> Transformation[
    DslPlanDomain,
    DslPlanDomain,
    FrameDistance[SymmetricIdDistance],
    FrameDistance[SymmetricDistance],
]:
    if not truncations:
        raise ValueError("failed to match truncation")

    t_prior = input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()

    # Recheck the exact candidate certificates against the stable input schema.
    schema = middle_domain.schema()
    unmatched = list(truncation_bounds)
    if isinstance(truncations[-1], Truncation.GroupBy):
        if not unmatched:
            raise ValueError("missing group-by truncation bound")
        unmatched.pop()

    for truncation in truncations:  # |\label{line:validate-sort-keys}|
        if not isinstance(truncation, Truncation.Filter):
            continue
        validated = match_truncation_predicate(
            truncation.predicate, middle_metric[0].identifier, schema
        )
        if validated is None:
            raise ValueError("failed to validate truncation")
        for bound in validated:  # |\label{line:check-certificates}|
            if bound not in unmatched:
                raise ValueError("validated truncation bounds changed")
            unmatched.remove(bound)
    if unmatched:
        raise ValueError("validated truncation bounds changed")

    # Truncation executes the original keys, not rewritten stable expressions.
    stable_input = t_prior(input)
    for bound in truncation_bounds:
        for key in bound.by:
            t_key = make_stable_expr(  # |\label{line:stable-keys}|
                WildExprDomain(
                    columns=middle_domain.series_domains,
                    context=Context.RowByRow,
                ),
                L0PInfDistance(middle_metric[0]),
                key,
            )
            if t_key(stable_input).expr != key:  # |\label{line:check-rewrites}|
                raise ValueError("truncation key requires a rewrite")

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
