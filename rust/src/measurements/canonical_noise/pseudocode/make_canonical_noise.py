# type: ignore
def exp_m1_lower(epsilon):
    if epsilon >= 1024:
        return RBig(f64_MAX)
    return RBig(max(exp_m1_down(epsilon), epsilon))


def make_canonical_noise(input_domain, input_metric, d_in, d_out):
    assert not input_domain.nan()
    assert d_in >= 0 and d_in.is_finite()
    epsilons = sorted(set(CANONICAL_EPSILONS + d_out.certificate_endpoints()))
    deltas = d_out.delta_batch(epsilons)
    samples = list(zip(epsilons, deltas))
    tv = deltas[0]
    if 0 < tv < 0.125:
        extra = [tv * k for k in (0.125, 0.25, 0.5, 1, 2, 4, 8) if 0 < tv * k < 0.125]
        samples.extend(zip(extra, d_out.delta_batch(extra)))

    lines = [(exp_m1_lower(e), RBig(d)) for e, d in samples]
    c = max((1 - d) / (2 + a) for a, d in lines)
    assert 0 < c < RBig(1, 2)
    # Lower hull, restricted to [0, c].
    segments = lower_envelope(lines, c)
    compiled = CanonicalTradeoff(segments, c)
    slack = segments[0].delta

    def log_core(epsilons):
        values = []
        for epsilon in epsilons:
            a = exp_m1_lower(epsilon)
            s = first_segment_with_slope_at_most(segments, a)
            core = (
                s.eval(s.start) - a * s.start if s is not None else 1 - (2 + a) * c
            ) - slack
            values.append(log_upper(core) if core > 0 else -inf)
        return values

    profile = PrivacyProfile.new_log(log_core).with_delta_slack(slack)
    scale = RBig(d_in)

    def function(arg):
        arg = clamp_finite(arg)
        if scale == 0:
            return arg
        return PartialSample.new(CanonicalRV(RBig(arg), scale, compiled)).value()

    def privacy_map(distance):
        assert 0 <= distance <= d_in
        return PrivacyGuarantee().with_profile(
            perfect_privacy_log_profile() if distance == 0 else profile
        )

    return Measurement.new(
        input_domain, input_metric, MultiDP.default(), function, privacy_map
    )
