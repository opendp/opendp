# type: ignore
def make_canonical_noise(
    input_domain: AtomDomain[f64],
    input_metric: AbsoluteDistance[f64],
    d_in: f64,
    d_out: PrivacyGuarantee,
):
    assert not input_domain.nan(), "input data must be non-nan"  # `\label{non-nan}`
    assert (
        not d_in.is_sign_negative() and d_in.is_finite()
    )  # `\label{sensitivity-check}`

    # Only an explicitly supplied symmetric tradeoff curve satisfies the CND
    # theorem; do not derive a view from another PrivacyGuarantee representation.
    source_tradeoff = d_out.symmetric_tradeoff()
    tradeoff = lambda alpha: RBig.try_from(source_tradeoff(alpha.to_f64().value()))
    fixed_point = find_fixed_point(tradeoff)

    if fixed_point <= rbig(0) or fixed_point >= rbig(1 / 2):
        return ValueError("tradeoff curve must have a fixed point strictly between 0 and 1/2")

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

    def privacy_map(d_in_p: f64) -> PrivacyGuarantee:  # `\label{privacy-map}`
        assert 0 <= d_in_p <= d_in
        if d_in_p == 0:
            return PrivacyGuarantee.new().with_symmetric_tradeoff(
                lambda alpha: 1.0 - alpha
            )
        return PrivacyGuarantee.new().with_symmetric_tradeoff(source_tradeoff)

    return Measurement.new(
        input_domain,
        input_metric,
        output_measure=MultiDP.with_tradeoff(),
        function=function,
        privacy_map=privacy_map,
    )
