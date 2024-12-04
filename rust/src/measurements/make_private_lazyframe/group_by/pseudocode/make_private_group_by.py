# type: ignore
def make_private_group_by(
    input_domain,
    input_metric,
    output_measure,
    plan,
    global_scale,
    threshold,
):
    input, keys, aggs, key_sanitizer = match_group_by(plan) # |\label{line:match-group-by}|

    # 1: establish stability of input |\label{line:input_stability}|
    t_prior = input.make_stable(input_domain, input_metric)  # |\label{line:tprior}|
    middle_domain, middle_metric = t_prior.output_space()

    by = match_grouping_columns(keys)  # |\label{line:groupcols}|
    margin = middle_domain.get_margin(by, Margin.default())  # |\label{line:margin}|

    match key_sanitizer:
        case KeySanitizer.Join(labels):
            num_keys = LazyFrame.from_(labels).select([len()]).collect()
            margin.max_num_partitions = num_keys.column("len").u32().last()  # |\label{line:keys-len}|

            is_join = True
        case _:
            is_join = False

    # 2: prepare for release of `aggs` |\label{line:prep-release-aggs}|
    expr_domain = WildExprDomain(  # |\label{line:joint}|
        columns=middle_domain.series_domains,
        context=ExprContext.Grouping(by, margin),
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

    f_comp = m_exprs.function
    f_privacy_map = m_exprs.privacy_map

    # 3: prepare for release of `keys` |\label{line:prep-release-keys}|
    dp_exprs, null_exprs = zip(*((ep.expr, ep.fill) for ep in m_exprs.invoke(input)))

    # 3.1: reconcile information about the threshold |\label{line:reconcile-threshold}|
    if margin.public_info is not None or is_join:  # |\label{line:no-thresholding}|
        threshold_info = None
    elif match_filter(key_sanitizer) is not None:  # |\label{line:sanitizer-threshold}|
        name, threshold_value = match_filter(key_sanitizer)
        noise = find_len_expr(dp_exprs, name)[1]
        threshold_info = name, noise, threshold_value, False
    elif threshold is not None:  # |\label{line:constructor-threshold}|
        name, noise = find_len_expr(dp_exprs, None)
        threshold_info = name, noise, threshold_value, True
    else:  # |\label{line:not-private}|
        raise f"The key set of {by} is private and cannot be released."

    # 3.2: update key sanitizer |\label{line:final-sanitizer}|
    if threshold_info is not None:  # |\label{line:incorporate-threshold}|
        name, _, threshold_value, is_present = threshold_info
        threshold_expr = col(name).gt(lit(threshold_value))
        if not is_present and predicate is not None:  # |\label{line:new-threshold-sanitizer}|
            key_sanitizer = KeySanitizer.Filter(threshold_expr.and_(predicate))
        else:
            key_sanitizer = KeySanitizer.Filter(threshold_expr)

    elif isinstance(key_sanitizer, KeySanitizer.Join):  # |\label{line:incorporate-join-fill}|
        key_sanitizer.fill_null = []
        for dp_expr, null_expr in zip(dp_exprs, null_exprs):
            name = dp_expr.meta().output_name()
            if null_expr is None:
                raise f"fill expression for {name} is unknown"

            key_sanitizer.fill_null.append(col(name).fill_null(null_expr))
    
    # 4: build final measurement |\label{line:build-meas}|
    def function(arg: DslPlan) -> DslPlan:  # |\label{line:function}|
        output = DslPlan.GroupBy(
            input=arg,
            keys=keys,
            aggs=[p.expr for p in f_comp.eval(arg)],
            apply=None,
            maintain_order=False,
        )
        match key_sanitizer:
            case KeySanitizer.Filter(predicate):
                output = DslPlan.Filter(input=output, predicate=predicate)
            case KeySanitizer.Join(
                labels,
                how,
                left_on,
                right_on,
                options,
                fill_null,
            ):
                match how:  # |\label{line:join-how}|
                    case JoinType.Left:
                        input_left, input_right = labels, output
                    case JoinType.Right:
                        input_left, input_right = output, labels
                    case _:
                        raise "unreachable"

                output = DslPlan.HStack(
                    input=DslPlan.Join(
                        input_left,
                        input_right,
                        left_on,
                        right_on,
                        options,
                        predicates=[],
                    ),
                    exprs=fill_null,
                    options=ProjectionOptions.default(),
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

        d_out = f_privacy_map.eval((l0, l1, li))

        if margin.public_info is None or is_join:  # |\label{line:privacy-map-static-keys}|
            pass
        elif threshold is not None:  # |\label{line:privacy-map-threshold}|
            _, noise, threshold_value = threshold_info
            if li >= threshold_value:
                raise f"Threshold must be greater than {li}."
            d_instability = threshold_value.inf_sub(li)
            delta_single = integrate_discrete_noise_tail(
                noise.distribution, noise.scale, d_instability
            )
            delta_joint = 1 - (1 - delta_single).inf_powi(l0)
            d_out = MO.add_delta(d_out, delta_joint)
        else:
            raise "keys must be public if threshold is unknown"

        return d_out

    m_group_by_agg = Measurement.new(
        middle_domain,
        function,
        middle_metric,
        output_measure,
        privacy_map,
    )

    return t_prior >> m_group_by_agg
