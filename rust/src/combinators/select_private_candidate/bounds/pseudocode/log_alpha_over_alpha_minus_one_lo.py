# type: ignore
def log_alpha_over_alpha_minus_one_lo(alpha: float):
    inv_alpha_lo = 1.0.neg_inf_div(alpha)  # |\label{line:div_lo}|
    neg_inv_alpha_hi = -inv_alpha_lo
    ln_one_minus_inv_alpha_hi = neg_inv_alpha_hi.inf_ln_1p()  # |\label{line:ln1p_hi}|
    return -ln_one_minus_inv_alpha_hi
