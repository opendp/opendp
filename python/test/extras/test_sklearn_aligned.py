import numpy as np
import pytest

import opendp.prelude as dp
from opendp._internal import _make_transformation


def _domains(size=2):
    return (
        dp.numpy.array2_domain(T=float, size=size, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=int), size=size),
        dp.vector_domain(dp.atom_domain(T=float), size=size),
    )


def test_aligned_domain_membership_and_optional_fields():
    X_domain, y_domain, weight_domain = _domains()
    domain = dp.sklearn.aligned_domain(
        X=X_domain, y=y_domain, sample_weight=weight_domain
    )
    value = dp.sklearn.Aligned(
        X=np.array([[1.0, 2.0], [3.0, 4.0]]),
        y=[0, 1],
        sample_weight=[0.5, 1.5],
    )

    descriptor = domain.cast(dp.sklearn.AlignedDomain)
    assert descriptor.member(value)
    assert domain.member(value)
    assert not descriptor.member(dp.sklearn.Aligned(X=value.X, y=value.y))
    assert not descriptor.member(
        dp.sklearn.Aligned(X=value.X, y=value.y, groups=["a", "b"])
    )

    X_only = dp.sklearn.aligned_domain(X=X_domain)
    assert X_only.member(dp.sklearn.Aligned(X=value.X))
    assert not X_only.cast(dp.sklearn.AlignedDomain).member(value)


def test_aligned_domain_rejects_mismatched_row_counts():
    X_domain, _, _ = _domains()
    with pytest.raises(TypeError, match="X must be an OpenDP Domain"):
        dp.sklearn.AlignedDomain(X=None)  # type: ignore[arg-type]

    y_domain = dp.vector_domain(dp.atom_domain(T=int), size=3)
    with pytest.raises(ValueError, match="matching row counts"):
        dp.sklearn.AlignedDomain(X=X_domain, y=y_domain)

    descriptor = dp.sklearn.AlignedDomain(
        X=X_domain,
        y=dp.vector_domain(dp.atom_domain(T=int)),
    )
    assert not descriptor.member(
        dp.sklearn.Aligned(X=np.array([[1.0, 2.0], [3.0, 4.0]]), y=[0])
    )


def test_aligned_domain_uses_one_symmetric_row_adjacency_space():
    dp.enable_features("contrib")
    X_domain, y_domain, weight_domain = _domains()
    domain = dp.sklearn.aligned_domain(
        X=X_domain, y=y_domain, sample_weight=weight_domain
    )
    data = dp.sklearn.Aligned(
        X=np.array([[1.0, 2.0], [3.0, 4.0]]),
        y=[0, 1],
        sample_weight=[0.5, 1.5],
    )

    # The single metric is over the composite aligned dataset: changing every
    # populated field of one row is one insertion/removal event, not three.
    context = dp.Context.compositor(
        data,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    query = context.query()
    assert query._chain == (domain, dp.symmetric_distance())
    assert query._d_in == 1


def test_row_alignment_propagates_only_from_known_preserving_transformations():
    dp.enable_features("contrib")
    X_domain, _, _ = _domains()

    # Creating an aligned descriptor establishes provenance for its X domain.
    dp.sklearn.AlignedDomain(X=X_domain)
    assert X_domain.preserves_row_alignment

    clamp = dp.numpy.make_np_clamp(X_domain, dp.symmetric_distance(), norm=1.0, p=2)
    assert clamp.output_domain.preserves_row_alignment

    unmarked_domain, _, _ = _domains()
    unmarked_clamp = dp.numpy.make_np_clamp(
        unmarked_domain, dp.symmetric_distance(), norm=1.0, p=2
    )
    assert not unmarked_clamp.output_domain.preserves_row_alignment

    # A d -> d map alone is not provenance: this arbitrary reordering is not
    # marked, even though it has the same stability map as clamping.
    reordered_domain, _, _ = _domains()
    reordering = _make_transformation(
        X_domain,
        dp.symmetric_distance(),
        reordered_domain,
        dp.symmetric_distance(),
        lambda data: data[::-1],
        lambda d_in: d_in,
    )
    assert not reordering.output_domain.preserves_row_alignment
