# type: ignore
def cdp_epsilon(rho: Q, delta: Q) -> Q:
    if rho.is_sign_negative():
        raise "rho must be non-negative"

    if delta.is_sign_negative():
        raise "delta must be non-negative"

    if rho.is_zero():
        return 0

    # checks if derivative is positive
    def deriv_pos(a):
        return rho > -log(a * delta) / (a - 1)**2

    # find bounds
    a_min = 1.01
    a_max = 2
    while not deriv_pos(a_max):
        a_max *= 2

    # optimize alpha
    while True:
        diff = a_max - a_min

        a_mid = a_min + diff / _2

        if a_mid == a_max or a_mid == a_min:
            break

        if deriv_pos(a_mid):
            a_max = a_mid
        else:
            a_min = a_mid
    
    # back out epsilon
    a_m1 = a_max.inf_sub(_1)

    numer = (a_m1.inf_div(a_max).inf_ln().inf_mul(a_m1)) \
        .inf_sub(a_max.inf_ln()) \
        .inf_add(delta.recip().inf_ln())

    denom = a_max.neg_inf_sub(_1)

    epsilon = a_max.inf_mul(rho).inf_add(numer.inf_div(denom))

    return epsilon
