import opendp.prelude as dp

import math
import numpy as np
from scipy.stats import binom
import scipy

# what needs to be done?

# arguments are in terms of b and q, but user doesn't know what those are
# examples
# code cleanup
# documentation
# another pass on naming
# tests


def _ptulap(t, m=0, b=0, q=0):
    lcut = q / 2
    rcut = q / 2
    t = t - m  # normalize
    # split the positive and negative t calculations, and factor out stuff
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


def make_ump_test(theta, size, alpha, epsilon, delta, tail):
    def function(values):
        b = math.exp(-epsilon)
        q = 2 * delta * b / (1 - b + 2 * delta * b)
        values = list(range(0, size + 1))
        B = binom.pmf(k=values, n=size, p=theta)
        """_summary_
        Args:
            theta (_type_): true probability of binomial distribution
            size (_type_): sample size
            alpha (_type_): significance level alpha
            epsilon (_type_), delta (_type_): DP parameters

        Returns:
            _type_: _description_
        """

        def obj(s):
            values = np.array(values)
            phi = _ptulap(t=values - s, m=0, b=b, q=q)
            return np.dot(B, phi) - alpha

        lower = -1
        upper = 1

        while obj(lower) < 0:
            lower *= 2
        while obj(upper) > 0:
            upper *= 2
        root = scipy.optimize.brentq(
            obj, lower, upper
        )  # scipy.optimize.brentq(function, min, max)
        s = root
        values = np.array(values)
        phi = _ptulap(t=values - s, m=0, b=b, q=q)

        if tail == "left":
            return phi
        elif tail == "right":
            return 1 - phi

    return dp.new_function(function, TO=dp.Vec[float])


def make_oneside_pvalue(theta, size, b, q, tail):
    """Right tailed p-value

    :param theta: true probability of binomial distribution
    :param size: number of trials
    :param b
    :param q: tulap parameters
    """

    def function(Z):
        """
        :param Z: tulap random variables
        """
        reps = Z.size  # sample size
        if reps > 1:
            pval = [0] * reps
            values = np.array(range(size))

            B = binom.pmf(k=values, n=size, p=theta)

            for r in range(reps):
                if tail == "right":
                    F = _ptulap(t=values - Z[r], m=0, b=b, q=q)
                elif tail == "left":
                    F = 1 - _ptulap(t=values - Z[r], m=0, b=b, q=q)
                pval[r] = np.dot(F.T, B)
            return pval

        else:
            pval = [0]
            values = np.array(range(size))
            B = binom.pmf(k=values, n=size, p=theta)
            if tail == "right":
                F = _ptulap(t=values - Z, m=0, b=b, q=q)
            elif tail == "left":
                F = 1 - _ptulap(t=values - Z, m=0, b=b, q=q)
            pval[0] = np.dot(F.T, B)
            return pval[0]

    return dp.new_function(function, TO=dp.Vec[float])


def make_twoside_pvalue(theta, size, b, q):
    def function(Z):
        T = abs(Z - size * theta)
        pval = np.subtract(
            make_oneside_pvalue(
                Z=T + size * theta, size=size, theta=theta, b=b, q=q, tail="right"
            ),
            make_oneside_pvalue(
                Z=size * theta - T, size=size, theta=theta, b=b, q=q, tail="right"
            ),
        )

        return pval + 1

    return dp.new_function(function, TO=dp.Vec[float])


def make_CI(alpha, size, b, q, tail):
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

    def function(Z):
        if tail == "lower":
            CIobj = lambda x: (
                (make_oneside_pvalue(Z=Z, size=size, theta=x, b=b, q=q, tail="right"))
                - alpha
            )
        elif tail == "upper":
            CIobj = lambda x: (
                (make_oneside_pvalue(Z=Z, size=size, theta=x, b=b, q=q, tail="right"))
                - (1 - alpha)
            )
        L = minimize_scalar(
            fun=CIobj, method=custmin, bracket=(0, 1)
        )  # args already set in CIobj
        return L.x

    return dp.new_function(function, TO=dp.Vec[float])


def make_CI_twoside(alpha, size, b, q):
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

    def function(Z):
        mle = Z / size
        mle = max(min(mle, 1), 0)
        CIobj2 = lambda x: (
            make_twoside_pvalue(Z=Z, theta=x, size=size, b=b, q=q) - alpha
        )

        if mle > 0:
            L = minimize_scalar(
                fun=CIobj2, method=custmin, bracket=(0, mle)
            )  # , args=mle/2
            L = L.x
        else:
            L = 0

        if mle < 1:
            U = minimize_scalar(
                fun=CIobj2, method=custmin, bracket=(mle, 1)
            )  # , args=((1-mle)/2)
            U = U.x
        else:
            U = 1

        CI = [L, U]
        return CI

    return dp.new_function(function, TO=dp.Vec[float])
