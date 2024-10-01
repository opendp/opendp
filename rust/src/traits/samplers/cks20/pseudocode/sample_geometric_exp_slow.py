# type: ignore
def sample_geometric_exp_slow(x) -> int:
    k = 0
    while True:
        if sample_bernoulli_exp(x): # |\label{line:B}|
            k += 1
        else:
            return k