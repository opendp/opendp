# type: ignore
def make_scalar_integer_laplace(
    input_domain: AtomDomain[T], input_metric: AbsoluteDistance[T], scale: f64
):
    if scale.is_sign_negative():
        raise ValueError("scale must not be negative")

    # conversion to rational will fail if scale is null
    r_scale = RBig.try_from(scale)

    if scale == 0.0:
        def function(x: T):
            return x
    else:
        def function(x: T):
            release = IBig.from_(x) + sample_discrete_laplace(r_scale)
            # postprocessing
            return T.saturating_cast(release)

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=MaxDivergence(),
        privacy_map=laplace_puredp_map(scale, relaxation=0.0),
    )
