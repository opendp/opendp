# type: ignore
def make_tulap(
    input_domain: AtomDomain[float],
    input_metric: AbsoluteDistance[float],
    epsilon: float,
    delta: float,
):
    assert not input_domain.nullable(), "input data must be non-null"
    assert epsilon >= 0 and delta >= 0, "epsilon and delta must not be negative"
    assert delta <= 1, "delta must not exceed 1"

    def function(arg: float) -> float:  # |\label{line:fn}|
        # inverse transform sampling of Tulap
        arg = arg or 0.0  # for robustness against NaN inputs
        tulap = TulapRV(shift=arg, epsilon=epsilon, delta=delta)
        return tulap.sample().value()

    def privacy_map(d_in: float) -> float:  # |\label{line:map}|
        assert 0 <= d_in <= 1
        if d_in == 0:
            return (0.0, 0.0)
        return epsilon, delta

    return Measurement(
        input_domain,
        function,
        input_metric,
        output_measure=dp.fixed_smoothed_max_divergence(),
        privacy_map=privacy_map,
    )
