# type: ignore
def new_negative_binomial_rdp_curve(base_curve, eta: float, x: float, mean: float):
    def curve(alpha: float):
        if alpha <= 1.0:  # |\label{line:guard}|
            return float("inf")
        eps_alpha = base_curve(alpha) 
        one_plus_eta_hi = 1.0.inf_add(eta) 
        alpha_minus_one_lo = alpha.neg_inf_sub(1.0) 
        inv_alpha_lo = 1.0.neg_inf_div(alpha) 
        one_minus_inv_alpha_hi = 1.0.inf_sub(inv_alpha_lo) 
        coeff1_hi = one_plus_eta_hi.inf_mul(one_minus_inv_alpha_hi) 
        term1_hi = coeff1_hi.inf_mul(eps_alpha) 
        log_inv_gamma_hi = log_inv_gamma_hi_from_x(x) 
        term2_num_hi = one_plus_eta_hi.inf_mul(log_inv_gamma_hi) 
        term2_hi = term2_num_hi.inf_div(alpha) 
        log_mean_hi_ = log_mean_hi(mean) 
        term3_hi = log_mean_hi_.inf_div(alpha_minus_one_lo) 
        return eps_alpha.inf_add(term1_hi).inf_add(term2_hi).inf_add(term3_hi) 
    return curve
