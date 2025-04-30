# type: ignore
def make_canonical_noise(
    input_domain: AtomDomain[f64],
    input_metric: AbsoluteDistance[f64],
    d_in: f64,
    d_out: tuple[f64, f64],
):
    assert not input_domain.nan(), "input data must be non-nan" # `\label{non-nan}`
    assert not d_in.is_sign_negative() and d_in.is_finite() # `\label{sensitivity-check}`

    tradeoff, fixed_point = approximate_to_tradeoff(d_out)
    r_d_in = RBig.try_from(d_in)

    def function(arg: f64) -> f64:  # `\label{function}`
        try:  # `\label{try-cast-arg}`
            arg = RBig.try_from(arg)
        except Exception:
            arg = RBig(0)
        
        canonical_rv = CanonicalRV(  # `\label{canonical-rv}`
            shift=arg, scale=r_d_in, tradeoff=tradeoff, fixed_point=fixed_point
        )
        return PartialSample.new(canonical_rv).value()  # `\label{sample-value}`

    def privacy_map(d_in_p: f64) -> f64:  # `\label{privacy-map}`
        assert 0 <= d_in_p <= d_in
        if d_in == 0:
            return (0.0, 0.0)
        return d_out

    return Measurement.new(
        input_domain,
        function,
        input_metric,
        output_measure=approximate(max_divergence()),
        privacy_map=privacy_map,
    )
