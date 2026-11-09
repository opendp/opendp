from dataclasses import dataclass
from operator import index

import scipy.sparse as sp

from opendp._internal import _extrinsic_domain
from opendp.mod import Domain
from opendp.typing import RuntimeType, RuntimeTypeDescriptor, _ELEMENTARY_TYPES


@dataclass(frozen=True)
class CSRArray2Descriptor:
    n_features: int
    size: int | None
    T: str | RuntimeType


def _member(x, desc: CSRArray2Descriptor) -> bool:
    return (
        isinstance(x, sp.csr_array)
        and x.ndim == 2
        and _ELEMENTARY_TYPES.get(x.dtype.type) == desc.T
        and x.has_canonical_format
        and bool(x.data.all())
        and x.shape[1] == desc.n_features
        and (desc.size is None or x.shape[0] == desc.size)
    )


def csr_array2_domain(
    n_features: int,
    *,
    size: int | None = None,
    T: RuntimeTypeDescriptor,
) -> Domain:
    """Domain of canonical 2D CSR arrays, optionally with an exact row count."""
    if isinstance(n_features, bool) or index(n_features) <= 0:
        raise ValueError("n_features must be positive")
    if size is not None and (isinstance(size, bool) or index(size) < 0):
        raise ValueError("size must be non-negative")
    n_features = index(n_features)
    size = None if size is None else index(size)

    T = RuntimeType.parse(T)
    descriptor = CSRArray2Descriptor(n_features, size, T)
    return _extrinsic_domain(
        f"CSRArray2Domain(n_features={n_features}, size={size}, T={T})",
        lambda x: _member(x, descriptor),
        descriptor,
    )
