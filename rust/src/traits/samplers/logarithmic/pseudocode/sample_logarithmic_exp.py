# type: ignore

def sample_logarithmic_exp(x):
    while True:
        k = sample_geometric_exp_fast(x)
        if k == 0:
            continue

        if sample_bernoulli_rational(1 / k):
            return k