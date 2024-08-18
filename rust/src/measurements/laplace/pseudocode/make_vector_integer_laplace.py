# type: ignore
def make_vector_integer_laplace(
    input_domain: VectorDomain[AtomDomain[T]], input_metric: L1Distance[T], scale: f64
):
    if scale.is_sign_negative():
        raise ValueError("scale must not be negative")

    # conversion to rational will fail if scale is null
    r_scale = RBig.try_from(scale)

    if scale == 0.0:
        def function(x: Vec[T]):
            return x

    else:
        def function(x: Vec[T]):
            release = [IBig.from_(x_i) + sample_discrete_laplace(r_scale) for x_i in x]
            # postprocessing
            return [T.saturating_cast(r_i) for r_i in release]

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=MaxDivergence(),
        privacy_map=laplace_puredp_map(scale, relaxation=0.0),
    )
