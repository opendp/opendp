from math import exp


class TulapPSRN(object):
    def __init__(self, shift, epsilon, delta) -> None:
        self.shift = shift
        self.epsilon = epsilon
        self.delta = delta
        self.uniform = UniformPSRN()
        self.precision = 50

    def q_cnd(self, u, c, R):  # CND quantile function for f
        if u < c:
            return self.q_cnd(1 - self.f(u, R), self.f(u, R), c) - 1
        elif c <= u <= 1 - c:  # the linear function
            return (u - 1 / 2) / (1 - 2 * c)
        else:
            return self.q_cnd(self.f(1 - u, R), self.f(u, R), c) + 1

    def f(self, u, _R):
        epsilon, delta = self.epsilon, self.delta
        return max(0, 1 - delta - exp(epsilon) * u, exp(-epsilon) * (1 - delta - u))

    def edge(self, R):
        epsilon, delta = self.epsilon, self.delta
        unif = self.uniform.edge(R)
        c = (1 - delta) / (1 + exp(epsilon))
        if c == 0.5:
            return None

        return self.q_cnd(unif, c, R)

    def refine(self):
        self.precision += 1
        self.uniform.refine()

    def refinements(self):
        return self.precision
