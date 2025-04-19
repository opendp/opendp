# type: ignore
def make_quantile_score_candidates(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: MI,
    candidates: list[TIA],
    alpha: f64,
) -> Transformation:
    if input_domain.element_domain.nan():
        raise ValueError("input_domain members must have non-nan elements")

    check_candidates(candidates)

    size = input_domain.size
    if size is not None:
        size = u64.exact_int_cast(input_domain.size)

    alpha_num, alpha_den, size_limit = score_candidates_constants(size, alpha)

    def function(arg: list[TIA]) -> list[u64]:
        scores = compute_score(arg, candidates, alpha_num, alpha_den, size_limit)
        return Vec.from_iter(scores)  # like calling list(s) on an iter s

    return Transformation(
        input_domain=input_domain,
        output_domain=VectorDomain(
            element_domain=AtomDomain(T=u64), size=len(candidates)
        ),
        function=function,
        input_metric=input_metric,
        output_metric=LInfDistance.default(T=u64),
        stability_map=StabilityMap.new_fallible( # `\label{map}`
            score_candidates_map(
                alpha_num,
                alpha_den,
                input_domain.size.is_some(),
            )
        ),
    )
