# type: ignore 
def sample_geometric_exp_fast(x: RBig) -> int: 
    if x == 0: 
        return 0 

    s, t = x.into_parts() 

    while True: 
        u = sample_uniform_ubig_below(t) # |\label{line:U}| 
        d = sample_bernoulli_exp(Rational(u, t)) # |\label{line:D}| 
        if d: 
            break 

    v = sample_geometric_exp_slow(1) # |\label{line:V}| 
    z = u + t * v 
    return z // s 
