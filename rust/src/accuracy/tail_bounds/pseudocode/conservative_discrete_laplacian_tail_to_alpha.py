# type: ignore
def conservative_discrete_laplacian_tail_to_alpha(scale: f64, tail: u32) -> f64:
    numer = f64.inf_cast(-tail / scale).inf_exp()
    denom = f64.neg_inf_cast(1.0 / scale).neg_inf_exp().neg_inf_add(1.0)
    return numer.inf_div(denom)
