# type: ignore
def beta_via_approx_dp(epsilon: f64, delta: f64, alpha: f64) -> Fallible[f64]:
    check_epsilon(epsilon)
    check_delta(delta)
    check_alpha(alpha)
    one_minus_delta = SInterval.point(1.0) - SInterval.point(delta)
    if alpha == 0.0:
        return one_minus_delta.lower_f64()
    if alpha == 1.0 or epsilon >= 1024.0:
        return 0.0

    epsilon = SInterval.point(epsilon)
    alpha = SInterval.point(alpha)
    zero = SInterval.point(0.0)
    z = epsilon + alpha.ln()
    t1 = zero if z.upper_f64() >= 0.0 else one_minus_delta - z.exp()
    base = (one_minus_delta - alpha).max(zero)
    t2 = (-epsilon).exp() * base
    return t1.max(t2).max(zero).lower_f64()
