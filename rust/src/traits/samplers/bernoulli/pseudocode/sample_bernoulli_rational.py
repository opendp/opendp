# type: ignore
# returns a single bit with some probability of success
def sample_bernoulli_rational(prob: RBig, trials: Optional[int]) -> bool:
    numer, denom = prob.into_parts()
    return numer > UBig.sample_uniform_int_below(denom, trials)
