# type: ignore
def ensure_open_unit_interval(x: float):
    if x <= 0.0 or x >= 1.0:  # |\label{line:guard}|
        raise "failed to compute the repetition distribution parameters"
    return ()
