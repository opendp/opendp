# type: ignore
def make_sized_quantile_score_candidates(size: usize, candidates: List[TI], alpha: TO):
    abs_dist_const = 1
    inf_diff_dist_const = 2 * abs_dist_const

    def function(arg: List[TI]):
        return score(arg, candidates, alpha)
    
    def privacy_relation(d_in: u32, d_out: TO):
        return d_in * inf_diff_dist_const <= d_out

    return Transformation(
        input_domain=SizedDomain(VectorDomain(AllDomain(TI)), size),
        output_domain=VectorDomain(AllDomain(TO)),
        function=function,
        input_metric=SymmetricDistance(),
        output_metric=InfDifferenceDistance(),
        privacy_relation=privacy_relation,
    )