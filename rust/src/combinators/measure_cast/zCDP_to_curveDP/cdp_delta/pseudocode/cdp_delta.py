# type: ignore
def cdp_delta(rho: float, eps: float) -> float:
    """The Rust code may be easier to follow due to more commenting."""
    if rho.is_sign_negative():
        raise ValueError(f"rho ({rho}) must be non-negative")

    if eps.is_sign_negative():
        raise ValueError(f"epsilon ({eps}) must be non-negative")

    if rho.is_zero() or eps.is_infinite():
        return 0.0

    if rho.is_infinite():
        return 1.0

    a_max = eps.inf_add(1.0).inf_div((2.0).neg_inf_mul(rho)).inf_add(2.0)

    a_min = 1.01

    while True:
        diff = a_max - a_min

        a_mid = a_min + diff / 2.0

        if a_mid == a_max or a_mid == a_min:
            break

        # calculate derivative
        deriv = (2.0 * a_mid - 1.0) * rho - eps + a_mid.recip().neg().ln_1p()

        if deriv.is_sign_negative():
            a_min = a_mid
        else:
            a_max = a_mid

    # calculate delta
    a_1 = a_max.inf_sub(1.0)
    ar_e = a_max.inf_mul(rho).inf_sub(eps)

    try:
        t1 = a_1.inf_mul(ar_e)

    except OpenDPException:

        # if t1 is negative, then handle negative overflow by making t1 larger: the most negative finite float
        # making t1 larger makes delta larger, so it's still a valid upper bound
        if a_1.is_sign_negative() != ar_e.is_sign_negative():
            t1 = 1.7976931348623157e308  # f64::MIN
        else:
            raise

    t2 = a_max.inf_mul(a_max.recip().neg().inf_ln_1p())

    delta = t1.inf_add(t2).inf_exp().inf_div((a_max.inf_sub(1.0)))

    # delta is always <= 1
    delta.min(1.0)
