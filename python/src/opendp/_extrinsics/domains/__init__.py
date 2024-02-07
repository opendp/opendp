from __future__ import annotations
from typing import NamedTuple, Literal
from opendp.mod import Domain
from opendp.typing import RuntimeTypeDescriptor, ELEMENTARY_TYPES
from opendp._convert import ATOM_MAP


def _check_norm_and_p(norm: float | None, p: int | None):
    """Checks that a scalar L`p` `norm` is well-defined"""
    if (norm is None) != (p is None):
        raise ValueError("norm and p must both be set")

    if norm is not None:
        if isinstance(norm, int):
            norm = float(norm)
        if not isinstance(norm, float):
            raise ValueError("norm must be float")
        if norm < 0.0:
            raise ValueError("norm must be non-negative")

    if p not in {None, 1, 2}:
        raise ValueError("p must be 1 or 2")


def _check_nonnegative_int(v: int | None, name: str):
    if v is not None:
        if not isinstance(v, int):
            raise ValueError(f"{name} must be an integer")
        if v < 0:
            raise ValueError(f"{name} must be non-negative")


def _fmt_attrs(attrs: NamedTuple) -> str:
    return ", ".join(f"{k}={v}" for k, v in attrs._asdict().items() if v is not None)


def np_array2_domain(
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
    import numpy as np  # type: ignore[import]
    import opendp.prelude as dp

    _check_norm_and_p(norm, p)

    if norm is not None:
        # normalize origin to a scalar
        origin = origin if origin is not None else 0.0

    if norm is None and origin is not None:
        raise ValueError("origin may only be set if data has bounded norm")

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
            raise ValueError("origin must have 0 or 1 dimensions")

    elif origin is not None:
        raise ValueError("origin must be a scalar or ndarray")

    _check_nonnegative_int(size, "size")
    _check_nonnegative_int(num_columns, "num_columns")


    T = T or ELEMENTARY_TYPES.get(origin.dtype.type)
    if T is None:
        raise ValueError("must specify T, the type of data in the array")
    T = dp.RuntimeType.parse(T)
    if T not in ATOM_MAP:
        raise ValueError(f"T must be in an elementary type")

    def member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        T_actual = ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"expected data of type {T}, got {T_actual}")
        if x.ndim != 2:
            raise ValueError("Expected 2-dimensional array")
        if num_columns is not None and x.shape[1] != num_columns:
            raise ValueError(f"must have {num_columns} columns")
        if origin is not None:
            x = x - origin
        if norm is not None:
            max_norm = np.linalg.norm(x, ord=p, axis=1).max()
            if max_norm > norm:
                raise ValueError(f"row norm is too large. {max_norm} > {norm}")
        if size is not None and len(x) != size:
            raise ValueError(f"expected exactly {size} rows")
        return True

    class NPArray2Descriptor(NamedTuple):
        origin: np.ndarray | None
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

    return dp.user_domain(f"NPArray2Domain({_fmt_attrs(desc)})", member, desc)


def _np_sscp_domain(
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
    import numpy as np  # type: ignore[import]

    _check_norm_and_p(norm, p)
    _check_nonnegative_int(size, "size")
    _check_nonnegative_int(num_features, "num_features")

    if T is None:
        raise ValueError("must specify T, the type of data in the array")
    T = dp.RuntimeType.parse(T)
    if T not in {dp.f32, dp.f64}:
        raise ValueError(f"T must be a float type")

    def member(x):
        import numpy as np  # type: ignore[import]

        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        T_actual = ELEMENTARY_TYPES.get(x.dtype.type)
        if T_actual != T:
            raise TypeError(f"expected data of type {T}, got {T_actual}")
        if x.shape != (num_features,) * 2:
            raise ValueError(f"expected a square array with {num_features} features")
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

    return dp.user_domain(f"NPSSCPDomain({_fmt_attrs(desc)})", member, desc)
