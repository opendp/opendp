# type: ignore
def make_randomized_response_bitvec(
    input_domain: BitVectorDomain,
    input_metric: DiscreteDistance,
    f: f64,
):
    output_measure = MaxDivergence(f64)

    if not (0.0 < f <= 1):  # |\label{line:range}|
        raise Exception("Probability must be in (0.0, 1]")

    m = input_domain.max_weight
    if m is None:  # |\label{line:max-weight}|
        raise Exception(
            "make_randomized_response_bitvec requires a max_weight on the input_domain"
        )

    if input_domain.size is None:  # |\label{line:size}|
        raise Exception(
            "make_randomized_response_bitvec requires a size on the input_domain"
        )

    threshold = bitvec_flip_threshold(f)
    q = bernoulli_u64_probability(threshold)
    # Compute epsilon with directed rounding from the quantized flip probability.
    epsilon = 2 * m * log((1 - q) / q)

    def privacy_map(d_in: IntDistance):  # |\label{line:map}|
        if d_in == 0:
            return 0.0
        return epsilon

    def function(arg: BitVector) -> BitVector:  # |\label{line:fn}|
        return [bit ^ sample_bernoulli_u64(threshold) for bit in arg]

    return Measurement(
        input_domain, function, input_metric, output_measure, privacy_map
    )
