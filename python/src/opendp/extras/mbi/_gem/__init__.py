from typing import Any
from dataclasses import dataclass, field

import jax
import jax.nn
import jax.numpy as jnp
import numpy as np
import optax
from mbi import Dataset, Domain, Factor, CliqueVector, LinearMeasurement
from mbi.clique_utils import maximal_subset
from mbi.estimation import minimum_variance_unbiased_total
from collections.abc import Sequence
from itertools import combinations
from math import prod, sqrt

@jax.tree_util.register_dataclass



@dataclass
class GemPlusModel():

    _params : Any #network weights
    _z : jax.Array #the inital seeds for person types
    domain : Domain = field(metadata={"static": True}) #dataset metadata
    total : float = field(metadata={"static": True}) #N
    potentials = None
    #_opt_state : Any = None #optimizer state

    def num_components(self) -> int:
        """Number of soft records B — the rows of the fixed noise batch."""
        return self._z.shape[0] #number of distinct types

    @staticmethod
    def random(
        domain: Domain,
        total: float,
        dim: int = 512,
        batch: int = 25000,
        hidden: tuple[int, ...] | None = None,
        seed: int = 0,
        scale: float = 0.1,
    ) -> "GemPlusModel":
        """Initialize with a fixed gaussian batch and random network weights."""
        if hidden is None:
            hidden = (dim * 2, dim * 2)

        key_z, key_w = jax.random.split(jax.random.PRNGKey(seed))
        z = jax.random.normal(key_z, (batch, dim))

        widths = [dim, *hidden, sum(domain.shape)]
        keys = jax.random.split(key_w, len(widths) - 1)
        params = [
        {
            "W": jax.random.normal(k, (n_in, n_out)) * scale,
            "b": jnp.zeros(n_out),
        }
        for k, n_in, n_out in zip(keys, widths[:-1], widths[1:])
        ]
        return GemPlusModel(params, z, domain, total)

    def _apply(self) -> jax.Array:
        h = self._z
        num_layers = len(self._params) #number of layers
        for i in range(num_layers): 
            W_i, b_i = self._params[i]["W"], self._params[i]["b"]
            h = h@W_i + b_i
            if i != num_layers-1 : #every layer except the last
                h = jax.nn.relu(h)

        return h

    def products(self) -> dict[str, jax.Array]:

        h = self._apply() #forward pass (call apply)

        out : dict[str, jax.Array] = {}

        #slice
        idx = 0
        for attribute_col in self.domain:
            n = self.domain[attribute_col]
            out[attribute_col] = jax.nn.softmax( h[:, idx : idx+n], axis=1 )
            idx += n
        
        
        return out

    def project(self, attrs: str | Sequence[str]) -> Factor:

        if isinstance(attrs, (str, int)):
            attrs = (attrs,)
        attrs = tuple(attrs)

        letters = "bcdefghijklmnopqrstuvwxyz"[: len(attrs)]
        formula = ",".join(f"a{l}" for l in letters) + "->" + "".join(letters)

        products = self.products()
        components = [products[a] for a in attrs]

        values = jnp.einsum(formula, *components) * self.total / self.num_components()
        return Factor(self.domain.project(attrs), values)

    
    def synthetic_data(self, rows=None, method=None) -> Dataset:
        """Generate synthetic data via randomized rounding."""
        total = max(1, int(rows or self.total))
        rng = np.random.default_rng()
        products = self.products()
        subtotal = total // self.num_components() + 1

        blocks = []
        for k in range(self.num_components()):
            comp_data = {}
            for col in self.domain.attributes:
                counts = np.asarray(products[col][k])
                counts = counts * subtotal / counts.sum()
                frac, integ = np.modf(counts)
                integ = integ.astype(int)
                extra = subtotal - integ.sum()
                if extra > 0:
                    p = frac / frac.sum()
                    idx = rng.choice(len(counts), extra, replace=False, p=p)
                    integ[idx] += 1
                vals = np.repeat(np.arange(len(counts)), integ)
                rng.shuffle(vals)
                comp_data[col] = vals
            blocks.append(
                np.stack([comp_data[col] for col in self.domain.attributes], axis=1)
            )

        full_data = np.concatenate(blocks, axis=0)
        rng.shuffle(full_data)
        full_data = full_data[:total]

        data = {col: full_data[:, i] for i, col in enumerate(self.domain.attributes)}
        return Dataset(data, self.domain)


def _marginal_closure(domain: Domain, measurements: list) -> list:
    """Gem+ line 7: add the noisy sub-marginals implied by each measurement.

    For every measurement over a clique of 2+ attributes, each nonempty proper
    subset of its attributes is estimated by summing the noisy marginal over
    the dropped attributes. This is post-processing of already-noised values,
    so it has no privacy cost.

    Rules:
      * Direct measurements always win: a subset that was measured directly
        is never derived.
      * Otherwise the first measurement (in the order given) that implies a
        subset wins, matching Gem+'s round-by-round semantics where a
        sub-marginal added to M in an earlier round stays there.
      * Only identity (datavector) measurements are expanded; other query
        types are passed through unchanged.

    Each derived cell is a sum of k independent noisy cells, where k is the
    product of the dropped attributes' sizes, so its stddev is stddev * sqrt(k).

    Returns the original measurements followed by the derived ones.
    """
    covered = {frozenset(m.clique) for m in measurements}
    derived = []

    for m in measurements:
        clique = tuple(m.clique)
        if len(clique) < 2 or m.query != Factor.datavector:
            continue

        # mbi stores a marginal's cells in clique order
        sizes = tuple(domain[a] for a in clique)
        table = jnp.reshape(jnp.asarray(m.noisy_measurement), sizes)

        for r in range(len(clique) - 1, 0, -1):
            for keep in combinations(range(len(clique)), r):
                subset = tuple(clique[i] for i in keep)
                if frozenset(subset) in covered:
                    continue
                covered.add(frozenset(subset))

                dropped = tuple(i for i in range(len(clique)) if i not in keep)
                values = table.sum(axis=dropped).flatten()
                cells_summed = prod(sizes[i] for i in dropped)

                derived.append(
                    LinearMeasurement(
                        values, subset, stddev=m.stddev * sqrt(cells_summed)
                    )
                )

    return list(measurements) + derived


def gem_plus(
    domain: Domain,
    loss_fn,
    *,
    # contract, matching mirror_descent
    potentials=None,
    warm_start=None,
    # hyperparameters
    optimizer: optax.GradientTransformation | None = None,
    dim: int = 512,
    batch: int = 25000,
    hidden: tuple[int, ...] | None = None,
    steps: int = 1500,
    seed: int = 0,
    weighted: bool = False
) -> GemPlusModel:
    """Fit a GemPlusModel to noisy marginals by gradient descent.

    Drop-in replacement for mbi.estimation.mirror_descent, implementing line 8
    of Gem+: minimize the mean unweighted L1 distance between the generator's
    soft marginals and the noisy measurements.

    `potentials` is accepted for signature compatibility and ignored; it is
    shaped for a graphical model and has no analogue in a neural generator.
    Warm starting comes from `warm_start`, the model fitted in the preceding
    round, whose weights, noise batch and optimizer state are all reused.
    Reusing `_z` is not only a convergence matter: AIM anneals on
    ||q(theta_t(z)) - q(theta_{t-1}(z))||, which measures learning rather than
    reinitialization noise only if both generators share the same z.

    MBI keeps optimizer state beside the model in a separate state object.
    Here it rides on the model because the estimator contract returns a model
    and nothing else, so that is the only channel across rounds.  It is not
    part of the represented distribution.
    """
    measurements = loss_fn
    total = minimum_variance_unbiased_total(measurements)  # originals only
    measurements = _marginal_closure(domain, measurements)  # Gem+ line 7
    cliques = maximal_subset([m.clique for m in measurements])

    if optimizer is None:
        optimizer = optax.adam(1e-4)

    if isinstance(warm_start, GemPlusModel):
        model = warm_start
    else:
        model = GemPlusModel.random(domain, total, dim=dim, batch=batch, hidden=hidden, seed=seed)

    params, z = model._params, model._z
    # opt_state = (
    #     model._opt_state
    #     if model._opt_state is not None
    #     else optimizer.init(params)
    # )

    opt_state = optimizer.init(params)

    def marginal_loss(marginals: CliqueVector):
        """L(theta) = mean over M of ||q(theta(z)) - q~||_1."""
        loss = 0.0
        for m in measurements:
            mu = marginals.project(m.clique)
            err = jnp.sum(jnp.abs(m.query(mu) - m.noisy_measurement))
            loss += err / m.stddev if weighted else err
        return loss / len(measurements)

    def objective(params):
        current = GemPlusModel(params, z, domain, total)
        return marginal_loss(
            CliqueVector(domain, cliques, {cl: current.project(cl) for cl in cliques})
        )

    @jax.jit
    def update(params, opt_state):
        loss, grads = jax.value_and_grad(objective)(params)
        updates, opt_state = optimizer.update(grads, opt_state)
        return optax.apply_updates(params, updates), opt_state, loss

    for _ in range(steps):
        params, opt_state, loss = update(params, opt_state)

    return GemPlusModel(params, z, domain, total)