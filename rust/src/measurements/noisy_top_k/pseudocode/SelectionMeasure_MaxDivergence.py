# type: ignore
class MaxDivergence(SelectionMeasure):
    ONE_SHOT = False
    RV = ExponentialRV

    @staticmethod
    def random_variable(shift: FBig, scale: FBig) -> ExponentialRV:
        return ExponentialRV(shift=shift, scale=scale)
    
    @staticmethod
    def privacy_map(d_in: f64, scale: f64, k: usize) -> f64:
        if d_in < 0:
            raise ValueError("input distance must be non-negative")

        if scale.is_zero():
            return f64.INFINITY

        return d_in.inf_div(scale).inf_mul(f64.inf_cast(k))