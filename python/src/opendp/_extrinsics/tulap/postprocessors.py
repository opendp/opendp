import opendp.prelude as dp
import math
import numpy as np
from scipy.stats import binom
import scipy
dp.enable_features("contrib")

def _ptulap(t, m=0, epsilon=0, delta=0):
    t = np.atleast_1d(t)
    b = math.exp(-epsilon)  
    q = (2 * delta * b) / (1 - b + 2 * delta * b)  
    lcut = q / 2
    rcut = q / 2
    t = t - m  # normalize
    r = np.rint(t)
    g = -math.log(b)
    l = math.log(1 + b)
    k = 1 - b

    negs = np.exp((r * g) - l + np.log(b + ((t - r + (1 / 2)) * k)))
    poss = 1 - np.exp((r * (-g)) - l + np.log(b + ((r - t + (1 / 2)) * k)))

    # check for infinities
    negs[np.isinf(negs)] = 0
    poss[np.isinf(poss)] = 0
    # truncate w.r.t. the indicator on t's positivity
    is_leq0 = np.less_equal(t, 0).astype(int)
    trunc = (is_leq0 * negs) + ((1 - is_leq0) * poss)

    # handle the cut adjustment and scaling
    q = lcut + rcut
    is_mid = np.logical_and(
        np.less_equal(lcut, trunc), np.less_equal(trunc, (1 - rcut))
    ).astype(int)
    is_rhs = np.less((1 - rcut), trunc).astype(int)
    return ((trunc - lcut) / (1 - q)) * is_mid + is_rhs

# define a public API. How about this one?
class Tulap(object):
    def __init__(self, data, epsilon, delta, theta, size) -> None:
        self.data = data
        self.epsilon = epsilon
        self.delta = delta
        self.theta = theta
        self.size = size


    def ump_test(self, alpha, tail):
        postprocessor = _make_ump_test(self.theta, self.size, alpha, self.epsilon, self.delta, tail)
        return postprocessor(self.data)

    def CI(self, alpha, tail=None):
        if tail is None:
            postprocessor = _make_CI_twoside(alpha, self.size, self.epsilon, self.delta)
        elif tail in {"lower", "upper"}:
            postprocessor = _make_CI_oneside(alpha, self.size, self.epsilon, self.delta, tail)
        else:
            raise ValueError("tail must be None, left or right")

        return postprocessor(self.data)

    def p_value(self, tail=None):
        if tail is None:
            postprocessor = _make_twoside_pvalue(self.theta, self.size, self.epsilon, self.delta)
        elif tail in {"left", "right"}:
            postprocessor = _make_oneside_pvalue(self.theta, self.size, self.epsilon, self.delta, tail)
        else:
            raise ValueError("tail must be None, left or right")

        return postprocessor(self.data)


def make_binomial_tulap(input_domain, input_metric, epsilon, delta) -> dp.Measurement:
    from opendp.measurements import make_tulap
    return make_tulap(input_domain, input_metric, epsilon, delta) >> (lambda data: Tulap(data, epsilon, delta))


def _make_ump_test(theta, size, alpha, epsilon, delta, tail):
    def function(data):
        b = math.exp(-epsilon)
        q = 2 * delta * b / (1 - b + 2 * delta * b)
        values = list(range(0, size + 1))
        B = binom.pmf(k=values, n=size, p=theta)

        def obj(s):
            values_array = np.array(values)
            phi = _ptulap(t=values_array - s, m=0, epsilon=epsilon, delta=delta)
            return np.dot(B, phi) - alpha

        lower = -1
        upper = 1

        while obj(lower) < 0:
            lower *= 2
        while obj(upper) > 0:
            upper *= 2
        root = scipy.optimize.brentq(obj, lower, upper)
        s = root
        values_array = np.array(values)
        phi = _ptulap(t=values_array - s, m=0, epsilon=epsilon, delta=delta)

        if data and tail == "left":
            return phi
        elif data and tail == "right":
            return 1 - phi
    return function

# should take in Z
def _make_oneside_pvalue(theta, size, epsilon, delta, tail):
    """Right tailed p-value

    :param theta: true probability of binomial distribution
    :param size: number of trials
    :param b
    :param q: tulap parameters
    """
    b = math.exp(-epsilon)  
    q = (2 * delta * b) / (1 - b + 2 * delta * b)  
    def function(Z):
        """
        :param Z: tulap random variables
        """
        Z = np.array(Z) 
        reps = Z.size  # sample size
        if reps > 1:
            pval = [0] * reps
            values = np.array(range(size + 1))  
            B = binom.pmf(k=values, n=size, p=theta)

            for r in range(reps):
                if tail == "right":
                    F = _ptulap(t=values - Z[r], m=0, epsilon=epsilon, delta=delta)
                elif tail == "left":
                    F = 1 - _ptulap(t=values - Z[r], m=0, epsilon=epsilon, delta=delta)
                pval[r] = np.dot(F.T, B)
            return pval

        else:
            pval = [0]
            values = np.array(range(size))
            B = binom.pmf(k=values, n=size, p=theta)
            if tail == "right":
                F = _ptulap(t=values - Z, m=0, epsilon=epsilon, delta=delta)
            elif tail == "left":
                F = 1 - _ptulap(t=values - Z, m=0, epsilon=epsilon, delta=delta)
            pval[0] = np.dot(F.T, B)
            return pval[0]

    return function


def _make_twoside_pvalue(theta, size, epsilon, delta):
    b = math.exp(-epsilon)  
    q = (2 * delta * b) / (1 - b + 2 * delta * b)  
    def function(Z):
        Z = np.array(Z) if not isinstance(Z, np.ndarray) else Z

        T = abs(Z - size * theta)
        pval_right = _make_oneside_pvalue(theta, size,epsilon, delta, "right")(T + size * theta)
        pval_left = _make_oneside_pvalue(theta, size, epsilon, delta, "right")(size * theta - T)

        pval = np.subtract(pval_right, pval_left) + 1

        return pval  # Ensure this is a vector if Z is a vector

    return function


def _make_CI_oneside(alpha, size, epsilon, delta, tail):
    b = math.exp(-epsilon)  
    q = (2 * delta * b) / (1 - b + 2 * delta * b)  
    from scipy.optimize import OptimizeResult, minimize_scalar  # type: ignore[import]

    def custmin(
        fun,
        bracket,
        args=(),
        maxfev=None,
        stepsize=1e-3,
        maxiter=500,
        callback=None,
        **options
    ):
        print("binary search, stepsize = ", 1e-3)
        lower = bracket[0]
        upper = bracket[1]

        funcalls = 1
        niter = 0

        mid = (lower + upper) / 2.0
        bestx = mid
        besty = fun(mid, *args)
        min_diff = 1e-6

        while lower <= upper:
            mid = (lower + upper) / 2.00
            # print("low: ", lower, "up: ", upper)
            # print("mid: ", mid)
            # print("diff: ", fun(mid, *args))
            funcalls += 1
            niter += 1
            if fun(mid, *args) == 0:
                # print("diff = 0")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif abs(fun(mid, *args)) <= min_diff:
                # print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif fun(mid, *args) > 0:  # mid > alpha
                # print("diff > 0")
                upper = mid - stepsize
            elif fun(mid, *args) < 0:  # mid < alpha
                # print("diff < 0")
                lower = mid + stepsize

        bestx = mid
        besty = fun(mid, *args)
        # print("while loop break")
        # print("low and up: ", lower, upper)
        return OptimizeResult(
            fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
        )

    def function(Z): # should not take a vector
        Z = np.array(Z) if not isinstance(Z, np.ndarray) else Z
        if tail == "lower":
            CIobj = lambda x: (_make_oneside_pvalue(x, size=size, epsilon=epsilon, delta=delta, tail="right")(Z)[0]) - alpha

        elif tail == "upper":
            CIobj = lambda x: (
                (_make_oneside_pvalue(x, size=size, epsilon=epsilon, delta=delta, tail="right")(Z))
                - (1 - alpha)
            )
        L = minimize_scalar(
            fun=CIobj, method=custmin, bracket=(0, 1)
        )  
        return L.x

    return function
# the postprocessor should take a vector and return the funtion without the vector


## this doesn't work
def _make_CI_twoside(alpha, size, epsilon, delta):
    b = math.exp(-epsilon)  
    q = (2 * delta * b) / (1 - b + 2 * delta * b)  
    from scipy.optimize import OptimizeResult, minimize_scalar

    def custmin(
        fun,
        bracket,
        args=(),
        maxfev=None,
        stepsize=1e-3,
        maxiter=500,
        callback=None,
        **options
    ):
        print("binary search, stepsize = ", 1e-3)
        lower = bracket[0]
        upper = bracket[1]

        funcalls = 1
        niter = 0

        mid = (lower + upper) / 2.0
        bestx = mid
        besty = fun(mid, *args)
        min_diff = 1e-6

        while lower <= upper:
            mid = (lower + upper) / 2.00
            print("low: ", lower, "up: ", upper)
            print("mid: ", mid)
            print("diff: ", fun(mid, *args))
            funcalls += 1
            niter += 1
            if fun(mid, *args) == 0:
                print("diff = 0")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif abs(fun(mid, *args)) <= min_diff:
                print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif fun(mid, *args) > 0:  # mid > alpha
                print("diff > 0")
                upper = mid - stepsize
            elif fun(mid, *args) < 0:  # mid < alpha
                print("diff < 0")
                lower = mid + stepsize

        bestx = mid
        besty = fun(mid, *args)
        print("while loop break")
        print("low and up: ", lower, upper)
        return OptimizeResult(
            fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
        )

    def function(Z):
        print ("step 1: inside function z")
        Z = np.array(Z) if not isinstance(Z, np.ndarray) else Z
        mle = Z / size
        mle = max(min(mle, 1), 0)
        print ("step 2: calc z")
        #twoside_pvalue_func = make_twoside_pvalue(theta=mle, size=size, epsilon=epsilon, delta=delta)
        print ("step 3: calc twoside_pvalue_func")
        #CIobj2 = lambda x: (twoside_pvalue_func(np.array([Z]))[0] - alpha)
        CIobj2 = lambda x: (
            _make_twoside_pvalue(theta=mle, size=size, epsilon=epsilon, delta=delta)(np.array([x]))[0]
            - alpha
        )

        if mle > 0:
            L = minimize_scalar(fun=CIobj2, method=custmin, bracket=(0, mle))
            L = L.x
        else:
            L = 0

        if mle < 1:
            U = minimize_scalar(fun=CIobj2, method=custmin, bracket=(mle, 1))
            U = U.x
        else:
            U = 1

        CI = [L, U]
        return CI
    return function
