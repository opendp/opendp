# type: ignore
def make_quantile_score_candidates(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: MI,
    candidates: list[TIA],
    alpha: A
) -> Transformation:

    input_domain.element_domain.assert_non_null()

    for i in range(len(candidates) - 1):
        assert candidates[i] < candidates[i + 1]

    alpha_numer, alpha_denom = alpha.into_frac(size=None)
    if alpha_numer > alpha_denom or alpha_denom == 0:
        raise ValueError("alpha must be within [0, 1]")

    if input_domain.size is not None:
        # to ensure that the function will not overflow
        input_domain.size.inf_mul(alpha_denom)
        size_limit = input_domain.size
    else:
        size_limit = (usize.MAX).neg_inf_div(alpha_den)

    def function(arg: list[TIA]) -> list[usize]:
        return compute_score(arg, candidates, alpha_numer, alpha_denom, size_limit)

    if input_domain.size is not None:
        def stability_map(d_in: u32) -> usize:
            return TOA.inf_cast(d_in // 2).inf_mul(2).inf_mul(alpha_denom)
    else:
        abs_dist_const: usize = max(alpha_numer, alpha_denom.inf_sub(alpha_numer))
        stability_map = new_stability_map_from_constant(abs_dist_const, QO=usize)

    return Transformation(
        input_domain=input_domain,
        output_domain=VectorDomain(
            element_domain=AtomDomain(T=usize), 
            size=len(candidates)),
        function=function,
        input_metric=input_metric,
        output_metric=LInfDistance(Q=usize),
        stability_map=stability_map,
    )
