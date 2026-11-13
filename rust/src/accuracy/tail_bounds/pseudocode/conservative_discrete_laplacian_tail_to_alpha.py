# type: ignore
def conservative_discrete_laplacian_tail_to_alpha(scale: RBig, tail: UBig) -> f64:
    check_tail_arguments(scale, RBig(tail))
    numer = f64.inf_cast(-RBig(tail) / scale).inf_exp()
    denom = f64.neg_inf_cast(RBig.ONE / scale).neg_inf_exp().neg_inf_add(1.0)
    return numer.inf_div(denom)
