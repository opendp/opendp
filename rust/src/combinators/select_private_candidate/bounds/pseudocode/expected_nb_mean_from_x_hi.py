# type: ignore
def expected_nb_mean_from_x_hi(eta: float, x: float):
    q_hi = (-x).inf_exp()  # |\label{line:q_hi}|
    gamma_lo = gamma_lo_from_x(x)  # |\label{line:gamma_lo}|
    ensure_open_unit_interval(gamma_lo)  # |\label{line:interval}|

    if eta == 0.0:  # |\label{line:logarithmic_branch}|
        log_inv_gamma_lo = log_inv_gamma_lo_from_x(x) 
        denom_lo = gamma_lo.neg_inf_mul(log_inv_gamma_lo) 
        if denom_lo <= 0.0:
            return float("inf")
        return q_hi.inf_div(denom_lo) 

    gamma_eta_hi = gamma_pow_eta_hi_from_x(eta, x)  # |\label{line:gamma_eta_hi}|
    one_minus_gamma_eta_lo = 1.0.neg_inf_sub(gamma_eta_hi) 
    if one_minus_gamma_eta_lo <= 0.0:
        return float("inf")
    denom_lo = gamma_lo.neg_inf_mul(one_minus_gamma_eta_lo) 
    if denom_lo <= 0.0:
        return float("inf")
    num_hi = eta.inf_mul(q_hi) 
    return num_hi.inf_div(denom_lo) 
