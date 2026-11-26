# type: ignore
def make_canonical_noise(input_domain, input_metric, d_in, d_out):
    assert not input_domain.nan()
    assert d_in >= 0 and d_in.is_finite()

    epsilons = sorted(set(CANONICAL_EPSILONS + d_out.certificate_endpoints()))
    samples = [(epsilon, d_out.delta(epsilon)) for epsilon in epsilons]
    # q is an outward-certified LOWER bound. Reciprocal arithmetic is exact.
    constraints = [
        (RBig(1) - RBig(delta), RBig(exp_lower(epsilon))) for epsilon, delta in samples
    ]

    def tradeoff(alpha):
        return max(max(RBig(0), a - q * alpha, (a - alpha) / q) for a, q in constraints)

    fixed_point = max(a / (RBig(1) + q) for a, q in constraints)
    assert RBig(0) < fixed_point < RBig(1) / RBig(2)

    # Construct the upper envelope of the affine constraints once.
    segments = upper_envelope(
        [(RBig(0), RBig(0))]
        + [(s, b) for a, q in constraints for s, b in [(-q, a), (-1 / q, a / q)]]
    )
    slack = min(delta for _, delta in samples)
    # Evaluate the hockey-stick dual at the maximizing vertex.
    def log_core(epsilons):
        values = []
        for epsilon in epsilons:
            if epsilon >= 1024:
                values.append(-inf)
                continue
            q = RBig(exp_lower(epsilon))
            vertex = first_segment_with_slope_at_least(segments, -q).start
            core = RBig(1) - tradeoff(vertex) - q * vertex - RBig(slack)
            values.append(log_upper(core) if core > 0 else -inf)
        return values

    profile = PrivacyProfile.new_log(log_core).with_delta_slack(slack)
    scale = RBig(d_in)

    def function(arg):
        rv = CanonicalRV(
            shift=RBig(clamp_finite(arg)),
            scale=scale,
            tradeoff=tradeoff,
            fixed_point=fixed_point,
        )
        return PartialSample.new(rv).value()

    def privacy_map(distance):
        assert 0 <= distance <= d_in
        if distance == 0:
            return PrivacyGuarantee().with_profile(perfect_privacy_log_profile())
        return PrivacyGuarantee().with_profile(profile.clone())

    return Measurement.new(
        input_domain,
        input_metric,
        MultiDP.with_profile(profile_purity(samples, d_in)),
        function,
        privacy_map,
    )
