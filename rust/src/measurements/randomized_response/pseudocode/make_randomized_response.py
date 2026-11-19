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

    uniform = prob == 1 / num_categories
    threshold = bernoulli_u64_threshold(prob)

    # prepare constant: |\label{line:map}|
    if uniform:
        c = 0.0
    elif prob == 1.0:
        c = float("inf")
    else:
        # Actual truth probability is (threshold + 1) / 2**64.
        odds = (threshold + 1).inf_div(2**64 - threshold - 1)
        c = odds.inf_mul(num_categories.inf_sub(1)).inf_ln()

    def privacy_map(d_in: u32) -> QO:
        if d_in == 0:
            return 0
        else:
            return c

    def function(truth: T) -> T:  # |\label{line:fn}|
        if uniform:
            return categories[usize.sample_uniform_int_below(num_categories)]

        index = categories.index(truth) if truth in categories else -1
        sample = usize.sample_uniform_int_below(
            num_categories - (0 if index == -1 else 1)
        )

        if index != -1 and sample >= index:
            sample += 1

        lie = categories[sample]

        be_honest = sample_bernoulli_u64(threshold)
        is_member = index != -1
        return truth if be_honest and is_member else lie

    return Measurement(
        input_domain, function, input_metric, output_measure, privacy_map
    )
