# type: ignore 
def sample_bernoulli_rational(prob: RBig) -> bool: 
    numer, denom = prob.into_parts() # |\label{prob-parts}|
    sign, numer = numer.into_parts() # |\label{numer-parts}|
    if sign == Negative:
        raise ValueError("prob must not be negative")
    if numer > denom:
        raise ValueError("prob must not be greater than one")
    return numer > sample_uniform_int_below(denom) 
