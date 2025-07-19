# type: ignore
class ZeroConcentratedDivergence(SelectionMeasure):
    ONE_SHOT = True
    RV = GumbelRV

    @staticmethod
    def random_variable(shift: FBig, scale: FBig) -> GumbelRV:
        return GumbelRV(shift=shift, scale=scale)
    
    @staticmethod
    def privacy_map(d_in: f64, scale: f64, k: usize) -> f64:
        if d_in < 0:
            raise ValueError("input distance must be non-negative")

        if scale.is_zero():
            return f64.INFINITY

        return d_in.inf_div(scale).inf_mul(f64.inf_cast(k))