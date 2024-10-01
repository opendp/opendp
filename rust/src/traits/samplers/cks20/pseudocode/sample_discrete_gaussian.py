# type: ignore
def sample_discrete_gaussian(scale) -> int:
    if scale == 0:
        return 0
    
    t = floor(scale) + 1 # |\label{line:t}|
    sigma2 = scale**2
    
    while True:
        candidate = sample_discrete_laplace(t) # |\label{line:candidate}|
        x = abs(candidate) - sigma2 / t
        bias = x**2 / (2 * sigma2)  # |\label{line:bias}|
        if sample_bernoulli_exp(bias): # |\label{line:bern}|
            return candidate