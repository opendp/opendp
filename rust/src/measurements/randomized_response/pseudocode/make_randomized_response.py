# type: ignore
def make_randomized_response(categories: set[T], prob: QO):
    input_domain = AtomDomain(bool)
    output_domain = AtomDomain(bool)
    input_metric = DiscreteMetric()
    output_measure = MaxDivergence(QO)

    categories = list(categories)

    if len(categories) < 2:  # |\label{line:num_cats}|
        raise ValueError("expected at least two categories")
    
    num_categories = len(categories)

    if not (1 / num_categories <= prob < 1):  # |\label{line:range}|
        raise ValueError("probability must be within [1/num_categories, 1)")
    
    # prepare constant: |\label{line:map}|
    c = p.inf_div((1).neg_inf_sub(prob)) \
        .inf_mul(num_categories.inf_sub(1)) \
        .inf_ln()
    
    def privacy_map(d_in: u32) -> QO:
        if d_in == 0:
            return 0
        else: 
            return c

    def function(truth: bool) -> bool:  # |\label{line:fn}|
        index = categories.index(truth)
        sample = usize.sample_uniform_int_below(
            len(num_categories) - (0 if index == -1 else 1))
        
        if index != -1 and sample >= index:
            sample += 1
        
        lie = categories[sample]

        be_honest = sample_bernoulli_float(prob, false)
        is_member = index != -1
        return truth if be_honest and is_member else lie
    
    return Measurement(input_domain, output_domain, function, input_metric, output_measure, privacy_map)