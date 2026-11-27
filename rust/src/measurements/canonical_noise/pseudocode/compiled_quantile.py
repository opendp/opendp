# type: ignore
def round_grid(value, grid, upper):
    scaled = value * grid
    return RBig(ceil(scaled) if upper else floor(scaled)) / grid


def quantile(tradeoff, u, upper, refinements):
    c = tradeoff.fixed_point
    width = 1 - 2 * c
    if u >= c:
        return (u - RBig(1, 2)) / width
    bits = u.denominator.bit_length() + width.denominator.bit_length()
    grid = RBig(2 ** (2 * bits + 64 + refinements))
    offset = RBig(0)
    while u < c:
        s, end = tradeoff.active_segment_and_end(u)
        if s.a == 0:
            steps = ceil((end - u) / s.delta)
            u += steps * s.delta
            offset -= steps
            continue

        def double(v):
            return v * (2 + s.a * v)

        limit = (end - u) / s.eval(u)
        powers = []
        lo = hi = RBig(1)
        steps = 1
        while hi < limit:
            powers.append((steps, lo, hi))
            lo = round_grid(double(lo), grid, False)
            hi = round_grid(double(hi), grid, True)
            steps *= 2
        for steps, lo, hi in reversed(powers):
            d = s.eval(u)
            if u + d * hi >= end:
                continue
            next_u = min(end, round_grid(u + d * (hi if upper else lo), grid, upper))
            if next_u > u:
                u = next_u
                offset -= steps
        if u < end:
            u += s.eval(u)
            offset -= 1
    return offset + (u - RBig(1, 2)) / width


def inverse_cdf(self, uniform, refinements, direction):
    if uniform == 0 or uniform == 1:
        return None
    reflect = uniform > RBig(1, 2)
    u = 1 - uniform if reflect else uniform
    value = quantile(self.tradeoff, u, (direction == Up) != reflect, refinements)
    return (-value if reflect else value) * self.scale + self.shift
