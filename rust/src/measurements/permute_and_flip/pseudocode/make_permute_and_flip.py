# type: ignore
def make_permute_and_flip(
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
        
        best_score = max(scores)
        indexes = range(len(scores))
        shuffled_indexes = shuffle(indexes)

        for current_index in shuffled_indexes:
            coin_bias = (best_score - scores[current_index])/scale 
            if Bern(coin_bias):
                return current_index

    def privacy_map(d_in: TIA):
        # convert to range distance
        # will multiply by 2 if not monotonic
        d_in = input_metric.range_distance(d_in)

        d_in = QO.inf_cast(d_in)
        if d_in < 0:
            raise ValueError("input distance must be non-negative")

        if d_in == 0:
            return 0

        return d_in.inf_div(scale_frac)

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_metric=MaxDivergence(QO),
        privacy_map=privacy_map,
    )

