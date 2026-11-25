# type: ignore
def make_randomized_response(categories: set[T], prob: f64):
    input_domain = AtomDomain(T)
    input_metric = DiscreteMetric()
    output_measure = MaxDivergence()

    categories = list(categories)

    if len(categories) < 2:  # |\label{line:num_cats}|
        raise ValueError("expected at least two categories")

    num_categories = len(categories)

    if not (1 / num_categories <= prob <= 1):  # |\label{line:range}|
        raise ValueError("probability must be within [1/num_categories, 1]")

    if prob == 1 / num_categories:
        randomize_prob = 1.0
    else:
        randomize_prob = min(
            (1.0).inf_sub(prob).inf_mul(num_categories).inf_div(num_categories - 1),
            1.0,
        )
    threshold = bernoulli_u64_threshold(randomize_prob)

    # prepare constant: |\label{line:map}|
    q = bernoulli_u64_probability(threshold)
    if q == 0.0:
        c = float("inf")
    else:
        c = (1.0).inf_sub(q).inf_mul(num_categories).inf_div(q).inf_ln_1p()

    def privacy_map(d_in: u32) -> QO:
        if d_in == 0:
            return 0
        else:
            return c

    def function(truth: T) -> T:  # |\label{line:fn}|
        sample = usize.sample_uniform_int_below(num_categories)
        randomize = sample_bernoulli_u64(threshold)
        return categories[sample] if randomize or truth not in categories else truth

    return Measurement(
        input_domain, function, input_metric, output_measure, privacy_map
    )
