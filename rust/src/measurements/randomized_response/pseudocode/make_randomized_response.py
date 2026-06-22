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
    
    # prepare constant: |\label{line:map}|
    if prob == 1.0:
        c = float("inf")
    else:
        c = prob.inf_div((1).neg_inf_sub(prob)) \
            .inf_mul(num_categories.inf_sub(1)) \
            .inf_ln()
    
    def privacy_map(d_in: u32) -> QO:
        if d_in == 0:
            return 0
        else: 
            return c

    def function(truth: T) -> T:  # |\label{line:fn}|
        index = find_index(categories, truth)
        sample = sample_uniform_uint_below(
            len(categories) - (1 if index is not None else 0))
        
        if index is not None and sample >= index:
            sample += 1
        
        lie = categories[sample]

        be_honest = sample_bernoulli_float(prob, False)
        is_member = index is not None
        return truth if be_honest and is_member else lie
    
    return Measurement(input_domain, function, input_metric, output_measure, privacy_map)
