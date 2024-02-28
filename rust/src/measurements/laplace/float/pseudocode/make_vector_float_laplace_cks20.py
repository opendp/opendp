# type: ignore
def make_vector_float_laplace_cks20(input_domain, input_metric, scale: QO, k):
    if scale < 0:
        raise ValueError("scale must not be negative")
    
    k, relaxation = get_discretization_consts(k)

    # each value in the input is rounded
    if relaxation != 0:
        if input_domain.size is None:
            raise ValueError("domain size must be known if discretization is not exact")
        relaxation = relaxation.inf_mul(T.inf_cast(input_domain.size))

    if scale.is_zero():
        def function(x: list[T]):
            return x
    else:
        def function(x: list[T]):
            return [sample_discrete_laplace_Z2k(x_i, scale, k) for x_i in x]
    
    return Measurement(
        input_domain,
        function,
        input_metric,
        MaxDivergence(QO),
        privacy_map=laplace_map(scale, relaxation=relaxation)
    )
