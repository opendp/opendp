# type: ignore
def make_private_group_by(
    input_domain,
    input_metric,
    output_measure,
    plan,
    global_scale,
    threshold,
):
    input_, keys, aggs, expr_threshold = match_group_by(plan)

    t_prior = input_.make_stable(input_domain, input_metric)
    middle_domain, middle_metric = t_prior.output_space()

    grouping_columns = match_grouping_columns(keys)

    margin = middle_domain \
        .margins \
        .get(grouping_columns, Margin.default())

    # prepare a join measurement over all expressions
    expr_domain = ExprDomain(
        middle_domain,
        ExprContext.Aggregate(grouping_columns),
    )

    m_exprs = make_basic_composition([
        make_private_expr(
             expr_domain,
             PartitionDistance(middle_metric),
             output_measure,
             expr,
             global_scale,
        ) for expr in aggs
    ])

    # reconcile information about the threshold |\label{reconcile-threshold}|
    dp_exprs = m_exprs.invoke((input_, all()))

    if threshold is not None and expr_threshold is not None:
        raise "two thresholds set"

    if margin.public_info is not None:
        threshold = None
    elif expr_threshold is not None:
        name, threshold = expr_threshold
        plugin = find_len_expr(dp_exprs, name.as_str())[1]
        threshold = name, plugin, threshold
    elif threshold is not None:
        name, plugin = find_len_expr(dp_exprs, None)
        threshold = name, plugin, threshold
    else:
        raise f"The key set of {grouping_columns} is unknown and cannot be released."

    # prepare supporting elements
    def function(arg):
        output = DslPlan.GroupBy(
            input=arg,
            keys=keys,
            aggs=m_exprs((arg, all())),
            apply=None,
            maintain_order=false,
        )

        if threshold is not None:
            name, _, threshold_value = threshold
            output = DslPlan.Filter(
                input=output,
                predicate=col(name).gt(lit(threshold_value)),
            )
        return output
    
    def privacy_map(d_in):
        l0 = margin.max_influenced_partitions.unwrap_or(d_in).min(d_in)
        li = margin.max_partition_contributions.unwrap_or(d_in).min(d_in)
        l1 = l0.inf_mul(li).min(d_in)

        d_out = m_exprs.map((l0, l1, li))

        if threshold is not None:
            _, plugin, threshold_value = threshold
            if li >= threshold_value:
                raise f"Threshold must be greater than {li}."
            d_instability = f64.inf_cast(threshold_value.inf_sub(li))
            delta_single = integrate_discrete_noise_tail(plugin.distribution, plugin.scale, d_instability)
            delta_joint = 1 - (1 - delta_single).inf_powi(l0)
            d_out = MO.add_delta(d_out, delta_joint)
        elif margin.public_info is None:
            raise "keys must be public if threshold is unknown"

        return d_out

    m_group_by_agg = Measurement(
        middle_domain,
        function,
        middle_metric,
        output_measure,
        privacy_map,
    )

    return t_prior >> m_group_by_agg