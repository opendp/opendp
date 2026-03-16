# type: ignore

def sample_negative_binomial_rational(eta, x) -> int:
    m = ceil(eta)
    eta_numer, eta_denom = Rational.into_numer_denom(eta)
    base_denom = eta_denom * m

    if eta_denom == 1:
        return sample_negative_binomial_integer(m, x)  # |\label{line:integer_fast_path}|

    while True:
        k = sample_negative_binomial_integer(m, x)  # |\label{line:proposal_draw}|
        if sample_integer_envelope_accept(k, eta_numer, eta_denom, base_denom):  # |\label{line:proposal_accept}|
            return k
