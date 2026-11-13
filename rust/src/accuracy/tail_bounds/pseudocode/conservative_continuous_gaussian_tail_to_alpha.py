# type: ignore
def conservative_continuous_gaussian_tail_to_alpha(scale: RBig, tail: RBig) -> f64:
    check_tail_arguments(scale, tail)
    # the SQRT_2 constant is already rounded down
    sqrt_2_ceil = SQRT_2.next_up_()

    # tail and scale division should be big rationals for precision and to avoid overflow
    t = f64.neg_inf_cast(tail / scale).neg_inf_div(sqrt_2_ceil)
    # round down to nearest smaller f32
    t = f64(f32.neg_inf_cast(t))
    # erfc error is at most 1 f32 ulp (see erfc_err_analysis.py)
    t = f32.inf_cast(erfc(t)).next_up_()

    return f64(t).inf_div(2.0)
