# type: ignore
def make_base_exponential_candidates_gumbel(temperature: TI):
    def function(scores: List[TI]):
        noisy_score = lambda i: scores[i] * temperature -ln( -ln(sample_uniform())) 
        return max(range(len(scores)), key=noisy_score)
    
    def privacy_relation(d_in: u32, d_out: TO):
        return d_out * temperature >= d_in

    return Transformation(
        input_domain=VectorDomain(AllDomain(TI)),
        output_domain=AllDomain(TO),
        function=function,
        input_metric=InfDifferenceDistance(TI),
        output_metric=MaxDivergence(TI),
        privacy_relation=privacy_relation,
    )