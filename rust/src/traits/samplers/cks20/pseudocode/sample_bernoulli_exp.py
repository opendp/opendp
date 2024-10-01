# type: ignore
def sample_bernoulli_exp(x) -> bool:
    while x >= 1:
        if sample_bernoulli_exp1(1):  # |\label{line:B_i}|
            x -= 1
        else: 
            return False
    return sample_bernoulli_exp1(x)   # |\label{line:C}|
