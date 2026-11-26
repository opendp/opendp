# type: ignore
def beta_via_approx_dp(epsilon: f64, delta: f64, alpha: f64) -> Fallible[f64]:
    check_epsilon(epsilon)
    check_delta(delta)
    check_alpha(alpha)
    if alpha == 1.0:
        return 0.0
    one_minus_delta = RBig(1) - RBig(delta)
    if alpha == 0.0:
        return f64.neg_inf_cast(one_minus_delta)
    epsilon = SInterval.point(epsilon)
    exp_eps_up = RBig(epsilon.exp().upper_f64())
    exp_neg_eps_down = RBig((-epsilon).exp().lower_f64())
    alpha = RBig(alpha)
    t1 = one_minus_delta - exp_eps_up * alpha
    base = max(one_minus_delta - alpha, RBig(0))
    t2 = exp_neg_eps_down * base
    return clamp(f64.neg_inf_cast(max(t1, t2, RBig(0))), 0.0, 1.0)
