# type: ignore

def sample_negative_binomial_integer(shape, x) -> int:
    total = 0
    remaining = shape
    while remaining != 0:
        total += sample_geometric_exp_fast(x)  # |\label{line:geom_draw}|
        remaining -= 1
    return total
