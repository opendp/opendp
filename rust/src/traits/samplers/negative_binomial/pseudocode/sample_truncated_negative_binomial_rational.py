# type: ignore

def sample_truncated_negative_binomial_rational(eta, x) -> int:
    while True:
        k = sample_negative_binomial_rational(eta, x)  # |\label{line:ordinary_draw}|
        if k != 0:  # |\label{line:positive_branch}|
            return k
