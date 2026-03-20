# type: ignore
def resolved_repetitions_sample(resolved):
    if resolved["family"] == "Poisson":
        return sample_poisson(resolved["mean"])  # |\label{line:poisson}|

    x_r = rational_from_float(resolved["x"])  # |\label{line:x-rational}|
    if resolved["eta"] == 0.0:  # |\label{line:logarithmic-branch}|
        return sample_logarithmic_exp(x_r)  # |\label{line:logarithmic}|
    eta_r = rational_from_float(resolved["eta"])  # |\label{line:eta-rational}|
    return sample_truncated_negative_binomial_rational(eta_r, x_r)  # |\label{line:negative-binomial}|
