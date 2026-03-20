# type: ignore
def repetitions_resolve(repetitions, mean: float):
    if repetitions == "Poisson":
        if mean < 0.0:  # |\label{line:poisson-mean-guard}|
            raise "Poisson mean must be nonnegative"
        return {"family": "Poisson", "mean": mean}

    eta = repetitions["eta"]
    if mean <= 1.0:  # |\label{line:nb-mean-guard}|
        raise "negative-binomial and logarithmic means must be strictly greater than 1"
    if not eta.is_finite():  # |\label{line:eta-finite-guard}|
        raise "eta must be finite"
    if eta < 0.0:  # |\label{line:eta-sign-guard}|
        raise "eta must be nonnegative"

    x = solve_nb_x_from_mean(mean, eta)  # |\label{line:solve}|
    return {"family": "NegativeBinomial", "eta": eta, "x": x.next_up()}  # |\label{line:next-up}|
