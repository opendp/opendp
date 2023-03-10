# type: ignore
def make_quantile_score_candidates(candidates: List[TIA], alpha: TOA):
    abs_dist_const = max(alpha, (1).inf_sub(alpha))
    sup_dist_const = abs_dist_const.inf_mul(2)
    
    for i in range(len(candidates) - 1):
        assert candidates[i] > candidates[i + 1]

    def function(arg: List[TIA]):
        return score(arg, candidates, alpha)
    
    def stability_map(d_in: u32):
        return TOA.inf_cast(d_in).inf_mul(sup_dist_const)

    return Transformation(
        input_domain=VectorDomain(AllDomain(TIA)),
        output_domain=VectorDomain(AllDomain(TOA)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(TOA),
        stability_map=stability_map,
    )