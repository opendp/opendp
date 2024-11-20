# type: ignore
def make_select_private_candidate(
    measurement: Measurement, 
    stop_probability: float, 
    threshold: float,
) -> Measurement:
    if not 0 <= stop_probability < 1: # |\label{stop-prob}|
        raise "stop_probability must be in [0, 1)"

    if not threshold.is_finite():
        raise "threshold must be finite"
    
    scale = None
    if stop_probability > 0.0:
        ln_cp = (1.0).neg_inf_sub(stop_probability).inf_ln()
        scale = ln_cp.recip().neg().into_rational() # |\label{scale}|

    def function(arg):
        remaining_iterations = None
        if scale is not None:
            remaining_iterations = UBig.ONE + sample_geometric_exp_fast(scale) # |\label{sample-geometric}|
        
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
        privacy_map=lambda d_in: measurement.map(d_in).inf_mul(2),
    )
