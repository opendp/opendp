# type: ignore
def solve_nb_x_from_mean(mean: float, eta: float):
    lo = 0.0
    hi = 1.0
    while expected_nb_mean_from_x_hi(eta, hi) > mean:  # |\label{line:expand}|
        hi *= 2.0
        if not hi.is_finite():  # |\label{line:finite_guard}|
            raise "failed to solve the repetition distribution parameter from mean"
    while True:  # |\label{line:bisection}|
        mid = lo + (hi - lo) / 2.0  # |\label{line:mid}|
        if mid == lo or mid == hi:  # |\label{line:converged}|
            return hi
        if expected_nb_mean_from_x_hi(eta, mid) > mean:  # |\label{line:compare}|
            lo = mid
        else:
            hi = mid
