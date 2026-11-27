# type: ignore
class ZeroConcentratedDivergence:
    def replacement(self):
        return True

    def privacy_map(self, d_in, scale, k):
        if d_in == 0:
            return 0.0
        if scale == 0:
            return f64.INFINITY
        return d_in.inf_div(scale).inf_powi(2).inf_div(8.0).inf_mul(f64.inf_cast(k))
