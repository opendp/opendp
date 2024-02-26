# type: ignore
def make_scalar_integer_laplace(input_domain, input_metric, scale: QO):
    if scale.is_sign_negative():
        raise ValueError("scale must not be negative")
    
    # conversion to rational will fail if scale is null
    r_scale = RBig.try_from(scale)

    if scale.is_zero():
        def function(x: T):
            return x
    else:
        def function(x: T):
            release = IBig(x) + sample_discrete_laplace(r_scale)
            # postprocessing
            return T.saturating_cast(release)
    
    return Measurement(
        input_domain,
        function,
        input_metric,
        MaxDivergence(QO),
        privacy_map=laplace_map(scale, relaxation=0.)
    )
        