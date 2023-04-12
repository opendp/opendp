# type: ignore
def make_sized_quantile_score_candidates(size: usize, candidates: List[TIA], alpha: TOA):
    for i in range(len(candidates) - 1):
        assert candidates[i] < candidates[i + 1]
    
    alpha_num, alpha_den = alpha.into_frac(size=size)
    if alpha_num > alpha_den or alpha_den == 0:
        raise ValueError("alpha must be within [0, 1]")
    
    # ensures that the function will not overflow
    size * alpha_den

    def function(arg: List[TIA]):
        return score(arg, candidates, alpha_num, alpha_den, size)
    
    def stability_map(d_in: IntDistance):
        return TOA.inf_cast(d_in // 2).inf_mul(4).inf_mul(alpha_den)

    return Transformation(
        input_domain=SizedDomain(VectorDomain(AtomDomain(TIA)), size),
        output_domain=VectorDomain(AtomDomain(TOA)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(),
        stability_map=stability_map,
    )