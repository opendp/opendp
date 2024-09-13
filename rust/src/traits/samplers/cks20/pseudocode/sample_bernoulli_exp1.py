# type: ignore
def sample_bernoulli_exp1(x) -> bool:
    k = 1
    while True:
        if bool.sample_bernoulli(x / k, false): # |\label{line:B_i}|
            k += 1
        else: 
            return is_odd(k) # |\label{line:K}|
