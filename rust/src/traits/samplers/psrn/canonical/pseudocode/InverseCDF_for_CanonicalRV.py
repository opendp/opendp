# type: ignore
class InverseCDF(CanonicalRV):
    # type Edge = RBig

    def inverse_cdf(self, uniform: RBig, _refinements: usize, _R) -> RBig | None:
        f_inv = quantile_cnd(uniform, self.tradeoff, self.fixed_point)  # `\label{f_inv}`
        return f_inv * self.scale + self.shift
