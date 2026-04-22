# type: ignore
def max_divergence_validate(has_threshold: bool, repetitions):
    if has_threshold and not repetitions.is_geometric():  # |\label{line:geometric-guard}|
        raise "thresholded MaxDivergence selection requires geometric repetitions"
    if repetitions["family"] == "Poisson":  # |\label{line:poisson-guard}|
        raise "Poisson selection is not supported under MaxDivergence"
    return ()


def max_divergence_new_privacy_map(base_map, has_threshold: bool, repetitions):
    if has_threshold:
        factor = 2.0  # |\label{line:threshold-factor}|
    else:
        factor = 2.0.inf_add(repetitions["eta"])  # |\label{line:nb-factor}|

    def privacy_map(d_in):
        return base_map(d_in).inf_mul(factor)  # |\label{line:mul}|

    return privacy_map
