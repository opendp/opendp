# type: ignore
def make_float_to_bigint_threshold(
    input_space: tuple[
        MapDomain[AtomDomain[TK], MapDomain[TV]], L0PInfDistance[P, AbsoluteDistance[QI]]
    ],
    threshold: TV,
    k: i32,
) -> Transformation[
    MapDomain[AtomDomain[TK], AtomDomain[TV]],
    MapDomain[AtomDomain[TK], AtomDomain[IBig]],
    L0PInfDistance[P, AbsoluteDistance[QI]],
    L0PInfDistance[P, AbsoluteDistance[RBig]],
]:
    input_domain, input_metric = input_space
    if input_domain.value_domain.nan():
        raise "input_domain hashmap values may not contain NaN elements"

    r_threshold = RBig.try_from(threshold)

    min_k = get_min_k(TV)
    if k < min_k:  # |\label{line:check-k}|
        raise f"k ({k}) must not be smaller than {min_k}"

    def value_function(val):  # |\label{line:value-function}|
        try:  # |\label{line:try-from}|
            val = RBig.try_from(val)
        except Exception:
            val = RBig.ZERO

        return find_nearest_multiple_of_2k(val, k)  # |\label{line:find-nearest}|

    def stability_map(d_in):
        l0, lp, li = d_in

        r_lp = RBig.try_from(lp)
        r_lp_round = get_rounding_distance(k, usize.from_(l0), P)
        r_lp = x_mul_2k(r_lp + r_lp_round, -k)  # |\label{line:lp-x-mul-2k}|

        r_li = RBig.try_from(li)
        if r_li > x_mul_2k(r_threshold, -k): # |\label{line:check-li}|
            raise f"li ({li}) must not be larger than threshold ({threshold})"
        r_li_round = get_rounding_distance(k, 1, P)
        r_li = x_mul_2k(r_li + r_li_round, -k)  # |\label{line:li-x-mul-2k}|

        return l0, r_lp, r_li

    return Transformation.new(
        input_domain,
        MapDomain(  # |\label{line:output-domain}|
            key_domain=input_domain.key_domain,
            value_domain=AtomDomain.default(IBig),
        ),
        Function.new(lambda x: {k: value_function(v) for k, v in x.items()}),
        input_metric,
        L0PI.default(),
        StabilityMap.new_fallible(stability_map),
    )
