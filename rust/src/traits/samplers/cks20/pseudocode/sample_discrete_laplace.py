# type: ignore
def sample_discrete_laplace(scale) -> int:
    if scale == 0:
        return 0
        
    inv_scale = recip(scale)
    
    while True:
        sign = sample_standard_bernoulli()
        magnitude = sample_geometric_exp_fast(inv_scale) # |\label{line:magnitude}|
        
        if sign or magnitude != 0: # |\label{line:branching}|
            if sign:
                return magnitude
            else:
                return -magnitude