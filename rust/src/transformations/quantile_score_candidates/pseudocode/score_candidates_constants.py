# type: ignore
def score_candidates_constants(size: Optional[u64], alpha: f64) -> tuple[u64, u64, u64]:
    if alpha < 0.0 or 1.0 < alpha:
        return ValueError("alpha must be within [0, 1]")

    alpha_num_exact, alpha_den_exact = RBig.try_from(alpha).into_parts()

    if size is not None:
        # choose the finest granularity that won't overflow
        # must have that size * denom < MAX, so let denom = MAX // size
        alpha_den_approx = u64.MAX.neg_inf_div(size)
    else:
        # default to an alpha granularity of .00001
        u64.exact_int_cast(10_000)

    if alpha_den_exact < UBig.from_(alpha_den_approx):
        alpha_num = u64.try_from(alpha_num_exact.into_parts()[1])
        alpha_den = u64.try_from(alpha_den_exact)
    else:
        # numer = alpha * denom
        alpha_num_approx = u64.round_cast(alpha * f64.round_cast(alpha_den_approx))
        alpha_num, alpha_den = alpha_num_approx, alpha_den_approx

    if size is not None:
        size_limit = size
    else:
        size_limit = u64.MAX.neg_inf_div(alpha_den)

    assert alpha_num <= alpha_den  # `\label{prop-1}`
    size_limit.alerting_mul(alpha_den)  # `\label{prop-2}`

    return alpha_num, alpha_den, size_limit  # `\label{return}`
