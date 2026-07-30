# type: ignore
def zcdp_delta(rho: float, epsilon: float) -> float:
    validate_nonnegative_finite_or_infinite(rho, epsilon)
    if rho == 0.0 or epsilon == infinity:
        return 0.0
    if rho == infinity:
        return 1.0

    # Ordinary floating-point optimization only selects candidate orders.
    # Every accepted candidate gives a valid bound.
    alpha_lower = 1.0 + sqrt(machine_epsilon)
    alpha_upper = choose_finite_search_bound(rho, epsilon)
    cks_bracket = minimize_in_log_domain(
        lambda alpha: approximate_cks_log_delta(alpha, rho, epsilon),
        alpha_lower,
        alpha_upper,
    )

    # Re-evaluate candidates with outward-rounded interval arithmetic.
    log_delta_upper = infinity
    for alpha in candidates(cks_bracket, alpha_lower, alpha_upper):
        alpha = interval_point(alpha)
        rho_interval = interval_point(rho)
        epsilon_interval = interval_point(epsilon)
        log_bound = (
            (alpha - 1) * (alpha * rho_interval - epsilon_interval)
            + alpha * log(1 - 1 / alpha)
            - log(alpha - 1)
        )
        log_delta_upper = min(log_delta_upper, log_bound.upper)

    # Asoodeh et al., Lemma 1, Equation (25), supplies a second bound
    # whenever epsilon > alpha * rho.
    if epsilon > rho:
        asoodeh_upper = interior_upper_bound(epsilon / rho)
        asoodeh_bracket = minimize_in_log_domain(
            lambda alpha: approximate_asoodeh_log_delta(
                alpha, rho, epsilon
            ),
            alpha_lower,
            asoodeh_upper,
        )
        for alpha in candidates(asoodeh_bracket):
            if epsilon <= alpha * rho:
                continue
            alpha = interval_point(alpha)
            x = (alpha - 1) * alpha * interval_point(rho)
            y = (alpha - 1) * interval_point(epsilon)
            log_bound = log_expm1(x) - log(alpha) - log_expm1(y)
            log_delta_upper = min(log_delta_upper, log_bound.upper)

    return conservative_exp_clamped_to_unit_interval(log_delta_upper)
