# type: ignore
def make_randomized_response_bitvec(
    input_domain: BitVectorDomain,
    input_metric: DiscreteDistance,
    f: f64,
):
    output_measure = MaxDivergence(f64)

    if f <= 0.0 or f > 1:  # |\label{line:range}|
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

    threshold = bernoulli_u64_threshold(f / 2)
    # Compute epsilon with directed rounding from the quantized flip probability.
    epsilon = 2 * m * log((2**64 - threshold - 1) / (threshold + 1))

    def privacy_map(d_in: IntDistance):  # |\label{line:map}|
        if d_in == 0:
            return 0.0
        if d_in > 1:
            raise ValueError("d_in must be 0 or 1.")
        return epsilon

    def function(arg: BitVector) -> BitVector:  # |\label{line:fn}|
        noise_vector = [sample_bernoulli_u64(threshold) for _ in range(len(arg))]
        return xor(arg, noise_vector)

    return Measurement(
        input_domain, function, input_metric, output_measure, privacy_map
    )
