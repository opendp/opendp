# type: ignore
def make_base_discrete_exponential(temperature: TIA, optimize: Optimize):
    if temperature <= 0:
        raise ValueError("temperature must be positive")

    if optimize == "max":
        sign = +1
    elif optimize == "min":
        sign = -1
    else:
        raise ValueError("must specify optimization")
    
    def function(scores: List[TIA]):
        noisy_score = lambda i: sign * scores[i] / temperature -ln( -ln(sample_uniform())) 
        return max(range(len(scores)), key=noisy_score)
    
    def privacy_map(d_in: TIA):
        d_in = QO.inf_cast(d_in)
        if d_in < 0:
            raise ValueError("input distance must be non-negative")
        
        if d_in == 0:
            return 0
        
        return d_in.inf_div(temperature)

    return Measurement(
        input_domain=VectorDomain(AllDomain(TIA)),
        function=function,
        input_metric=InfDifferenceDistance(TIA),
        output_metric=MaxDivergence(QO),
        privacy_map=privacy_map,
    )   