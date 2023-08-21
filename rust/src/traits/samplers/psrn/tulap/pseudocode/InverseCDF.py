# type: ignore

class TulapRV(object):
    def __init__(self, shift, epsilon, delta) -> None:
        self.shift = shift
        self.exp_eps = Fraction(epsilon.neg_inf_exp())
        self.exp_neg_eps = Fraction((-epsilon).inf_exp())
        self.c = (1 - delta) / (1 + self.exp_eps)
        self.delta = delta
        self.uniform = UniformPSRN()

        if c >= 0.5:
            raise ValueError("c must be less than 1/2")

    def q_cnd(self, unif) -> Fraction | None:  # CND quantile function for f
        if unif < c:
            return self.q_cnd(1 - self.f(unif)) - 1
        elif unif <= 1 - self.c:  # the linear function
            num = unif - 1 / 2
            den = 1 - 2 * self.c
            if den.is_zero():
                return
            return num / den
        else:
            return self.q_cnd(self.f(1 - unif)) + 1

    def f(self, unif):
        t1 = 1 - self.delta - self.exp_eps * unif
        t2 = self.exp_neg_eps * (1 - self.delta - unif)
        return max(t1, t2, 0)

    def edge(self, r_unif, _refinements, _R):
        return self.q_cnd(r_unif) + self.shift
