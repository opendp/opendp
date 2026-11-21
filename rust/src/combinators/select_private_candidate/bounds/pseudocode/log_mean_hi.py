# type: ignore
def log_mean_hi(mean: float):
    if not (mean > 0.0 and mean.is_finite()):  # |\label{line:guard}|
        raise "mean must be positive and finite"
    mean_minus_one_hi = mean.inf_sub(1.0)  # |\label{line:minus_one_hi}|
    return mean_minus_one_hi.inf_ln_1p()  # |\label{line:ln1p_hi}|
