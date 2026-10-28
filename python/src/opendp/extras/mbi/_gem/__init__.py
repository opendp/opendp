from typing import Any, NamedTuple
from dataclasses import dataclass, field
import functools

import jax
import jax.nn
import jax.numpy as jnp
import numpy as np
import optax
from mbi import (
    Dataset,
    DatavectorQuery,
    Domain,
    Factor,
    CliqueVector,
    LinearMeasurement,
    MarginalLossFn,
)
from mbi.extensions.mixture_of_products import MixtureOfProducts
from mbi.clique_utils import maximal_subset
from mbi.estimation import minimum_variance_unbiased_total, Estimator
from collections.abc import Sequence
from itertools import combinations
from math import prod, sqrt


@jax.tree_util.register_dataclass
@dataclass(frozen=True)
class GemPlusModel:
    """Gem+ generator: a fixed Gaussian batch z fed through GEM's generator network,
    giving a mixture of B product distributions (one per row of z).
    :param _params: network weights
    :param _z: initial seeds for person types
    :param domain: metadata for dataset
    :param total: number of rows (N)"""

    _params: Any
    _z: jax.Array
    domain: Domain = field(metadata={"static": True})
    total: float = field(metadata={"static": True})

    def num_components(self) -> int:
        """Number of soft records B"""
        return self._z.shape[0]

    @staticmethod
    def random(
        domain: Domain,
        total: float,
        dim: int = 512,
        batch: int = 25000,
        hidden: tuple[int, ...] | None = None,
        seed: int = 0,
    ) -> "GemPlusModel":
        """New model with a fixed Gaussian batch z and GEM's generator.

        Weights start as in GEM: uniform on ±1/sqrt(fan_in), PyTorch's nn.Linear
        default, and batch norm with gamma = 1, beta = 0. The output layer has
        one score per value of every attribute.

        :param domain: the mbi Domain to generate over
        :param total: number of rows the marginals are scaled to
        :param dim: width of each noise vector
        :param batch: number of components B
        :param hidden: widths of the residual blocks; defaults to (2 * dim, 2 * dim)
        :param seed: random seed for z and the initial weights
        :raises ValueError: if batch < 2, since batch norm needs batch statistics
        """
        if hidden is None:
            hidden = (dim * 2, dim * 2)
        if batch < 2:
            raise ValueError(
                "batch must be at least 2 (batch norm uses batch statistics)"
            )

        key_z, key_w = jax.random.split(jax.random.PRNGKey(seed))
        z = jax.random.normal(key_z, (batch, dim))

        def linear(key, n_in, n_out):
            # torch.nn.Linear default: weights and bias ~ U(-1/sqrt(n_in), 1/sqrt(n_in))
            bound = 1 / sqrt(n_in)
            k_w, k_b = jax.random.split(key)
            return {
                "W": jax.random.uniform(
                    k_w, (n_in, n_out), minval=-bound, maxval=bound
                ),
                "b": jax.random.uniform(k_b, (n_out,), minval=-bound, maxval=bound),
            }

        keys = jax.random.split(key_w, len(hidden) + 1)
        blocks, width = [], dim
        for k, n_out in zip(keys, hidden):
            blocks.append(
                {
                    **linear(k, width, n_out),
                    "gamma": jnp.ones(n_out),
                    "beta": jnp.zeros(n_out),
                }
            )
            width += n_out  # each block's output is concatenated with its input
        out = linear(keys[-1], width, sum(domain.shape))
        return GemPlusModel({"blocks": blocks, "out": out}, z, domain, total)

    def _apply(self) -> jax.Array:
        """GEM's generator: residual blocks (Linear, BatchNorm, ReLU, concat input), then Linear."""
        h = self._z
        for p in self._params["blocks"]:
            out = h @ p["W"] + p["b"]
            out = (out - out.mean(axis=0)) / jnp.sqrt(
                out.var(axis=0) + 1e-5
            )  # BatchNorm1d, eps=1e-5
            out = out * p["gamma"] + p["beta"]
            h = jnp.concatenate(
                [jax.nn.relu(out), h], axis=1
            )  # same order as GEM: [out, input]
        return h @ self._params["out"]["W"] + self._params["out"]["b"]

    @functools.cached_property
    def _mixture(self) -> MixtureOfProducts:
        """The network's output as mbi's mixture of products.

        Computed once per model: weights are fixed after construction, and AIM
        calls project() once per candidate clique every round."""
        h, logits, idx = self._apply(), {}, 0
        for col in self.domain.attributes:
            n = self.domain[col]
            logits[col] = h[:, idx : idx + n]
            idx += n
        return MixtureOfProducts(logits, self.domain, self.total)

    def project(self, attrs: str | Sequence[str]) -> Factor:
        """Marginal over ``attrs``, scaled to ``total`` rows: the average over the
        B components of each component's product distribution."""
        return self._mixture.project(attrs)

    def supports(self, attrs: str | Sequence[str]) -> bool:
        """A mixture of products can answer any marginal over its domain."""
        return self.domain.supports(attrs)

    def synthetic_data(self, rows=None, method=None):
        """Synthetic rows via randomized rounding. ``method`` is accepted for
        compatibility with MarkovRandomField.synthetic_data and ignored."""
        return self._mixture.synthetic_data(rows)


def _marginal_closure(domain: Domain, measurements: list) -> list:
    """Gem+ line 7: add the noisy sub-marginals implied by each measurement.

    Sub-marginals are computed by summing over the dropped attributes, so this is
    post-processing with no privacy cost. Directly measured cliques are never
    re-derived, and otherwise the first measurement implying a clique is used.
    A derived cell sums k noisy cells, so its stddev is stddev * sqrt(k).
    """
    covered = {frozenset(m.clique) for m in measurements}
    derived = []

    for m in measurements:
        clique = tuple(m.clique)
        if len(clique) < 2 or not isinstance(m.query, DatavectorQuery):
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


def _l1_loss(marginals, measurements):
    """Gem+ line 8: mean over measurements of the unweighted L1 error.

    mbi's built-in L1 loss weights each measurement by 1/stddev; Gem+ does not.

    :param marginals: CliqueVector of the model's marginals
    :param measurements: LinearMeasurements, including those added by the closure
    """
    loss = 0.0
    for m in measurements:
        mu = marginals.project(m.clique)
        loss += jnp.sum(jnp.abs(m.query(mu) - m.noisy_measurement))
    return loss / len(measurements)


class GemPlusState(NamedTuple):
    """Optimization state: the model must come first (mbi passes state[0] to callbacks)."""

    model: GemPlusModel
    opt_state: Any


@dataclass(frozen=True, kw_only=True)
class GemPlusEstimator(Estimator):
    """Trains the generator with a fresh optimizer each call, warm-starting params and z."""

    optimizer: optax.GradientTransformation
    dim: int = 512
    batch: int = 25000
    hidden: tuple[int, ...] | None = None
    seed: int = 0

    def _init(self, domain, loss_fn, known_total, *, warm_start=None, constraints=()):

        if constraints:
            raise NotImplementedError("GemPlusEstimator does not support constraints")
        if warm_start is not None and not isinstance(warm_start, GemPlusModel):
            raise TypeError(
                f"warm_start must be a GemPlusModel, got {type(warm_start).__name__}"
            )
        if warm_start is not None and warm_start.domain != domain:
            raise ValueError("warm_start was fit on a different domain")

        if warm_start is not None:
            model = GemPlusModel(warm_start._params, warm_start._z, domain, known_total)
        else:
            model = GemPlusModel.random(
                domain,
                known_total,
                dim=self.dim,
                batch=self.batch,
                hidden=self.hidden,
                seed=self.seed,
            )
        return GemPlusState(model, self.optimizer.init(model._params))

    def _step(self, state: GemPlusState, loss_fn, known_total, constraints=()):
        model = state.model

        def objective(params):
            current = GemPlusModel(params, model._z, model.domain, model.total)
            marginals = CliqueVector(
                current.domain,
                loss_fn.cliques,
                {cl: current.project(cl) for cl in loss_fn.cliques},
            )
            return loss_fn(marginals)

        _, grads = jax.value_and_grad(objective)(model._params)
        updates, opt_state = self.optimizer.update(
            grads, state.opt_state, model._params
        )
        params = optax.apply_updates(model._params, updates)
        return GemPlusState(
            GemPlusModel(params, model._z, model.domain, model.total), opt_state
        )

    def _finalize(self, state, known_total, constraints=()):
        return state.model


def gem_plus(
    domain: Domain,
    loss_fn,
    *,
    potentials=None,
    warm_start=None,
    optimizer: optax.GradientTransformation | None = None,
    dim: int = 512,
    batch: int = 25000,
    hidden: tuple[int, ...] | None = None,
    steps: int = 1500,
    seed: int = 0,
) -> GemPlusModel:
    """Fit a GemPlusModel to noisy marginals using Gem+.

    Adds the sub-marginals implied by each measurement (Gem+ line 7), then trains
    the generator on the mean unweighted L1 error (line 8). ``warm_start`` continues
    from a previous model's weights and noise batch; ``potentials`` is ignored.
    Defaults follow the Gem+ paper (batch, steps) and GEM's code (network, optimizer).

    :param domain: the mbi Domain to fit over
    :param loss_fn: noisy measurements, as a list of LinearMeasurement
    :param potentials: ignored; accepted for OpenDP's estimator signature
    :param warm_start: a GemPlusModel to continue training, or None
    :param optimizer: optax optimizer; defaults to Adam with learning rate 1e-4
    :param dim: width of the noise vector
    :param batch: number of mixture components
    :param hidden: widths of the residual blocks; defaults to (2 * dim, 2 * dim)
    :param steps: optimizer steps, a multiple of 50
    :param seed: random seed for the noise batch and initial weights
    """
    # Estimates must be f64 to pass through the OpenDP FFI.
    jax.config.update("jax_enable_x64", True)
    if steps % 50:
        raise ValueError(f"steps ({steps}) must be a multiple of 50")
    if isinstance(loss_fn, MarginalLossFn):
        raise TypeError(
            "gem_plus needs the measurements themselves (for the marginal closure), not a MarginalLossFn"
        )

    measurements = list(loss_fn)
    total = minimum_variance_unbiased_total(measurements)  # originals only
    measurements = _marginal_closure(domain, measurements)  # Gem+ line 7

    loss = MarginalLossFn(
        maximal_subset([m.clique for m in measurements]), _l1_loss, tuple(measurements)
    )

    if optimizer is None:
        optimizer = optax.adam(1e-4)

    estimator = GemPlusEstimator(
        optimizer=optimizer,
        dim=dim,
        batch=batch,
        hidden=tuple(hidden) if hidden else None,
        seed=seed,
    )
    return estimator.estimate(
        domain, loss, known_total=total, iters=steps, warm_start=warm_start
    )
