# type: ignore
def log_inv_gamma_hi_from_x(x: float):
    gamma_lo = gamma_lo_from_x(x)  # |\label{line:gamma_lo}|
    ensure_open_unit_interval(gamma_lo)  # |\label{line:interval}|
    gamma_minus_one_lo = gamma_lo.neg_inf_sub(1.0)  # |\label{line:minus_one_lo}|
    ln_gamma_lo = gamma_minus_one_lo.neg_inf_ln_1p()  # |\label{line:ln_gamma_lo}|
    return -ln_gamma_lo
