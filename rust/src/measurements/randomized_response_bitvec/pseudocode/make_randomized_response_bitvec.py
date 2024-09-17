# type: ignore
def make_randomized_response_bitvec(
        input_domain: BitVectorDomain, 
        input_metric: DiscreteDistance, 
        f: f64, 
        constant_time: bool):
    output_measure = MaxDivergence(f64)
    
    if f <= 0.0 or f > 1: # |\label{line:range}|
        raise Exception("Probability must be in (0.0, 1]")
    
    m = input_domain.max_weight
    if m is None: 
        raise Exception("make_randomized_response_bitvec requires a max_weight on the input_domain")
    
    epsilon = 2 * m * log((2 - f) / f)
    def privacy_map(d_in: IntDistance): # |\label{line:map}|
        if d_in == 0:
            return 0.
        if d_in > 1:
            raise ValueError("d_in must be 0 or 1.")
        return epsilon
    def function(arg: BitVector) -> BitVector: # |\label{line:fn}|
        k = len(arg)
        noise_vector = [bool.sample_bernoulli(f/2, constant_time) for _ in range(k)]
        return xor(arg, noise_vector)
    
    return Measurement(input_domain, function, input_metric, output_measure, privacy_map)