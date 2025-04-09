# type: ignore
def make_int_to_bigint_threshold(
    input_space: tuple[
        MapDomain[AtomDomain[TK], AtomDomain[TV]], L0PInfDistance[P, AbsoluteDistance[QI]]
    ],
) -> Transformation[
    MapDomain[AtomDomain[TK], AtomDomain[TV]],
    MapDomain[AtomDomain[TK], AtomDomain[IBig]],
    L0PInfDistance[P, AbsoluteDistance[QI]],
    L0PInfDistance[P, AbsoluteDistance[RBig]],
]:
    input_domain, input_metric = input_space

    def stability_map(d_in):
        l0, lp, li = d_in
        lp = UBig.try_from(lp)
        li = UBig.try_from(li)
        return l0, lp, li

    return Transformation.new(
        input_domain,
        MapDomain( # |\label{line:output-domain}|
            key_domain=input_domain.key_domain,
            value_domain=AtomDomain.default(IBig),
        ),
        Function.new(lambda x: {k: IBig.from_(v) for k, v in x.items()}), # |\label{line:function}|
        input_metric,
        L0PI.default(),
        StabilityMap.new_fallible(stability_map),
    )
