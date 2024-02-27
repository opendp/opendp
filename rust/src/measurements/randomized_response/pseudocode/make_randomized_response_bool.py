# type: ignore
def make_randomized_response_bool(prob: QO, constant_time: bool):
    input_domain = AtomDomain(bool)
    output_domain = AtomDomain(bool)
    input_metric = DiscreteMetric()
    output_measure = MaxDivergence(QO)
    
    if (prob < 0.5 or prob >= 1):  # |\label{line:range}|
        raise Exception("probability must be in [0.5, 1)")

    c = prob.inf_div((1).neg_inf_sub(prob)).inf_ln()
    def privacy_map(d_in: u32) -> QO:  # |\label{line:map}|
        if d_in == 0:
            return 0
        else:
            return c

    def function(arg: bool) -> bool:  # |\label{line:fn}|
        sample = not sample_bernoulli_float(prob, constant_time)

        return arg ^ sample
    
    return Measurement(input_domain, output_domain, function, input_metric, output_measure, privacy_map)