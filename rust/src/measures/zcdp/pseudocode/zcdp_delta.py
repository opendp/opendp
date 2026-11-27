# type: ignore
def zcdp_delta(rho: float, eps: float) -> float:
    """The Rust code may be easier to follow due to more commenting."""
    if rho < 0.0:
        raise ValueError(f"rho ({rho}) must be non-negative")

    if eps < 0.0:
        raise ValueError(f"epsilon ({eps}) must be non-negative")

    if rho.is_infinite():
        return 1.0

    if rho.is_zero() or eps.is_infinite():
        return 0.0

    lo = -52.0
    hi = min(1023.0, (F64_MAX / max(1.0, eps)).log2())
    while True:
        mid = lo + (hi - lo) / 2.0
        if mid == lo or mid == hi:
            break
        t = mid.exp2()
        alpha = 1.0 + t
        derivative = t * rho + rho / 2.0 - eps / 2.0 + (-1.0 / alpha).ln_1p() / 2.0
        if derivative.is_sign_negative():
            lo = mid
        else:
            hi = mid

    a_max = 1.0 + hi.exp2()
    # Evaluate the fixed-order bound conservatively in log space.
    alpha = SInterval.point(a_max)
    gamma = alpha * SInterval.point(rho)
    log_delta = rdp_log_delta0_on(alpha, gamma, SInterval.point(eps)).upper_f64()
    return log_to_delta_upper(max(-F64_MAX, min(0.0, log_delta)), 0.0)
