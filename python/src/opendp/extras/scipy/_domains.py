from dataclasses import dataclass
from typing import Any

from opendp._internal import _extrinsic_domain
from opendp._lib import import_optional_dependency
from opendp.mod import Domain
from opendp.typing import RuntimeType, RuntimeTypeDescriptor, _ELEMENTARY_TYPES


def _sp() -> Any:
    return import_optional_dependency("scipy.sparse")


@dataclass(frozen=True)
class CSRArray2Domain:
    n_features: int
    max_rows: int | None
    T: str | RuntimeType


def csr_array2_domain(
    n_features: int,
    max_rows: int | None,
    *,
    T: RuntimeTypeDescriptor,
) -> Domain:
    """Domain of canonical 2D CSR arrays of atom type T, without explicit zeros."""
    if n_features <= 0:
        raise ValueError("n_features must be positive")
    if max_rows is not None and max_rows < 0:
        raise ValueError("max_rows must be non-negative")

    T = RuntimeType.parse(T)
    descriptor = CSRArray2Domain(n_features, max_rows, T)

    def member(x) -> bool:
        return (
            isinstance(x, _sp().csr_array)
            and x.ndim == 2
            and _ELEMENTARY_TYPES.get(x.dtype.type) == T
            and x.has_canonical_format
            and bool(x.data.all())
            and x.shape[1] == n_features
            and (max_rows is None or x.shape[0] <= max_rows)
        )

    return _extrinsic_domain(
        str(descriptor),
        member,
        descriptor,
    )
