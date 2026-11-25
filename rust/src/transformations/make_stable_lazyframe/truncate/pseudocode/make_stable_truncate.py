# type: ignore
def make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance[SymmetricIdDistance],
    matched: MatchedTruncations,
) -> Transformation[
    DslPlanDomain,
    DslPlanDomain,
    FrameDistance[SymmetricIdDistance],
    FrameDistance[SymmetricDistance],
]:
    t_prior = matched.input.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()
    stable_input = t_prior(matched.input)
    identifier = middle_metric[0].identifier
    keys = {identifier}
    keys.update(key for bound in matched.bounds for key in bound.bound.by)
    stable_keys = {}
    for key in keys:
        t_key = make_stable_expr(  # |\label{line:stable-keys}|
            WildExprDomain(
                columns=middle_domain.series_domains,
                context=Context.RowByRow,
            ),
            L0PInfDistance(middle_metric[0]),
            key,
        )
        stable_key = t_key(stable_input).expr
        if key == identifier and stable_key != key:
            raise ValueError("The identifier cannot be rewritten.")
        stable_keys[key] = stable_key

    filters, group_by = matched.filters, matched.group_by
    for index, predicate in enumerate(filters):  # |\label{line:check-rewrites}|
        filters[index] = rewrite_windows(predicate, stable_keys)

    schema = middle_domain.schema()
    validated_bounds = []
    for candidate in matched.bounds:  # |\label{line:validate-sort-keys}|
        candidate.bound.by = {stable_keys[key] for key in candidate.bound.by}
        for key in candidate.sort_keys:
            if contains_categorical(key.to_field(schema).dtype):
                raise ValueError("SortBy truncation cannot use Categorical keys")
            if not rank_key_dtype_supported(key.to_field(schema).dtype):
                raise ValueError("Unsupported SortBy truncation key dtype")
            if not is_permutation_equivariant(key, schema):
                raise ValueError(
                    "SortBy truncation keys must be permutation-equivariant"
                )
        if candidate.bound.num_groups is not None:
            for key in candidate.bound.by:
                if contains_categorical(key.to_field(schema).dtype):
                    raise ValueError(
                        "num_groups truncation cannot use Categorical keys"
                    )
                if not rank_key_dtype_supported(key.to_field(schema).dtype):
                    raise ValueError("Unsupported num_groups truncation key dtype")
        validated_bounds.append(candidate.bound)  # |\label{line:check-certificates}|
    truncation_bounds = validated_bounds

    if group_by is not None:
        group_by.keys = [stable_keys[key] for key in group_by.keys]
        allowed_keys = set(group_by.keys)
        for bound in truncation_bounds:  # |\label{line:allowed_keys_check}|
            if not bound.by.issubset(allowed_keys):
                raise ValueError(
                    "filter truncation keys must be a subset of groupby keys"
                )

    if group_by is not None:  # |\label{line:truncation}|
        output_domain = prepare_group_by(middle_domain, group_by.keys, group_by.aggs)
    else:
        output_domain = middle_domain.clone()
        for margin in output_domain.margins:
            margin.invariant = None

    has_group_by = group_by is not None

    def stability_map(id_bounds: Bounds) -> Bounds:
        # |\label{line:total-ids}|
        total_num_ids = id_bounds.get_bound({}).per_group

        # each truncation is used to derive row bounds
        new_bounds = []
        for truncation_bound in truncation_bounds:  # |\label{line:truncation-bound}|
            # Look up the per-id bound using the certified execution expression.
            row_bound = truncate_id_bound(  # |\label{line:truncate-id-bound}|
                id_bounds.get_bound(truncation_bound.by),  # |\label{line:id-bound-by}|
                truncation_bound,
                total_num_ids,
            )
            # Describe the resulting rows using materialized output columns.
            if has_group_by:
                row_bound.by = materialize_grouping_keys(
                    row_bound.by
                )  # |\label{line:output-bound-by}|
            new_bounds.append(row_bound)
        return Bounds(new_bounds)

    def function(plan):
        for predicate in filters:
            plan = DslPlan.Filter(input=plan, predicate=predicate)
        if group_by is not None:
            plan = group_by_plan(plan, group_by.keys, group_by.aggs, False)
        return plan

    t_truncate = Transformation.new(
        middle_domain,
        output_domain,
        Function.new(function),
        middle_metric,
        FrameDistance(SymmetricDistance),
        StabilityMap.new_fallible(stability_map),
    )
    return t_prior >> t_truncate
