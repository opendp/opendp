# type: ignore
# MaxDivergence
REPLACEMENT = False

def privacy_map(d_in: f64, scale: f64) -> f64:
    return d_in.inf_div(scale)
