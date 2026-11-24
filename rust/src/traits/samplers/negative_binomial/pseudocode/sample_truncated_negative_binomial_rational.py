# type: ignore


def sample_truncated_negative_binomial_rational(eta, x) -> int:
    if eta == 1:
        return sample_geometric_exp_fast(x) + 1  # |\label{line:geometric}|

    if eta < 1:
        eta_numer, eta_denom = Rational.into_numer_denom(eta)
        eta_numer += eta_denom
        base_denom = eta_denom * 2
        while True:
            k = sample_geometric_exp_fast(x)  # |\label{line:positive_draw}|
            if sample_integer_envelope_accept(
                k, eta_numer, eta_denom, base_denom
            ):  # |\label{line:positive_accept}|
                return k + 1  # |\label{line:positive_return}|

    while True:
        k = sample_negative_binomial_rational(eta, x)  # |\label{line:ordinary_draw}|
        if k != 0:  # |\label{line:positive_branch}|
            return k
