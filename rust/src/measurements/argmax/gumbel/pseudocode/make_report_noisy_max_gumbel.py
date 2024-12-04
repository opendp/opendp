# type: ignore
def make_report_noisy_max_gumbel(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: RangeDistance[TIA],
    scale: QO, 
    optimize: Union[Literal["max"], Literal["min"]]
) -> Measurement:
    if input_domain.element_domain.nullable:
        raise ValueError("input domain must be non-nullable")

    if scale < 0:
        raise ValueError("scale must be non-negative")

    if optimize == "max":
        sign = +1
    elif optimize == "min":
        sign = -1
    else:
        raise ValueError("must specify optimization")

    scale_frac = Fraction(scale)

    def function(scores: list[TIA]):
        def map_gumbel(score):
            return GumbelPSRN(shift=sign * Fraction(score), scale=scale_frac)
        gumbel_scores = map(map_gumbel, scores)

        def reduce_best(a, b):
            return a if a[1].greater_than(b[1]) else b
        return reduce(reduce_best, enumerate(gumbel_scores))[0]

    def privacy_map(d_in: TIA):
        # convert to range distance
        # will multiply by 2 if not monotonic
        d_in = input_metric.range_distance(d_in)

        d_in = QO.inf_cast(d_in)
        if d_in < 0:
            raise ValueError("input distance must be non-negative")

        if d_in == 0:
            return 0

        return d_in.inf_div(scale)

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_metric=MaxDivergence(QO),
        privacy_map=privacy_map,
    )
