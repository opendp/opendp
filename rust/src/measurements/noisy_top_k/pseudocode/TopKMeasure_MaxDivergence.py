# type: ignore
# MaxDivergence
def noisy_top_k(x: list[TIA], scale: f64, k: usize, negate: bool) -> list[usize]:
    return exponential_top_k(x, scale, k, negate)

def privacy_map(d_in: f64, scale: f64) -> f64:
    return d_in.inf_div(scale)
