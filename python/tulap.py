import math

def f(alpha, delta, epsilon):
    t1 = 1.0 - delta - math.exp(epsilon) * alpha
    t2 = math.exp(-epsilon) * (1.0 - delta - alpha)
    return max(t1, t2, 0.0)

def q_cnd(u, c, delta, epsilon):
    if u < c: # this is evaluated
        return q_cnd(1.0 - f(u, delta, epsilon), c, delta, epsilon) - 1.0
    elif c <= u <= 1.0 - c:
        return (u - 0.5) / (1.0 - 2.0 * c)
    else:
        return q_cnd(f(1.0 - u, delta, epsilon), c, delta, epsilon) + 1.0

def inverse_tulap(unif, delta, epsilon): # epsilon = 0 and delta = 0 
    unif = 0
    c = (1.0 - delta) / (1.0 + math.exp(epsilon)) # c = 0
    return q_cnd(unif, c, delta, epsilon) 
