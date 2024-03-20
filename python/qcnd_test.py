import unittest
import math
import random
import functools
import signal

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

def inverse_tulap(unif, delta, epsilon): # epsilon = 0.1 and delta = 0.001
    unif = random.random() # should this be generated randomly?
    c = (1.0 - delta) / (1.0 + math.exp(epsilon)) # c = 0
    return q_cnd(unif, c, delta, epsilon) 

# Timeout decorator
def timeout(seconds=10):
    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            signal.signal(signal.SIGALRM, handler)
            signal.alarm(seconds)
            try:
                result = func(*args, **kwargs)
            finally:
                signal.alarm(0)
            return result
        def handler(signum, frame):
            raise TimeoutError()
        return wrapper
    return decorator

class TestQ_CND(unittest.TestCase):
    
    @timeout(5)  # Set a timeout of 5 seconds for the test
    def test_no_infinite_recursion(self):
        epsilon = 0.1
        delta = 0.001
        c = (1.0 - delta) / (1.0 + math.exp(epsilon))
        
        # Test for a range of unif values
        for unif in [i * 0.01 for i in range(101)]:
            q_cnd(unif, c, delta, epsilon)

if __name__ == "__main__":
    unittest.main()
