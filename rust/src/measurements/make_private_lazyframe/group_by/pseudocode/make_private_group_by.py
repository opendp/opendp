# type: ignore
def make_private_group_by(
    input_domain,
    input_metric,
    output_measure,
    plan,
    global_scale,
    threshold,
):
    input_expr, keys, aggs, predicate = match_group_by(plan)

    t_prior = input_expr.make_stable(input_domain, input_metric)  # |\label{line:tprior}|
    middle_domain, middle_metric = t_prior.output_space()

    by = match_grouping_columns(keys)  # |\label{line:groupcols}|

    margin = (
        middle_domain
        .margins
        .get(by, Margin.default())
    )  # |\label{line:margin}|

    # prepare a joint measurement over all expressions
    expr_domain = ExprDomain(  # |\label{line:joint}|
        middle_domain,
        ExprContext.Aggregate(by),
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

    # reconcile information about the threshold |\label{line:reconcile-threshold}|
    dp_exprs = m_exprs.invoke((input_expr, all()))

    if margin.public_info is not None:
        threshold_info = None
    elif is_threshold_predicate(predicate) is not None:
        name, threshold_value = is_threshold_predicate(predicate)
        noise = find_len_expr(dp_exprs, name)[1]
        threshold_info = name, noise, threshold_value, False
    elif threshold is not None:
        name, noise = find_len_expr(dp_exprs, None)[1]
        threshold_info = name, noise, threshold_value, True
    else:
        raise f"The key set of {by} is private and cannot be released."

    # prepare the final_predicate to be used in the function |\label{line:final-predicate}|
    if threshold_info is not None:
        name, _, threshold_value, is_present = threshold_info
        if not is_present and predicate is not None:
            final_predicate = threshold_expr.and_(predicate)
        else:
            final_predicate = threshold_expr
    else:
        final_predicate = predicate
    
    # prepare supporting elements
    def function(arg):  # |\label{line:function}|
        output = DslPlan.GroupBy(
            input=arg,
            keys=keys,
            aggs=m_exprs((arg, all())),
            apply=None,
            maintain_order=False,
        )

        if final_predicate is not None:
            output = DslPlan.Filter(
                input=output,
                predicate=final_predicate,
            )
        return output
    
    def privacy_map(d_in):  # |\label{line:privacy-map}|
        mip = margin.get("max_influenced_partitions", default=d_in)
        mnp = margin.get("max_num_partitions", default=d_in)
        mpc = margin.get("max_partition_contributions", default=d_in)
        mpl = margin.get("max_partition_length", default=d_in)

        l0 = min(mip, mnp, d_in)
        li = min(mpc, mpl, d_in)
        l1 = l0.inf_mul(li).min(d_in)

        d_out = m_exprs.map((l0, l1, li))

        if threshold is not None:
            _, plugin, threshold_value = threshold_info
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