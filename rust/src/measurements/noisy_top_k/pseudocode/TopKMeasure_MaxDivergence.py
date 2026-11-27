# type: ignore
class MaxDivergence:
    def replacement(self):
        return False

    def privacy_map(self, d_in, scale, k):
        if d_in == 0:
            return 0.0
        if scale == 0:
            return f64.INFINITY
        return d_in.inf_div(scale).inf_mul(f64.inf_cast(k))
