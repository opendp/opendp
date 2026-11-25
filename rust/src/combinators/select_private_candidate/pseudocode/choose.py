# type: ignore
def choose(best, next_):
    if next_[0].is_nan():  # |\label{line:nan-guard}|
        return best
    if best is None:  # |\label{line:best-none}|
        return next_
    if best[0] > next_[0]:  # |\label{line:compare}|
        return best
    return next_
