from __future__ import annotations
from typing import NamedTuple, Optional, Literal
from opendp.mod import Domain
from opendp._convert import ATOM_MAP


def check_norm_and_p(norm, p):
    if (norm is None) != (p is None):
        raise ValueError("norm and p must both be set")

    if norm is not None:
        if not isinstance(norm, float):
            raise ValueError("norm must be float")
        if norm < 0.0:
            raise ValueError("norm must be non-negative")

    if p not in {None, 1, 2}:
        raise ValueError("p must be 1 or 2")


def check_nonnegative_int(v, name):
    if v is not None:
        if not isinstance(v, int):
            raise ValueError(f"{name} must be an integer")
        if v < 0:
            raise ValueError(f"{name} must be non-negative")


def np_array2_domain(
    *, norm=None, p=None, origin=None, size=None, num_columns=None, T=None
) -> Domain:
    """Construct a new Domain representing 2-dimensional numpy arrays.

    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param origin: center of the norm region. Assumed to be at zero
    :param size: number of rows in data
    :param num_columns: number of columns in the data
    :param T: atom type
    """
    import numpy as np  # type: ignore[import]
    import opendp.prelude as dp

    class NPArray2Descriptor(NamedTuple):
        origin: Optional[np.ndarray]
        norm: Optional[np.ndarray]
        p: Literal[1, 2, None]
        size: Optional[int]
        num_columns: Optional[int]
        T: str | dp.RuntimeType

    check_norm_and_p(norm, p)

    if norm is not None:
        # normalize origin to a scalar
        origin = origin if origin is not None else 0.0

    if norm is None and origin is not None:
        raise ValueError("origin may only be set if data has bounded norm")

    if isinstance(origin, (int, float)):
        # normalize origin to a 1d-ndarray
        origin = np.array(origin)

    if isinstance(origin, np.ndarray):
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

    check_nonnegative_int(size, "size")
    check_nonnegative_int(num_columns, "num_columns")

    T = dp.parse_or_infer(T, norm)
    if T not in ATOM_MAP:
        raise ValueError(f"T must be in {ATOM_MAP}")

    desc = NPArray2Descriptor(
        origin=origin,
        norm=norm,
        p=p,
        size=size,
        num_columns=num_columns,
        T=T,
    )

    def member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.ndim != 2:
            raise ValueError("Expected 2-dimensional array")
        if num_columns is not None and x.shape[0] != num_columns:
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

    attrs = ", ".join(f"{k}={v}" for k, v in desc._asdict().items() if v is not None)
    return dp.user_domain(f"NPArray2Domain({attrs})", member, desc)


def _np_SSCP_domain(*, norm=None, p=None, size=None, num_features=None, T) -> Domain:
    """The domain of sums of squares and cross products matrices formed by computing x^Tx,
    for some dataset x.

    :param num_features: number of rows/columns in the matrix
    :param norm: each row in x is bounded by the norm
    :param p: designates L`p` norm
    :param size: number of rows in data
    """
    import opendp.prelude as dp
    import numpy as np  # type: ignore[import]

    check_norm_and_p(norm, p)
    check_nonnegative_int(size, "size")
    check_nonnegative_int(num_features, "num_features")

    class NPSSCPDescriptor(NamedTuple):
        num_features: Optional[np.ndarray]
        norm: Optional[np.ndarray]
        p: Literal[1, 2, None]
        size: Optional[int]
        T: str | dp.RuntimeType

    desc = NPSSCPDescriptor(
        num_features=num_features,
        norm=norm,
        p=p,
        size=size,
        T=dp.RuntimeType.parse(T),
    )

    def member(x):
        import numpy as np  # type: ignore[import]

        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.shape != (num_features,) * 2:
            raise ValueError(f"expected a square array with {num_features} features")
        return True

    attrs = ", ".join(f"{k}={v}" for k, v in desc._asdict().items() if v is not None)
    ident = f"NPSSCPDomain({attrs})"

    return dp.user_domain(ident, member, desc)
