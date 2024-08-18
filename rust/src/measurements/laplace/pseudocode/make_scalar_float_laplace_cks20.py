# type: ignore
def make_scalar_float_laplace_cks20(input_domain, input_metric, scale: QO, k):
    if scale.is_sign_negative():
        raise ValueError("scale must not be negative")
    
    k, relaxation = get_discretization_consts(k)

    if scale == 0.:
        def function(x: T):
            return x
    else:
        def function(x: T):
            return sample_discrete_laplace_Z2k(x, scale, k)
    
    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=MaxDivergence(QO),
        privacy_map=laplace_map(scale, relaxation=relaxation)
    )
