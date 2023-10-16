import opendp
from opendp.transformations import make_cast_default, make_clamp, make_bounded_sum
from opendp.measurements import make_base_discrete_laplace
from opendp.combinators import *
from opendp.mod import enable_features
from opendp.typing import *

import math
import numpy as np
from scipy.stats import binom
from Tulap import ptulap
import scipy

from opendp.core import new_function
from opendp.measurements import make_tulap


# def make_postprocess_frac():
#     """An example user-defined postprocessor from Python"""
#     def function(arg):
#         return arg[0] / arg[1]

#     return dp.new_function(function, float)

# def test_new_function():
#     mech = make_postprocess_frac()
#     print(mech([12., 100.]))

## TODO: update ptulap

def make_ump_test(VectorDomain[AllDomain[float,str]]):
    
    def function(theta, size, alpha, epsilon, delta, tail):
        b = math.exp(-epsilon)
        q = 2 * delta * b / (1 - b + 2 * delta * b)
        values = list(range(0, size+1))
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
            phi = ptulap(t=values-s, m=0, b=b, q=q)
            return np.dot(B, phi) - alpha


        lower = -1
        upper = 1

        while obj(lower) < 0:
            lower *= 2
        while obj(upper) > 0:
            upper *= 2
        root = scipy.optimize.brentq(obj, lower, upper)  # scipy.optimize.brentq(function, min, max)
        s = root
        values = np.array(values)
        phi = ptulap(t=values-s, m=0, b=b, q=q)
        
        if tail == 'left':
            return phi
        elif tail == 'right':
            return 1 - phi

    return new_function(
        function,
        VectorDomain[AllDomain[float]]
    )


def make_oneside_pvalue(VectorDomain[AllDomain[float, str]]):
        
    def function(Z, theta, size, b, q, tail):
        """_summary_
        Right tailed p-value
        Args:
            Z (_type_): tulap random variables
            theta (_type_): true probability of binomial distribution
            size (_type_): number of trials
            b (_type_), q (_type_): tulap parameters
            
        Returns:

        """
        reps = Z.size  # sample size
        if reps > 1:
            pval = [0] * reps
            values = np.array(range(size))

            B = binom.pmf(k=values, n=size, p=theta)

            for r in range(reps):
                if tail == 'right':
                    F = ptulap(t=values-Z[r], m=0, b=b, q=q)
                elif tail == 'left':
                    F = 1 - ptulap(t=values-Z[r], m=0, b=b, q=q)
                pval[r] = np.dot(F.T, B)
            return pval
        
        else:
            pval = [0]
            values = np.array(range(size))
            B = binom.pmf(k=values, n=size, p=theta)
            if tail == 'right':
                F = ptulap(t=values-Z, m=0, b=b, q=q)
            elif tail == 'left':
                F = 1 - ptulap(t=values-Z, m=0, b=b, q=q)
            pval[0] = np.dot(F.T, B)
            return pval[0]
    
    return new_function(
        function, 
        VectorDomain[AllDomain[float]]
    )


def make_twoside_pvalue(VectorDomain[AllDomain[float]]):
    
    def function(Z, theta, size, b, q):
        T = abs(Z - size * theta)
        pval = np.subtract(make_oneside_pvalue(Z=T+size*theta, size=size, theta=theta, b=b, q=q, tail='right'), 
                           make_oneside_pvalue(Z=size*theta-T, size=size, theta=theta, b=b, q=q, tail='right'))

        return pval+1
    
    return new_function(
        function, 
        VectorDomain[AllDomain[float]]
    )
            


def make_CI(VectorDomain[AllDomain[float, str]]):
    from scipy.optimize import OptimizeResult, minimize_scalar

    def custmin(fun, bracket, args=(), 
                maxfev=None, stepsize=1e-3, maxiter=500, callback=None, **options):
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
                return OptimizeResult(fun=besty, x=bestx, nit=niter,
                            nfev=funcalls, success=(niter > 1))
            elif abs(fun(mid, *args)) <= min_diff:
                # print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(fun=besty, x=bestx, nit=niter,
                            nfev=funcalls, success=(niter > 1))
            elif fun(mid, *args) > 0:      # mid > alpha
                # print("diff > 0")
                upper = mid-stepsize
            elif fun(mid, *args) < 0:      # mid < alpha
                # print("diff < 0")
                lower = mid+stepsize
                
        bestx = mid
        besty = fun(mid, *args)
        # print("while loop break")
        # print("low and up: ", lower, upper)
        return OptimizeResult(fun=besty, x=bestx, nit=niter,
                        nfev=funcalls, success=(niter > 1)) 
          
    def function(alpha, Z, size, b, q, tail):
        if tail == 'lower':
            CIobj = lambda x: ((make_oneside_pvalue(Z=Z, size=size, theta=x, b=b, q=q, tail='right')) - alpha)
        elif tail == 'upper':
            CIobj = lambda x: ((make_oneside_pvalue(Z=Z, size=size, theta=x, b=b, q=q, tail='right')) - (1-alpha))
        L = minimize_scalar(fun=CIobj, method=custmin, bracket=(0, 1))    # args already set in CIobj
        return L.x
    
    return new_function(
        function, 
        VectorDomain[AllDomain[float]]
    )

def make_CI_twoside(VectorDomain[AllDomain[float]]):
    from scipy.optimize import OptimizeResult, minimize_scalar
    def custmin(fun, bracket, args=(), 
                maxfev=None, stepsize=1e-3, maxiter=500, callback=None, **options):
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
                return OptimizeResult(fun=besty, x=bestx, nit=niter,
                            nfev=funcalls, success=(niter > 1))
            elif abs(fun(mid, *args)) <= min_diff:
                # print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(fun=besty, x=bestx, nit=niter,
                            nfev=funcalls, success=(niter > 1))
            elif fun(mid, *args) > 0:      # mid > alpha
                # print("diff > 0")
                upper = mid-stepsize
            elif fun(mid, *args) < 0:      # mid < alpha
                # print("diff < 0")
                lower = mid+stepsize
                
        bestx = mid
        besty = fun(mid, *args)
        # print("while loop break")
        # print("low and up: ", lower, upper)
        return OptimizeResult(fun=besty, x=bestx, nit=niter,
                        nfev=funcalls, success=(niter > 1))
    def function(alpha, Z, size, b, q):
        mle = Z/size
        mle = max(min(mle, 1), 0)
        CIobj2 = lambda x: (make_twoside_pvalue(Z=Z, theta=x, size=size, b=b, q=q) - alpha)

        if mle > 0:
            L = minimize_scalar(fun=CIobj2, method=custmin, bracket=(0, mle)) #, args=mle/2
            L = L.x
        else:
            L = 0
        
        if mle < 1:
            U = minimize_scalar(fun=CIobj2, method=custmin, bracket=(mle, 1)) #, args=((1-mle)/2)
            U = U.x
        else:
            U = 1
        
        CI = [L, U]
        return CI
    
    return new_function(
        function, 
        VectorDomain[AllDomain[float]]
    )

