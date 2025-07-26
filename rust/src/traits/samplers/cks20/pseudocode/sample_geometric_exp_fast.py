# type: ignore
def sample_geometric_exp_fast(x) -> int:
    if x == 0:
        return 0
    
    s, t = Rational.into_numer_denom(x)
    
    while True:
        u = Integer.sample_uniform_int_below(t) # |\label{line:U}|
        d = bool.sample_bernoulli_exp(Rational(u, t)) # |\label{line:D}|
        if d:
            break
        
    v = sample_geometric_exp_slow(1) # |\label{line:V}|
    z = u + t * v
    return z // s