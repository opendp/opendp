# type: ignore
class InverseCDF(CanonicalRV):
    # type Edge = FBig

    def inverse_cdf(self, r_unif: RBig, refinements: usize, R) -> FBig | None:
        precision = refinements + 1
        r_unif_comp = RBig.ONE - r_unif  # `\label{line:complement}`
        f_unif_comp = FBig.from_(r_unif_comp, R.C).with_precision(precision).value() # `\label{line:f_uni}`

        # infinity is not in the range
        if f_unif_comp == FBig.ZERO: # `\label{line:infinity}`
            return

        f_exp = (-f_unif_comp.ln()).with_rounding(R) # `\label{line:ln}`

        f_exp *= self.scale.with_rounding()
        f_exp += self.shift.with_rounding()

        return f_exp.with_rounding()
