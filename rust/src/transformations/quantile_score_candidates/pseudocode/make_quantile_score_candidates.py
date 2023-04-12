# type: ignore
def make_quantile_score_candidates(candidates: List[TIA], alpha: TOA):
    
    for i in range(len(candidates) - 1):
        assert candidates[i] < candidates[i + 1]

    alpha_numer, alpha_denom = alpha.into_frac(size=None)
    if alpha_numer > alpha_denom or alpha_denom == 0:
        raise ValueError("alpha must be within [0, 1]")
    
    # ensures that the function will not overflow
    size_limit = size * alpha_denom

    abs_dist_const = max(alpha_numer, alpha_denom.inf_sub(alpha_numer))
    sup_dist_const = abs_dist_const.inf_mul(2)

    def function(arg: List[TIA]):
        return score(arg, candidates, alpha_numer, alpha_denom, size_limit)
    
    def stability_map(d_in: u32):
        return TOA.inf_cast(d_in).alerting_mul(sup_dist_const)

    return Transformation(
        input_domain=VectorDomain(AtomDomain(TIA)),
        output_domain=VectorDomain(AtomDomain(TOA)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(TOA),
        stability_map=stability_map,
    )