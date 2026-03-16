# type: ignore
def renyi_divergence_validate(has_threshold: bool, repetitions):
    if has_threshold and not repetitions.is_geometric():  # |\label{line:geometric-guard}|
        raise "thresholded Renyi-Divergence selection requires geometric repetitions"
    return ()


def renyi_divergence_new_privacy_map(base_map, has_threshold: bool, mean: float, repetitions):
    def privacy_map(d_in):
        base_curve = base_map(d_in)  # |\label{line:base-curve}|
        if repetitions["family"] == "NegativeBinomial" and has_threshold:  # |\label{line:conditional-branch}|
            return new_conditional_rdp_curve(base_curve, repetitions["x"])
        if repetitions["family"] == "NegativeBinomial":  # |\label{line:nb-branch}|
            return new_negative_binomial_rdp_curve(
                base_curve, repetitions["eta"], repetitions["x"], mean
            )
        return new_poisson_rdp_curve(base_curve, mean)  # |\label{line:poisson-branch}|

    return privacy_map
