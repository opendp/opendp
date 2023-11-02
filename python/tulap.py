import math
import random
import unittest
def f(alpha, delta, epsilon):
    t1 = 1.0 - delta - math.exp(epsilon) * alpha
    t2 = math.exp(-epsilon) * (1.0 - delta - alpha)
    return max(t1, t2, 0.0)
# the value of c shuldn't be 0.5. it should always be less than 0.5
def q_cnd(u, c, delta, epsilon):
    if u < c: # this is evaluated
        print("u<c")
        print (u, c)
        return q_cnd(1.0 - f(u, delta, epsilon), c, delta, epsilon) - 1.0
    elif c <= u <= 1.0 - c:
        print("c <= u <= 1.0 - c")
        print (u, c)
        return (u - 0.5) / (1.0 - 2.0 * c)
    else:
        print("else")
        print (u, c)
        return q_cnd(f(1.0 - u, delta, epsilon), c, delta, epsilon) + 1.0

def inverse_tulap(delta, epsilon): # epsilon = 0.1 and delta = 0.001
    unif = random.random() # should this be generated randomly?
    #print(unif)
    c = (1.0 - delta) / (1.0 + math.exp(epsilon)) # c = 0
    #c = 0.5
    #print(c)
    #print (q_cnd(unif, c, delta, epsilon))
    return q_cnd(unif, c, delta, epsilon) 

inverse_tulap(0.001,0.1)