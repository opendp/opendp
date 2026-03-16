# type: ignore
def sample_poisson_0_1(lmbda):
    if lmbda == 0:
        return 0

    while True:
        k = 0
        while sample_bernoulli_rational(lmbda):  # |\label{line:prop}|
            k += 1

        i = 2
        accepted = True
        while i <= k:
            if not sample_bernoulli_rational(1 / i):  # |\label{line:acc}|
                accepted = False
                break
            i += 1

        if accepted:  # |\label{line:returnk}|
            return k