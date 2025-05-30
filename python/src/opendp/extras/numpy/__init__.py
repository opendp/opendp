'''
This module requires extra installs: ``pip install 'opendp[numpy]'``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.numpy``.    
'''

from __future__ import annotations
from typing import NamedTuple, Literal, Optional
from opendp.mod import Domain
from opendp.typing import RuntimeTypeDescriptor, _ELEMENTARY_TYPES, _PRIMITIVE_TYPES
from opendp._lib import import_optional_dependency
from opendp._internal import _extrinsic_domain
import typing
from opendp.extras.numpy._make_np_clamp import make_np_clamp, then_np_clamp # noqa: F401

if typing.TYPE_CHECKING: # pragma: no cover
    import numpy # type: ignore[import-not-found]

def _check_norm_and_p(norm: float | None, p: int | None):
    """Checks that a scalar L`p` `norm` is well-defined"""
    if (norm is None) != (p is None):
        raise ValueError("norm and p must both be set")

    if norm is not None:
        if isinstance(norm, int):
            norm = float(norm)
        if not isinstance(norm, float):
            raise ValueError("norm must be float")  # pragma: no cover
        if norm < 0.0:
            raise ValueError("norm must be non-negative")  # pragma: no cover

    if p not in {None, 1, 2}:
        raise ValueError("p must be 1 or 2")  # pragma: no cover


def _check_nonnegative_int(v: int | None, name: str):
    if v is not None:
        if not isinstance(v, int):
            raise ValueError(f"{name} must be an integer")  # pragma: no cover
        if v < 0:
            raise ValueError(f"{name} must be non-negative")  # pragma: no cover


def _fmt_attrs(attrs: NamedTuple) -> str:
    return ", ".join(f"{k}={v}" for k, v in attrs._asdict().items() if v is not None)


def array2_domain(
    *,
    norm: float | None = None,
    p: Literal[1, 2, None] = None,
    origin=None,
    size: int | None = None,
    num_columns: int | None = None,
    nan: Optional[bool] = None,
    T: RuntimeTypeDescriptor | None = None,
) -> Domain:
    """Construct a Domain representing 2-dimensional numpy arrays.

    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param origin: center of the norm region. Assumed to be at zero
    :param size: number of rows in data
    :param num_columns: number of columns in the data
    :param nan: whether NaN values are allowed
    :param T: atom type
    """
    np = import_optional_dependency('numpy')
    import opendp.prelude as dp

    _check_norm_and_p(norm, p)

    if norm is not None:
        # normalize origin to a scalar
        origin = origin if origin is not None else 0.0

    if norm is None and origin is not None:
        raise ValueError("origin may only be set if data has bounded norm")  # pragma: no cover

    if isinstance(origin, (int, float)):
        # normalize origin to a 1d-ndarray
        origin = np.array(origin)

    if isinstance(origin, np.ndarray):
        if origin.dtype.kind in {"i", "u"}:
            origin = origin.astype(float)

        if origin.dtype.kind != "f":
            raise ValueError("origin array must be numeric")

        if origin.ndim == 0:
            if origin != 0:
                raise ValueError("scalar origin must be zero")
            if num_columns is not None:
                # normalize to a 1d-ndarray
                origin = np.repeat(origin, num_columns)

        if origin.ndim == 1:
            if num_columns is None:
                num_columns = origin.size
            if num_columns != origin.size:
                raise ValueError(f"origin must have num_columns={num_columns} values")

        if origin.ndim not in {0, 1}:
            raise ValueError("origin must have 0 or 1 dimensions")  # pragma: no cover

    elif origin is not None:
        raise ValueError("origin must be a scalar or ndarray")

    _check_nonnegative_int(size, "size")
    _check_nonnegative_int(num_columns, "num_columns")
    
    T = T or _ELEMENTARY_TYPES.get(origin.dtype.type)
    if T is None:
        raise ValueError("must specify T, the type of data in the array")  # pragma: no cover
    T = dp.RuntimeType.parse(T)
    if T not in _PRIMITIVE_TYPES:
        raise ValueError(f"T ({T}) must be a primitive type")

    if nan is None:
        nan = T in {"f32", "f64"}

    def _member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        T_actual = _ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"must have data of type {T}, got {T_actual}")
        if x.ndim != 2:
            raise ValueError("must be a 2-dimensional array")
        if num_columns is not None and x.shape[1] != num_columns:
            raise ValueError(f"must have {num_columns} columns")
        
        if T in {"f32", "f64"} and not nan and np.isnan(x).any():
            raise ValueError("must not contain NaN values")

        if origin is not None:
            x = x - origin
        if norm is not None and np.linalg.norm(x, ord=p, axis=1).max() > norm:
            raise ValueError(f"must have row norm at most {norm}")
        if size is not None and len(x) != size:
            raise ValueError(f"must have exactly {size} rows")
        return True

    class NPArray2Descriptor(NamedTuple):
        origin: numpy.ndarray | None
        norm: float | None
        p: Literal[1, 2, None]
        size: int | None
        num_columns: int | None
        nan: bool
        T: str | dp.RuntimeType

    desc = NPArray2Descriptor(
        origin=origin,
        norm=norm,
        p=p,
        size=size,
        num_columns=num_columns,
        nan=nan,
        T=T,
    )

    return _extrinsic_domain(f"NPArray2Domain({_fmt_attrs(desc)})", _member, desc)


def _sscp_domain(
    *,
    norm: float | None = None,
    p: Literal[1, 2, None] = None,
    size: int | None = None,
    num_features: int | None = None,
    T: RuntimeTypeDescriptor = float,
) -> Domain:
    """The domain of sums of squares and cross products matrices formed by computing x^Tx,
    for some dataset x.

    Elements are finite, members are square symmetric positive semi-definite matrices.

    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param size: number of rows in data
    :param num_features: number of rows/columns in the matrix
    :param T: the type of data elements in the array
    """
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    _check_norm_and_p(norm, p)
    _check_nonnegative_int(size, "size")
    _check_nonnegative_int(num_features, "num_features")

    T = dp.RuntimeType.parse(T)
    if T not in {dp.f32, dp.f64}:
        raise ValueError("T must be a float type")

    def _member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        T_actual = _ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"must have data of type {T}, got {T_actual}")
        if x.shape != (x.shape[0], x.shape[0]):
            raise ValueError("must be a square array")
        if num_features is not None and x.shape[0] != num_features:
            raise ValueError(f"must have {num_features} features")
        if (~np.isfinite(x)).any():
            raise ValueError("must have finite values")
        if np.any(x != x.T):
            raise ValueError("must be symmetric")
        if np.any(np.linalg.eigvals(x) < 0):
            raise ValueError("must be positive semi-definite")
        return True

    class NPSSCPDescriptor(NamedTuple):
        num_features: int | None
        norm: float | None
        p: Literal[1, 2, None]
        size: int | None
        T: str | dp.RuntimeType

    desc = NPSSCPDescriptor(
        num_features=num_features,
        norm=norm,
        p=p,
        size=size,
        T=T,
    )

    return _extrinsic_domain(f"NPSSCPDomain({_fmt_attrs(desc)})", _member, desc)

def make_private_theil_sen(x_bounds, y_bounds, scale, runs=1):
    import numpy as np
    import opendp.prelude as dp

    def pairwise_predict(data, x_cuts):
        '''
        The function matches data points together into pairs, 
        fits a line that passes through each pair of points, 
        and then computes the y values of each line at `x_cuts`.
        That is, it predicts the y values at `x_cuts`, pairwise.
        '''
        # Get an even number of rows.
        data = np.array(data, copy=True)[: len(data) // 2 * 2]

        # Shuffle the data to get random matchings.
        np.random.shuffle(data)

        # Split data into pairs, where pair i is (p1[i], p2[i]).
        p1, p2 = np.array_split(data, 2)

        # Compute differences.
        dx, dy = (p2 - p1).T

        # Compute the midpoints of the pairs.
        x_bar, y_bar = (p1 + p2).T / 2

        # Compute y values on the pairwise slopes at x_cuts.
        points = dy / dx * (x_cuts[None].T - x_bar) + y_bar

        # Pairs where the x difference is zero are degenerate.
        return points.T[dx != 0]
    
    def make_pairwise_predict(x_cuts, runs: int = 1):
        '''
        The parameter `runs` controls how many times randomized pairwise predictions are computed. 
        The default is 1. Increasing `runs` can improve the robustness and accuracy of the results; 
        however, it can also increase computational cost and amount of noise needed later in the algorithm. 
        '''
        return dp.t.make_user_transformation(
            # Outputs are Nx2 non-nan float numpy arrays.
            input_domain=dp.numpy.array2_domain(num_columns=2, T=float),
            # Neighboring input datasets differ by addition/removal of rows.
            input_metric=dp.symmetric_distance(),
            # Outputs are Nx2 non-nan float numpy arrays, but are half as long.
            output_domain=dp.numpy.array2_domain(num_columns=2, T=float),
            # Neighboring output datasets also differ by additions/removals.
            output_metric=dp.symmetric_distance(),
            # Apply the function `runs` times.
            function=lambda x: np.vstack(
                [pairwise_predict(x, x_cuts) for _ in range(runs)]
            ),
            # Each execution of pairwise predict contributes b_in records.
            stability_map=lambda b_in: b_in * runs,
        )
    
    def make_select_column(j):
        return dp.t.make_user_transformation(
            input_domain=dp.numpy.array2_domain(num_columns=2, T=float),
            input_metric=dp.symmetric_distance(),
            output_domain=dp.vector_domain(dp.atom_domain(T=float)),
            output_metric=dp.symmetric_distance(),
            function=lambda x: x[:, j],
            stability_map=lambda b_in: b_in)
    
    def make_private_percentile_medians(y_bounds, scale):
        # this median mechanism favors candidates closest to the true median
        m_median = dp.m.then_private_quantile(
            # 100 evenly spaced points between y_bounds
            candidates=np.linspace(*y_bounds, 100),
            alpha=0.5,
            scale=scale,
        )
        # apply the median mechanism to the 25th and 75th percentile columns
        return dp.c.make_basic_composition([
            make_select_column(0) >> dp.t.then_drop_null() >> m_median,
            make_select_column(1) >> dp.t.then_drop_null() >> m_median,
        ])

    # x_cuts are the 25th and 75th percentiles of x_bounds. 
    # We'll predict y's at these x_cuts.
    x_cuts = x_bounds[0] + (x_bounds[1] - x_bounds[0]) * np.array([0.25, 0.75])

    # we want coefficients, not y values! 
    # Luckily y values are related to coefficients via a linear system
    P_inv = np.linalg.inv(np.vstack([x_cuts, np.ones_like(x_cuts)]).T)

    return (
        # pairwise_predict will return a 2xN array of y values at the x_cuts.
        make_pairwise_predict(x_cuts, runs)
        # privately select median y values for each x_cut
        >> make_private_percentile_medians(y_bounds, scale)
        # transform median y values to coefficients (slope and intercept)
        >> (lambda ys: P_inv @ ys)
    )