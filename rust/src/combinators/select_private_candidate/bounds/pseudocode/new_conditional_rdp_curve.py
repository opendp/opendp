# type: ignore
def new_conditional_rdp_curve(base_curve, x: float):
    def curve(alpha: float):
        if alpha <= 2.0:  # |\label{line:guard}|
            return float("inf")
        eps_alpha = base_curve(alpha)  # |\label{line:eps_alpha}|
        alpha_minus_one_hi = alpha.inf_sub(1.0) 
        eps_alpha_minus_one_hi = base_curve(alpha_minus_one_hi) 
        alpha_minus_two_hi = alpha.inf_sub(2.0) 
        alpha_minus_one_lo = alpha.neg_inf_sub(1.0) 
        coeff_hi = alpha_minus_two_hi.inf_div(alpha_minus_one_lo) 
        log_inv_gamma_hi = log_inv_gamma_hi_from_x(x) 
        term1_hi = coeff_hi.inf_mul(eps_alpha_minus_one_hi) 
        term2_num_hi = 2.0.inf_mul(log_inv_gamma_hi) 
        term2_hi = term2_num_hi.inf_div(alpha_minus_one_lo) 
        return eps_alpha.inf_add(term1_hi).inf_add(term2_hi) 
    return curve
