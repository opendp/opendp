from dataclasses import dataclass
from operator import index

import scipy.sparse as sp

from opendp._internal import _extrinsic_domain
from opendp.mod import Domain
from opendp.typing import RuntimeType, RuntimeTypeDescriptor, _ELEMENTARY_TYPES


@dataclass(frozen=True)
class CSRArray2Descriptor:
    n_features: int
    max_rows: int | None
    T: str | RuntimeType


def _member(x, desc: CSRArray2Descriptor) -> bool:
    return (
        isinstance(x, sp.csr_array)
        and x.ndim == 2
        and _ELEMENTARY_TYPES.get(x.dtype.type) == desc.T
        and x.has_canonical_format
        and bool(x.data.all())
        and x.shape[1] == desc.n_features
        and (desc.max_rows is None or x.shape[0] <= desc.max_rows)
    )


def csr_array2_domain(
    n_features: int,
    max_rows: int | None = None,
    *,
    T: RuntimeTypeDescriptor,
) -> Domain:
    """Domain of canonical 2D CSR arrays of atom type T, without explicit zeros."""
    if isinstance(n_features, bool) or index(n_features) <= 0:
        raise ValueError("n_features must be positive")
    if max_rows is not None and (isinstance(max_rows, bool) or index(max_rows) < 0):
        raise ValueError("max_rows must be non-negative")
    n_features = index(n_features)
    max_rows = None if max_rows is None else index(max_rows)

    T = RuntimeType.parse(T)
    descriptor = CSRArray2Descriptor(n_features, max_rows, T)
    return _extrinsic_domain(
        f"CSRArray2Domain(n_features={n_features}, max_rows={max_rows}, T={T})",
        lambda x: _member(x, descriptor),
        descriptor,
    )
