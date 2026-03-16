# type: ignore

def sample_integer_envelope_accept(k, eta_numer, eta_denom, base_denom) -> bool:
    i = 0
    numer = eta_numer
    denom = base_denom
    while i < k:
        if not sample_bernoulli_rational(Rational(numer, denom)):  # |\label{line:accept_coin}|
            return False
        numer += eta_denom
        denom += eta_denom
        i += 1
    return True
