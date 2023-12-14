import numpy as np
import math
import numpy.random as random
import scipy
from scipy.stats import norm, geom, uniform

'''
m - real number
b - (0, 1)
q - [0, 1)

Note:
Tulap random variable Tulap(m, b, q) is continuous and symmetric of m
'''

random.seed(1000)

def ptulap(t, m=0, b=0, q=0):
    lcut = q/2
    rcut = q/2
    t = t-m     # normalize
    # split the positive and negative t calculations, and factor out stuff
    r = np.rint(t)
    g = -math.log(b)
    l = math.log(1+b)
    k = 1-b

    negs = np.exp((r*g) - l + np.log(b + ((t - r + (1/2))*k)))
    poss = 1 - np.exp((r*(-g)) - l + np.log(b + ((r - t + (1/2)) * k)))

    # check for infinities
    negs[np.isinf(negs)] = 0
    poss[np.isinf(poss)] = 0
    # truncate w.r.t. the indicator on t's positivity
    is_leq0 = np.less_equal(t, 0).astype(int)
    trunc = (is_leq0 * negs) + ((1-is_leq0) * poss)

    # handle the cut adjustment and scaling
    q = lcut + rcut
    is_mid = np.logical_and(np.less_equal(lcut, trunc), np.less_equal(trunc, (1-rcut))).astype(int)
    is_rhs = np.less((1-rcut), trunc).astype(int)
    return (((trunc-lcut) / (1-q)) * is_mid+is_rhs)



def approx_trials(n, prob=1, alpha=0):
    # solve a quadratic form for this
    a = prob ** 2
    b = -((2 * n * prob) + ((norm.ppf(q=alpha)**2) * prob * (1-prob)))
    c = n ** 2
    n_trials = ((-b + math.sqrt(b**2 - (4*a*c))) / (2*a))
    return int(round(n_trials))

# generate random samples from Tulap distribution using rejection sampling
def rTulap(n, m=0, b=0, q=0):
    # q represents truncation
    if q >= 0:
        alpha =  0.95
        lcut = q/2
        rcut = q/2
    
        # calculate actual amount needed
        q = lcut + rcut
        n2 = approx_trials(n=n, prob=(1-q), alpha=alpha)

        # sample from the original Tulambda distribution
        geos1 = geom.rvs(size=n2, p=(1-b))
        geos2 = geom.rvs(size=n2, p=(1-b))
        unifs = uniform.rvs(loc=-1/2, scale=1, size=n2)    # range = [loc, loc+scale]
        samples = m + geos1 - geos2 + unifs     # numpy ndarray


        # cut the tails based on the untampered CDF (i.e. no cuts)
        probs = ptulap(samples, m=m, b=b)
        is_mid_bool = np.logical_and(np.less_equal(lcut, probs), np.less_equal(probs, (1-rcut))).astype(int)
        is_mid = []
        for i in range(len(is_mid_bool)):
            if is_mid_bool[i] == 1:
                is_mid.append(i)

        # abuse the NA property of R wrt arithmetics
        mids = samples[is_mid]
        length = len(mids)
        while length < n:
            diff = n - length
            mids = np.concatenate((mids, rTulap(n=diff, m=m, b=b, q=q)), axis=None)
            length = len(mids)
        return mids[:n]
    
    geos1 = geom.rvs(size=n2, p=(1-b))
    geos2 = geom.rvs(size=n2, p=(1-b))
    unifs = uniform.rvs(loc=-1/2, scale=1, size=n2)
    samples = m + geos1 - geos2 + unifs
    print("q < 0")
    return samples


# generate random samples from Tulap distribution using inverse transform sampling
def rTulap_inv(epsilon, delta):
    def qCND(u, f, c):         # CND quantile function for f
        if u < c:
            return qCND(1 - f(u), f, c) - 1
        elif c <= u <= 1-c: # the linear function 
            return (u - 1/2)/(1 - 2*c)
        else:
            return qCND(f(1-u),f ,c) + 1  
    unif = np.random.uniform(0, 1)
    c = (1-delta) / (1 + math.exp(epsilon))
    f = max(0, 1 - delta - math.exp(epsilon) * unif, math.exp(-epsilon) * (1 - delta - unif))
    samples = qCND(unif, f, c)
    return samples
