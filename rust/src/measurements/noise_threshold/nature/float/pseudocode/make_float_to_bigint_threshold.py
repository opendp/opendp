# type: ignore
def make_float_to_bigint_threshold(
    input_space: tuple[
        MapDomain[AtomDomain[TK], MapDomain[TV]], L0PI[P, AbsoluteDistance[QI]]
    ],
    k: i32,
) -> Transformation[
    MapDomain[AtomDomain[TK], AtomDomain[TV]],
    MapDomain[AtomDomain[TK], AtomDomain[IBig]],
    L0PI[P, AbsoluteDistance[QI]],
    L0PI[P, AbsoluteDistance[RBig]],
]:
    input_domain, input_metric = input_space
    if input_domain.value_domain.nan():
        raise "input_domain hashmap values may not contain NaN elements"

    def value_function(val): # |\label{line:value-function}|
        try: # |\label{line:try-from}|
            val = RBig.try_from(val)
        except Exception:
            val = RBig.ZERO

        return find_nearest_multiple_of_2k(val, k) # |\label{line:find-nearest}|

    def stability_map(d_in):
        l0, lp, li = d_in
        rounding_distance = get_rounding_distance(k, usize.from_(l0), P)

        lp = RBig.try_from(lp)
        lp = x_mul_2k(lp + rounding_distance, -k) # |\label{line:lp-x-mul-2k}|

        li = RBig.try_from(li)
        li = x_mul_2k(li + rounding_distance, -k) # |\label{line:li-x-mul-2k}|
        return l0, lp, li

    return Transformation.new(
        input_domain,
        MapDomain( # |\label{line:output-domain}|
            key_domain=input_domain.key_domain,
            value_domain=AtomDomain.default(IBig),
        ),
        Function.new(lambda x: {k: value_function(v) for k, v in x.items()}),
        input_metric,
        L0PI.default(),
        StabilityMap.new_fallible(stability_map),
    )
