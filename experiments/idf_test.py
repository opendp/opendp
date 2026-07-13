import sys, dataclasses
import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score
sys.path.insert(0,"/private/tmp/claude-503/-Users-shoebox-OpenDP-openDP/604e3fb5-3b64-4659-a08c-e537c1f57452/scratchpad")
from improve import prep, clip_first_L, rand_centers, topm_norm

def run_idf(x, xn_raw, y, k, rho, use_idf, L=128, m=96, T=5, seed=0):
    rng=np.random.default_rng(seed); d=x.shape[1]
    xc=clip_first_L(x,L)
    n_rel = T + (1 if use_idf else 0)
    sigma=np.sqrt(L*n_rel/(2.0*rho))   # split budget over all releases incl. df
    if use_idf:
        df = np.asarray(xc.sum(0)).ravel() + rng.normal(0,sigma,d)   # DP df release
        df = np.maximum(df, 0.0)
        idf = np.log((x.shape[0]+1)/(df+1))+1.0
        xw = x.multiply(idf).tocsr()
        nrm=np.sqrt(np.asarray(xw.multiply(xw).sum(1)).ravel()); nrm[nrm==0]=1
        xn = xw.multiply(1.0/nrm[:,None]).tocsr()
    else:
        xn = xn_raw
    C=rand_centers(k,d,m,rng)
    for _ in range(T):
        lab=xn.dot(C.T.tocsr()).toarray().argmax(1)
        S=np.zeros((k,d))
        for j in range(k):
            idx=np.flatnonzero(lab==j)
            if idx.size: S[j]=np.asarray(xc[idx].sum(0)).ravel()
        S=S+rng.normal(0,sigma,S.shape)
        C=topm_norm(S,m)
    lab=xn.dot(C.T.tocsr()).toarray().argmax(1)
    return adjusted_rand_score(y,lab)

for pname,(chp,sh,k) in {"default":(1.0,0.15,16),"overlap":(1.0,0.5,16),"hard":(0.4,0.5,40)}.items():
    x,xn,y=prep(chp,sh,k)
    for rho in (0.1,0.5):
        base=np.mean([run_idf(x,xn,y,k,rho,False,seed=s) for s in range(3)])
        idf =np.mean([run_idf(x,xn,y,k,rho,True ,seed=s) for s in range(3)])
        print(f"{pname:8s} rho={rho}: no-idf={base:.3f}  +DP-idf={idf:.3f}",flush=True)
