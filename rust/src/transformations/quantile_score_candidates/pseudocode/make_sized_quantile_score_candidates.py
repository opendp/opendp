# type: ignore
def make_sized_quantile_score_candidates(size: usize, candidates: List[TIA], alpha: TOA):
    for i in range(len(candidates) - 1):
        assert candidates[i] > candidates[i + 1]

    def function(arg: List[TIA]):
        return score(arg, candidates, alpha)
    
    def stability_map(d_in: IntDistance):
        return TOA.inf_cast(d_in // 2).inf_mul(4)

    return Transformation(
        input_domain=SizedDomain(VectorDomain(AllDomain(TIA)), size),
        output_domain=VectorDomain(AllDomain(TOA)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(),
        stability_map=stability_map,
    )