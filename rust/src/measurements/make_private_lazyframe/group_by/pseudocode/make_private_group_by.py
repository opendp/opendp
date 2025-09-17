# type: ignore
def make_private_group_by(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance[MI],
    output_measure: MO,
    plan: DslPlan,
    global_scale: Optional[f64],
    threshold: Optional[u32],
):
    input, group_by, aggs, key_sanitizer = match_group_by(plan) # |\label{line:match-group-by}|

    # 1: establish stability of `group_by` |\label{line:input_stability}|
    t_prior = input.make_stable(input_domain, input_metric)  # |\label{line:tprior}|
    middle_domain, middle_metric = t_prior.output_space()

    for expr in group_by: # |\label{line:group-by-stability}|
        # grouping keys must be stable
        t_group_by = expr.make_stable(
            WildExprDomain(
                columns=middle_domain.series_domains,
                context=ExprContext.RowByRow,
            ),
            L0PInfDistance(middle_metric[0]),
        )

        series_domain = t_group_by.output_domain.column
        try:
            domain = series_domain.element_domain(CategoricalDomain)
        except Exception:
            pass
            
        if domain is not None and domain.categories() is None:
            raise "Categories are data-dependent, which may reveal sensitive record ordering."

    group_by_id = HashSet.from_iter(group_by)  # |\label{line:groupcols}|
    margin = middle_domain.get_margin(group_by_id)  # |\label{line:margin}|

    # 2: prepare for release of `aggs` |\label{line:prep-release-aggs}|
    match key_sanitizer:
        case KeySanitizer.Join(keys):
            num_keys = LazyFrame.from_(keys).select([len()]).collect()
            margin.max_num_partitions = num_keys.column("len").u32().last()  # |\label{line:keys-len}|
            is_join = True
        case _:
            is_join = False

    m_expr_aggs = [
        make_private_expr(
            WildExprDomain(  # |\label{line:joint}|
                columns=middle_domain.series_domains,
                context=ExprContext.Aggregation(margin),
            ),
            PartitionDistance(middle_metric),
            output_measure,
            expr,
            global_scale,
        ) for expr in aggs
    ]
    m_aggs = make_composition(m_expr_aggs)

    f_comp = m_aggs.function
    f_privacy_map = m_aggs.privacy_map

    # 3: prepare for release of `keys` |\label{line:prep-release-keys}|
    dp_exprs, null_exprs = zip(*((plan.expr, plan.fill) for plan in m_aggs.invoke(input)))

    # 3.2: reconcile information about the threshold |\label{line:reconcile-threshold}|
    if margin.invariant is not None or is_join:  # |\label{line:no-thresholding}|
        threshold_info = None
    elif match_filter(key_sanitizer) is not None:  # |\label{line:sanitizer-threshold}|
        name, threshold_value = match_filter(key_sanitizer)
        noise = find_len_expr(dp_exprs, name)[1]
        threshold_info = name, noise, threshold_value, False
    elif threshold is not None:  # |\label{line:constructor-threshold}|
        name, noise = find_len_expr(dp_exprs, None)
        threshold_info = name, noise, threshold, True
    else:  # |\label{line:not-private}|
        raise f"The key set of {group_by_id} is private and cannot be released."

    # 3.3: update key sanitizer |\label{line:final-sanitizer}|
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
            keys=group_by,
            aggs=[p.expr for p in f_comp.eval(arg)],
            apply=None,
            maintain_order=False,
        )
        match key_sanitizer:
            case KeySanitizer.Filter(predicate):
                output = DslPlan.Filter(input=output, predicate=predicate)
            case KeySanitizer.Join(
                how,
                left_on,
                right_on,
                options,
                fill_null,
                keys=labels,
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

    def privacy_map(d_in: Bounds):  # |\label{line:privacy-map}|
        bound = d_in.get_bound(group_by_id)

        # incorporate all information into optional bounds
        l0 = option_min(bound.num_groups, margin.max_groups)
        li = option_min(bound.per_group, margin.max_length)
        l1 = d_in.get_bound(HashSet.new()).per_group  # |\label{line:trivial-l1}|

        # reduce optional bounds to concrete bounds
        if l0 is not None and l1 is not None and li is not None:
            pass
        elif l1 is not None:
            l0 = l0 or l1 # |\label{line:l0-from-l1}|
            li = li or l1 # |\label{line:li-from-l1}|
        elif l0 is not None and li is not None:
            l1 = l0.inf_mul(li) # |\label{line:l1-from-l0-li}|
        else: # |\label{line: helpful_error_message}|            
            raise f"num_groups ({l0}), total contributions ({l1}), and per_group ({li}) are not sufficiently well-defined."

        d_out = f_privacy_map.eval((l0, l1, li))

        if margin.invariant is not None or is_join:  # |\label{line:privacy-map-static-keys}|
            pass
        elif threshold_info is not None:  # |\label{line:privacy-map-threshold}|
            _, noise, threshold_value, _ = threshold_info
            if li >= threshold_value:
                raise f"Threshold must be greater than {li}."
            
            d_instability = threshold_value.neg_inf_sub(li)
            delta_single = integrate_discrete_noise_tail(
                noise.distribution, noise.scale, d_instability
            )
            delta_joint = (1).inf_sub(
                (1).neg_inf_sub(delta_single).neg_inf_powi(IBig.from_(l0))
            )
            d_out = MO.add_delta(d_out, delta_joint)
        else:
            raise "the key-set is sensitive"

        return d_out

    return t_prior >> Measurement.new(
        middle_domain,
        function,
        middle_metric,
        output_measure,
        privacy_map,
    )
