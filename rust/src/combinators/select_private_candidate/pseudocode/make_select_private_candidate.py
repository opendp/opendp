# type: ignore
def make_select_private_candidate(
    measurement: Measurement, 
    stop_probability: float, 
    threshold: float, 
    max_iterations: Optional[float],
) -> Measurement:
    if not 0 <= stop_probability < 1: # |\label{stop-prob}|
        raise "stop_probability must be in [0, 1)"

    if not threshold.is_finite():
        raise "threshold must be finite"
    
    if max_iterations is None:
        epsilon_0 = 0.0
    else:
        fewest_max_iterations = u64.inf_cast( # |\label{fewest-max-iterations}|
            E.next_up_().inf_mul(stop_probability).recip().inf_add(1.0))
    
        if max_iterations < fewest_max_iterations:
            raise f"max_iterations must not be less than {fewest_max_iterations}"
        
        x = f64.neg_inf_cast(max_iterations).neg_inf_mul(stop_probability)
        epsilon_0 = (2.0).inf_div(x.neg_inf_exp()) # |\label{epsilon-0}|


    scale = None
    if stop_probability > 0.0:
        ln_cp = (1.0).neg_inf_sub(stop_probability).inf_ln()
        scale = ln_cp.recip().neg().into_rational() # |\label{scale}|

    if max_iterations is not None:
        max_iterations = UBig.from_(max_iterations)

    def function(arg):
        num_iterations = None
        if scale is not None:
            num_iterations = UBig.ONE + sample_geometric_exp_fast(scale) # |\label{sample-geometric}|

        remaining_iterations = option_min(num_iterations, max_iterations)
        
        while True:
            score, output = measurement(arg)

            if score >= threshold:
                return score, output

            if remaining_iterations is not None:
                remaining_iterations -= UBig.ONE
                if remaining_iterations == UBig.ZERO:
                    return None

    return Measurement(
        input_domain=measurement.input_domain,
        input_metric=measurement.input_metric,
        output_measure=measurement.output_measure,
        function=function,
        privacy_map=lambda d_in: measurement.map(d_in).inf_mul(2).inf_add(epsilon_0),
    )
