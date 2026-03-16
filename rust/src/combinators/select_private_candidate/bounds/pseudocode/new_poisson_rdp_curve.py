# type: ignore
def new_poisson_rdp_curve(base_curve, mean: float):
    def curve(alpha: float):
        if mean == 0.0:  # |\label{line:mean_zero}|
            return 0.0
        if alpha <= 1.0:  # |\label{line:guard}|
            return float("inf")
        eps_alpha = base_curve(alpha) 
        max_eps_lo = log_alpha_over_alpha_minus_one_lo(alpha) 
        if eps_alpha > max_eps_lo:  # |\label{line:admissibility}|
            return float("inf")
        inv_alpha_hi = 1.0.inf_div(alpha) 
        inv_alpha_lo = 1.0.neg_inf_div(alpha) 
        one_minus_inv_alpha_hi = 1.0.inf_sub(inv_alpha_lo) 
        one_minus_inv_alpha_minus_one_hi = one_minus_inv_alpha_hi.inf_sub(1.0) 
        ln_one_minus_inv_alpha_hi = one_minus_inv_alpha_minus_one_hi.inf_ln_1p() 
        alpha_minus_one_lo = alpha.neg_inf_sub(1.0) 
        exponent_hi = alpha_minus_one_lo.inf_mul(ln_one_minus_inv_alpha_hi) 
        power_hi = exponent_hi.inf_exp() 
        delta_hi = inv_alpha_hi.inf_mul(power_hi) 
        term1_hi = mean.inf_mul(delta_hi) 
        log_mean_hi_ = log_mean_hi(mean) 
        term2_hi = log_mean_hi_.inf_div(alpha_minus_one_lo) 
        return eps_alpha.inf_add(term1_hi).inf_add(term2_hi) 
    return curve
