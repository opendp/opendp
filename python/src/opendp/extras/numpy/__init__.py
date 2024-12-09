'''
This module requires extra installs: ``pip install 'opendp[numpy]'``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.numpy``.    
'''

from __future__ import annotations
from typing import NamedTuple, Literal
from opendp.mod import Domain, Metric, Transformation
from opendp.typing import RuntimeTypeDescriptor, ELEMENTARY_TYPES
from opendp._convert import ATOM_MAP
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import register_transformation
from opendp._internal import _make_transformation, _extrinsic_domain
import typing

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
    T: RuntimeTypeDescriptor | None = None,
) -> Domain:
    """Construct a Domain representing 2-dimensional numpy arrays.

    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param origin: center of the norm region. Assumed to be at zero
    :param size: number of rows in data
    :param num_columns: number of columns in the data
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


    T = T or ELEMENTARY_TYPES.get(origin.dtype.type)
    if T is None:
        raise ValueError("must specify T, the type of data in the array")  # pragma: no cover
    T = dp.RuntimeType.parse(T)
    if T not in ATOM_MAP:
        raise ValueError("T must be in an elementary type")  # pragma: no cover

    def member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")  # pragma: no cover
        T_actual = ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"expected data of type {T}, got {T_actual}")  # pragma: no cover
        if x.ndim != 2:
            raise ValueError("Expected 2-dimensional array")  # pragma: no cover
        if num_columns is not None and x.shape[1] != num_columns:
            raise ValueError(f"must have {num_columns} columns")  # pragma: no cover
        if origin is not None:
            x = x - origin
        if norm is not None:
            max_norm = np.linalg.norm(x, ord=p, axis=1).max()
            if max_norm > norm:
                raise ValueError(f"row norm is too large. {max_norm} > {norm}")  # pragma: no cover
        if size is not None and len(x) != size:
            raise ValueError(f"expected exactly {size} rows")  # pragma: no cover
        return True

    class NPArray2Descriptor(NamedTuple):
        origin: numpy.ndarray | None
        norm: float | None
        p: Literal[1, 2, None]
        size: int | None
        num_columns: int | None
        T: str | dp.RuntimeType

    desc = NPArray2Descriptor(
        origin=origin,
        norm=norm,
        p=p,
        size=size,
        num_columns=num_columns,
        T=T,
    )

    return _extrinsic_domain(f"NPArray2Domain({_fmt_attrs(desc)})", member, desc)


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

    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param size: number of rows in data
    :param num_features: number of rows/columns in the matrix
    """
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    _check_norm_and_p(norm, p)
    _check_nonnegative_int(size, "size")
    _check_nonnegative_int(num_features, "num_features")

    if T is None:
        raise ValueError("must specify T, the type of data in the array")  # pragma: no cover
    T = dp.RuntimeType.parse(T)
    if T not in {dp.f32, dp.f64}:
        raise ValueError("T must be a float type")

    def member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")  # pragma: no cover
        T_actual = ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"expected data of type {T}, got {T_actual}")
        if x.shape != (num_features,) * 2:
            raise ValueError(f"expected a square array with {num_features} features")  # pragma: no cover
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

    return _extrinsic_domain(f"NPSSCPDomain({_fmt_attrs(desc)})", member, desc)


def make_np_clamp(
    input_domain: Domain, input_metric: Metric, norm, p, origin=None
) -> Transformation:
    """Construct a Transformation that clamps the norm of input data.

    :param input_domain: instance of `array2_domain(...)`
    :param input_metric: instance of `symmetric_distance()`
    :param norm: clamp each row to this norm. Required if data is not already bounded
    :param p: designates L`p` norm
    :param origin: norm clamping is centered on this point. Defaults to zero
    """
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    dp.assert_features("contrib")

    norm = float(norm)
    if norm < 0.0:
        raise ValueError("norm must not be negative")  # pragma: no cover
    if p not in {1, 2}:
        raise ValueError("order p must be 1 or 2")  # pragma: no cover

    if origin is None:
        origin = 0.0

    def function(arg):
        arg = arg.copy()
        arg -= origin

        # may have to run multiple times due to FP rounding
        current_norm = np.linalg.norm(arg, ord=p, axis=1, keepdims=True)
        while current_norm.max() > norm:
            arg /= np.maximum(current_norm / norm, 1)
            current_norm = np.linalg.norm(arg, ord=p, axis=1, keepdims=True)

        arg += origin
        return arg

    kwargs = input_domain.descriptor._asdict() | {
        "norm": norm,
        "p": p,
        "origin": origin,
    }
    return _make_transformation(
        input_domain,
        input_metric,
        dp.numpy.array2_domain(**kwargs),
        input_metric,
        function,
        lambda d_in: d_in,
    )


# generate then variant of the constructor
# TODO: Show this in the API Reference?
then_np_clamp = register_transformation(make_np_clamp)