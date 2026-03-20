# type: ignore
def log_inv_gamma_lo_from_x(x: float):
    gamma_hi = gamma_hi_from_x(x)  # |\label{line:gamma_hi}|
    ensure_open_unit_interval(gamma_hi)  # |\label{line:interval}|
    gamma_minus_one_hi = gamma_hi.inf_sub(1.0)  # |\label{line:minus_one_hi}|
    ln_gamma_hi = gamma_minus_one_hi.inf_ln_1p()  # |\label{line:ln_gamma_hi}|
    return -ln_gamma_hi
