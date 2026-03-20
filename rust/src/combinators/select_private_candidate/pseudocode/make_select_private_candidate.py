# type: ignore
def make_select_private_candidate(
    measurement: Measurement,
    mean: float,
    threshold: Optional[float],
    distribution,
) -> Measurement:
    if not mean.is_finite():  # |\label{line:mean-guard}|
        raise "mean must be finite"
    if threshold is not None and not threshold.is_finite():  # |\label{line:threshold-guard}|
        raise "threshold must be finite"

    resolved = distribution.resolve(mean)  # |\label{line:resolve}|
    MO.validate(threshold is not None, resolved)  # |\label{line:validate}|
    function = measurement.function
    privacy_map = MO.new_privacy_map(  # |\label{line:new-privacy-map}|
        measurement.privacy_map, threshold is not None, mean, resolved
    )

    def function_(arg):
        remaining = resolved.sample()  # |\label{line:sample}|
        best = None
        while remaining > 0:  # |\label{line:loop}|
            next_ = function(arg)  # |\label{line:eval}|
            if threshold is not None:
                if next_[0] >= threshold:  # |\label{line:threshold-branch}|
                    return (next_[0], next_[1])
            else:
                best = choose(best, next_)  # |\label{line:choose}|
            remaining -= 1
        return best

    return Measurement(
        input_domain=measurement.input_domain,
        input_metric=measurement.input_metric,
        output_measure=measurement.output_measure,
        function=function_,
        privacy_map=privacy_map,
    )
