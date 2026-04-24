# type: ignore
# ZeroConcentratedDivergence
REPLACEMENT = True

def privacy_map(d_in: f64, scale: f64) -> f64:
    return d_in.inf_div(scale).inf_powi(ibig(2)).inf_div(8.0)
