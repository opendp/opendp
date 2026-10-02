"""Aligned private inputs for scikit-learn-style estimators.

``Aligned`` keeps the row-aligned private inputs to an estimator together.  When
an ``aligned_domain`` is paired with ``symmetric_distance()``, a row
``(X[i], y[i], sample_weight[i], groups[i])`` is one adjacency event: its fields
are not independently adjacent datasets.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, cast

from opendp._internal import _extrinsic_domain
from opendp.mod import Domain, ExtrinsicDomain, VectorDomain

__all__ = ["Aligned", "AlignedDomain", "aligned_domain"]


@dataclass
class Aligned:
    """Private sklearn inputs whose populated fields share sample order."""

    X: Any
    y: Any | None = None
    sample_weight: Any | None = None
    groups: Any | None = None


@dataclass(frozen=True)
class AlignedDomain:
    """Domains for private sklearn inputs with a shared row correspondence.

    The populated fields of an :class:`Aligned` value must have the same number
    of samples in the same order.  This descriptor represents one vertically
    partitioned dataset, not independently adjacent datasets.
    """

    X: Domain
    y: Domain | None = None
    sample_weight: Domain | None = None
    groups: Domain | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.X, Domain):
            raise TypeError("X must be an OpenDP Domain")

        fields = self._fields()
        for name, domain in fields.items():
            if not isinstance(domain, Domain):
                raise TypeError(f"{name} must be an OpenDP Domain")

        counts = {
            name: count
            for name, domain in fields.items()
            if (count := _domain_row_count(domain)) is not None
        }
        if len(set(counts.values())) > 1:
            raise ValueError(
                "aligned domains must have matching row counts; "
                + ", ".join(f"{name}={count}" for name, count in counts.items())
            )

        # This provenance is established by the enclosing aligned dataset, not
        # inferred from a transformation's stability map. Known preserving
        # transformations explicitly propagate it to their output domains.
        object.__setattr__(self.X, "_preserves_row_alignment", True)

    def _fields(self) -> dict[str, Domain]:
        return {
            name: domain
            for name in ("X", "y", "sample_weight", "groups")
            if (domain := getattr(self, name)) is not None
        }

    def member(self, value: Any) -> bool:
        """Return whether ``value`` satisfies domain membership and alignment."""
        if not isinstance(value, Aligned):
            return False

        values: dict[str, Any] = {}
        for name, domain in self._fields().items():
            field = getattr(value, name)
            if field is None or not domain.member(field):
                return False
            values[name] = field

        # Values absent from the descriptor are not private fit inputs in this
        # domain and must not be silently accepted.
        for name in ("X", "y", "sample_weight", "groups"):
            if name not in values and getattr(value, name) is not None:
                return False

        try:
            return len({len(field) for field in values.values()}) == 1
        except TypeError:
            return False


def aligned_domain(
    X: Domain,
    y: Domain | None = None,
    sample_weight: Domain | None = None,
    groups: Domain | None = None,
) -> ExtrinsicDomain:
    """Construct a domain for one private, row-aligned sklearn dataset.

    Pair the returned domain with :func:`opendp.metrics.symmetric_distance`.
    An insertion/removal changes one aligned row regardless of how many of
    ``X``, ``y``, ``sample_weight``, and ``groups`` are populated. Replacing one
    row is the usual two symmetric-distance events.
    """
    descriptor = AlignedDomain(X, y, sample_weight, groups)
    fields = ", ".join(descriptor._fields())
    return cast(
        ExtrinsicDomain,
        _extrinsic_domain(f"AlignedDomain({fields})", descriptor.member, descriptor),
    )


def _domain_row_count(domain: Domain) -> int | None:
    """Return a domain's public row count when it declares one."""
    if isinstance(domain, VectorDomain):
        return domain.size

    descriptor = getattr(domain, "descriptor", None)
    size = getattr(descriptor, "size", None)
    return size if isinstance(size, int) else None
