# type: ignore
def conservative_continuous_gaussian_tail_to_alpha(scale: f64, tail: f64) -> f64:
    # the SQRT_2 constant is already rounded down
    SQRT_2_CEIL = SQRT_2.next_up_()

    t = tail.neg_inf_div(scale).neg_inf_div(SQRT_2_CEIL)
    # round down to nearest smaller f32
    t = f64(f32.neg_inf_cast(t))
    # erfc error is at most 1 f32 ulp (see erfc_err_analysis.py)
    t = f32.inf_cast(erfc(t)).next_up_()

    f64(t).inf_div(2.0)
