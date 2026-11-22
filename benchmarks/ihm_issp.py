"""OpenDP vs authors' IHM; ISSP Algorithm 1 eligibility.

Needs numpy, scipy, sklearn. No compiled OpenDP runtime.
Run: OPENBLAS_NUM_THREADS=1 python benchmarks/ihm_issp.py --trials 300
The IHM Gaussian sketch uses its EXACT Wishart marginal, avoiding a k*n
matrix. Reproduces authors' released T=3 update, with an optional paper
-eta^2*theta correction as a separate ablation.

Inputs are PUBLIC benchmark fixtures, normalized before the mechanism.
Do not use their data-dependent normalization on confidential datasets.
ISSP is NOT simulated: its Algorithm 1 public gates and the necessary
d/n leverage lower bound are checked instead.
"""
import argparse
import csv
import math

import numpy as np
from scipy.optimize import brentq, minimize_scalar
from scipy.special import ndtr
from sklearn.datasets import load_diabetes, load_linnerud


def gaussian_sd(eps, delta):
    """Analytic Gaussian mechanism of Balle and Wang."""
    def delta_at(s):
        a, b = 1 / (2*s), eps*s
        return ndtr(a-b) - math.exp(eps) * ndtr(-a-b)
    high = 1.
    while delta_at(high) > delta:
        high *= 2
    return brentq(lambda s: delta_at(s)-delta, 1e-5, high)


def renyi_cost(alpha, gamma, k, delta, T):
    if alpha <= 1 or alpha >= gamma:
        return float("inf")
    a = k*alpha*math.log1p(-1/gamma)/(2*(alpha-1))
    b = -k*math.log1p(-alpha/gamma)/(2*(alpha-1))
    c = (math.log(1/delta) + (alpha-1)*math.log1p(-1/alpha)
         - math.log(alpha))/(alpha-1)
    eigen = math.sqrt(2*math.log(1.25/delta))/(gamma/math.sqrt(k))
    return T*(a+b) + c + eigen


def ihm_params(d, eps, delta):
    """Port of public calibration from authors' HessianMix source."""
    T = 3
    varrho = delta/10
    k = max(math.ceil(6*d), math.ceil(6*math.log(4*T/varrho)))
    def privacy(gamma):
        if gamma <= 1.0001:
            return float("inf")
        res = minimize_scalar(
            lambda a: renyi_cost(a,gamma,k,delta/4,T),
            bounds=(1.00001,gamma-1e-5), method="bounded")
        return res.fun
    lo, hi = 1.0001, 2.
    while privacy(hi) > eps/2:
        hi *= 2
        if hi > 1e10:
            raise RuntimeError("IHM calibration did not converge")
    for _ in range(44):
        mid = (lo+hi)/2
        if privacy(mid) <= eps/2:
            hi = mid
        else:
            lo = mid
    return dict(k=k,T=T,gamma=hi,
                tau=math.sqrt(2*math.log(max(4/delta,4/varrho))),
                grad_sd=gaussian_sd(eps/2,delta/4)*math.sqrt(T))


def open_dp(G, cross, eps, delta, rng):
    d = len(cross)
    L = math.log(1/delta)
    rho = (math.sqrt(L+eps)-math.sqrt(L))**2
    sigma = 1/math.sqrt(rho)
    Z = rng.normal(size=(d,d))
    H = G + sigma*(Z+Z.T)/2
    b = cross + sigma*rng.normal(size=d)
    vals, Q = np.linalg.eigh(H)
    alpha = .05
    margin = sigma * min(
        math.sqrt(2*d)+math.sqrt(-2*math.log(alpha)),
        math.sqrt((d+1)*math.log(d/alpha)))
    return Q@((Q.T@b)/np.maximum(vals,margin))


def ihm(X, y, G, p, rng, paper_correction=False):
    d = X.shape[1]
    k,T,gamma,tau = p["k"],p["T"],p["gamma"],p["tau"]
    lambda_min = float(np.linalg.eigvalsh(G)[0])
    private_min = max(0,lambda_min-gamma/math.sqrt(k)*(tau-rng.normal()))
    eta2 = max(0,gamma-private_min)
    vals,Q = np.linalg.eigh(G)
    root = Q @ np.diag(np.sqrt(np.maximum(vals,0)+eta2))
    theta = np.zeros(d)
    for _ in range(T):
        # Exactly the distribution of S X + sqrt(eta2)*xi.
        Z = rng.normal(size=(k,d)) @ root.T
        H = Z.T@Z/k
        b = X.T @ np.clip(y-X@theta,-1,1)
        b += p["grad_sd"]*rng.normal(size=d)
        if paper_correction:
            b -= eta2*theta
        theta += np.linalg.solve(H,b)
    return theta


def issp_gate(n,d,eps,delta):
    k = math.ceil(12*math.log(3/delta)/eps)+8
    L0 = min(1/(96*k),3*eps/(56*math.log(12/delta)))
    # For any full rank weight vector w in [0,1]^n:
    # sum_i w_i (x_i.T (X.T diag(w) X)^-1 x_i) = d.
    # So an eligible full rank fit requires L0 >= d/n.
    return dict(issp_k=k,issp_L0_max=L0,
                issp_n_necessary=math.ceil(d/L0),
                issp_leverage_gate=(d/n <= L0))


def normalize(X,y):
    X=np.asarray(X,dtype=float).copy()
    y=np.asarray(y,dtype=float).copy()
    X-=X.mean(axis=0)
    y-=y.mean()
    X/=max(float(np.linalg.norm(X,axis=1).max()),1e-12)
    y/=max(float(np.abs(y).max()),1e-12)
    return X,y


def datasets():
    for n,d in ((100,5),(500,5),(500,10),(2000,10)):
        for kind in ("isotropic","moderate","collinear"):
            rng=np.random.default_rng(137+n+d)
            V,_=np.linalg.qr(rng.normal(size=(d,d)))
            scales={"isotropic":np.ones(d),
                    "moderate":np.geomspace(1,.3,d),
                    "collinear":np.geomspace(1,.025,d)}[kind]
            X=rng.normal(size=(n,d))@(V*scales).T
            theta=rng.normal(size=d)/math.sqrt(d)
            y=X@theta+.3*rng.normal(size=n)
            yield f"{kind}_n{n}_d{d}",*normalize(X,y)
    ds=load_diabetes()
    yield "diabetes",*normalize(ds.data,ds.target)
    ds=load_linnerud()
    yield "linnerud",*normalize(ds.data,ds.target[:,0])


def benchmark(trials, delta_policy, seed=51):
    cache={}
    for index,(name,X,y) in enumerate(datasets()):
        n,d=X.shape
        G,cross=X.T@X,X.T@y
        ref=np.linalg.lstsq(X,y,rcond=None)[0]
        delta=1e-6 if delta_policy=="fixed" else 1/n**2
        for eps in (1,4,16):
            key=(d,eps,delta)
            if key not in cache:
                cache[key]=ihm_params(d,eps,delta)
            p=cache[key]
            results={"opendp":[],"ihm_authors":[],"ihm_paper":[]}
            for rep in range(trials):
                for j,method in enumerate(results):
                    rng=np.random.default_rng(
                        np.random.SeedSequence([seed,index,eps,rep,j]))
                    if method=="opendp":
                        theta=open_dp(G,cross,eps,delta,rng)
                    else:
                        theta=ihm(X,y,G,p,rng,method=="ihm_paper")
                    error=theta-ref
                    results[method].append(float(error@G@error/n))
            row=dict(dataset=name,n=n,d=d,epsilon=eps,delta=delta,
                     trials=trials,ihm_k=p["k"],ihm_gamma=p["gamma"],
                     **{f"loss_{name}":float(np.mean(values))
                        for name,values in results.items()},
                     **issp_gate(n,d,eps,delta))
            yield row


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--trials",type=int,default=300)
    parser.add_argument("--delta-policy",choices=("fixed","n-2"),
                        default="fixed")
    parser.add_argument("--output",default="ihm_issp_results.csv")
    args=parser.parse_args()
    result=list(benchmark(args.trials,args.delta_policy))
    with open(args.output,"w",newline="") as output:
        writer=csv.DictWriter(output,fieldnames=list(result[0]))
        writer.writeheader()
        writer.writerows(result)
    for eps in (1,4,16):
        selected=[r for r in result if r["epsilon"]==eps]
        ratios=np.array([r["loss_ihm_authors"]/r["loss_opendp"]
                         for r in selected])
        print(eps,"median IHM/OpenDP",round(float(np.median(ratios)),3),
              "IHM wins",int(sum(ratios<1)),"/",len(ratios),
              "ISSP necessary gate",sum(r["issp_leverage_gate"]
                                         for r in selected),"/",len(selected))


if __name__=="__main__":
    main()
