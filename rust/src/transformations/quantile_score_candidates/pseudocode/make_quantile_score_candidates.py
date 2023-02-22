# type: ignore
def make_quantile_score_candidates(candidates: List[TI], alpha: TO):
    abs_dist_const = max(alpha, 1 - alpha)
    sup_dist_const = 2 * abs_dist_const

    def function(arg: List[TI]):
        return score(arg, candidates, alpha)
    
    def privacy_relation(d_in: u32, d_out: TO):
        return d_in * sup_dist_const <= d_out

    return Transformation(
        input_domain=VectorDomain(AllDomain(TI)),
        output_domain=VectorDomain(AllDomain(TO)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(),
        privacy_relation=privacy_relation,
    )