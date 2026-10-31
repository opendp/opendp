"""Row-aligned private inputs for scikit-learn estimators."""

from dataclasses import dataclass
from typing import Any

from opendp._internal import _extrinsic_domain
from opendp.mod import Domain


@dataclass
class SklearnFit:
    """A vertically partitioned sklearn dataset with aligned rows."""

    X: Any
    y: Any | None = None


@dataclass(frozen=True)
class SklearnFitDomain:
    """Domains for the populated fields of a SklearnFit dataset."""

    X: Domain
    y: Domain | None = None


def sklearn_fit_domain(X, y=None) -> Domain:
    """Construct and return an OpenDP Domain for private, row-aligned sklearn inputs.

    SklearnFitDomain is the descriptor for the returned Domain, not the Domain
    itself. This follows the NPArray2Domain / array2_domain naming convention.
    """
    descriptor = SklearnFitDomain(X, y)

    def member(value):
        if not isinstance(value, SklearnFit):
            return False

        if not X.member(value.X):
            return False

        if (y is None) != (value.y is None):
            return False

        if y is not None:
            if not y.member(value.y):
                return False
            if _n_rows(value.X) != _n_rows(value.y):
                return False

        return True

    return _extrinsic_domain(str(descriptor), member, descriptor)


def _n_rows(value) -> int:
    """Return the number of records along the first axis."""
    shape = getattr(value, "shape", None)
    return int(shape[0]) if shape else len(value)
