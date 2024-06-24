# type: ignore
def report_noisy_max_exponential(
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
        scores = sign * scores
        noised_scores = []

        for score in scores:
            Z = ExponentialNoise()
            noised_scores.append(score + Z)

        return argmax(noised_scores)

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

