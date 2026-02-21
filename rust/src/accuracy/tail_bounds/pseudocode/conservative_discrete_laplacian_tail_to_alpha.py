# type: ignore
def conservative_discrete_laplacian_tail_to_alpha(scale: f64, tail: u32) -> f64:
    t = f64.neg_inf_cast(tail)
    numer = t.neg_inf_div(-scale).inf_exp()
    denom = scale.recip().neg_inf_exp().neg_inf_add(1.0)
    return numer.inf_div(denom)
