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


def test_project_X_is_stable_and_does_not_mutate_aligned_inputs():
    dp.enable_features("contrib")
    X_domain, y_domain, weight_domain = _domains()
    domain = dp.sklearn.aligned_domain(
        X_domain, y_domain, weight_domain, y_domain
    )
    data = dp.sklearn.Aligned(
        np.array([[1.0, 2.0], [3.0, 4.0]]), [0, 1], [0.5, 1.5], [2, 3]
    )
    project = dp.sklearn.make_project_X(domain, dp.symmetric_distance())
    assert project.output_domain == X_domain
    assert project(data) is data.X
    assert [project.map(d) for d in (0, 1, 2)] == [0, 1, 2]
    assert data.y == [0, 1]
    assert domain.member(data)
    assert ((domain, dp.symmetric_distance()) >> dp.sklearn.then_project_X())(data) is data.X
    with pytest.raises(ValueError, match="symmetric_distance"):
        dp.sklearn.make_project_X(domain, dp.change_one_distance())


@pytest.mark.parametrize("size", [0, 2])
def test_lift_X_preserves_all_metadata_and_handles_empty_data(size):
    dp.enable_features("contrib")
    X_domain, y_domain, weight_domain = _domains(size=size)
    domain = dp.sklearn.aligned_domain(X_domain, y_domain, weight_domain, y_domain)
    data = dp.sklearn.Aligned(
        np.ones((size, 2)), list(range(size)), [0.5] * size, list(range(size))
    )
    clamp = dp.numpy.make_np_clamp(X_domain, dp.symmetric_distance(), norm=1.0, p=2)
    lift = (domain, dp.symmetric_distance()) >> dp.sklearn.then_lift_X(clamp)
    result = lift(data)
    assert result is not data
    np.testing.assert_array_equal(data.X, np.ones((size, 2)))
    assert np.all(np.linalg.norm(result.X, axis=1) <= 1.0)
    for name in ("y", "sample_weight", "groups"):
        assert getattr(result, name) is getattr(data, name)
        assert getattr(lift.output_domain.descriptor, name) == getattr(domain.descriptor, name)
    assert lift.output_domain.member(result)
    assert [lift.map(d) for d in (0, 1, 2)] == [0, 1, 2]
    query = dp.Query(lift, dp.max_divergence(), d_in=1, d_out=1.0)
    assert query.resolve(allow_transformations=True) is lift
    with pytest.raises(ValueError, match="not yet a measurement"):
        query.resolve()


def test_lift_X_rejects_domain_provenance_and_X_only_stability_as_evidence():
    dp.enable_features("contrib")
    X_domain, y_domain, _ = _domains(size=None)
    domain = dp.sklearn.aligned_domain(X_domain, y_domain)
    metric = dp.symmetric_distance()
    assert X_domain.preserves_row_alignment

    # Reusing an already-marked domain must not authorize sorting/reordering.
    reorder = _make_transformation(
        X_domain, metric, X_domain, metric,
        lambda data: data[::-1], lambda d: d,
    )
    with pytest.raises(ValueError, match="explicitly row-preserving"):
        dp.sklearn.make_lift_X(domain, metric, reorder)

    # A cross-row operation can even preserve positional correspondence and
    # its X multiset while changing every complete row once y is attached.
    def complement_balanced(data):
        return 1 - data if np.sum(data) == data.size / 2 else data

    cross_row = _make_transformation(
        X_domain, metric, X_domain, metric, complement_balanced, lambda d: d,
    )
    with pytest.raises(ValueError, match="explicitly row-preserving"):
        dp.sklearn.make_lift_X(domain, metric, cross_row)


def test_lift_X_checks_domain_and_metrics():
    dp.enable_features("contrib")
    X_domain, y_domain, _ = _domains()
    domain = dp.sklearn.aligned_domain(X_domain, y_domain)
    metric = dp.symmetric_distance()
    clamp = dp.numpy.make_np_clamp(X_domain, metric, norm=1.0, p=2)
    other_X, _, _ = _domains(size=3)
    with pytest.raises(ValueError, match="input domain must match"):
        dp.sklearn.make_lift_X(dp.sklearn.aligned_domain(other_X), metric, clamp)
    with pytest.raises(ValueError, match="symmetric_distance"):
        dp.sklearn.make_lift_X(domain, dp.change_one_distance(), clamp)
    with pytest.raises(TypeError, match="OpenDP Transformation"):
        dp.sklearn.make_lift_X(domain, metric, object())  # type: ignore[arg-type]

    wrong_metric = _make_transformation(
        X_domain, metric, X_domain, dp.change_one_distance(), lambda data: data, lambda d: d,
    )
    with pytest.raises(ValueError, match="X input and output"):
        dp.sklearn.make_lift_X(domain, metric, wrong_metric)


def test_aligned_membership_counts_sparse_rows_without_len():
    from opendp._internal import _extrinsic_domain

    sparse = pytest.importorskip("scipy.sparse")
    dp.enable_features("contrib")
    X_domain = _extrinsic_domain(
        "SparseMatrix", lambda value: sparse.issparse(value) and value.shape[1] == 2
    )
    y_domain = dp.vector_domain(dp.atom_domain(T=int))
    domain = dp.sklearn.aligned_domain(X_domain, y_domain)
    data = dp.sklearn.Aligned(sparse.csr_matrix(np.ones((2, 2))), [0, 1])
    assert domain.member(data)
    assert not domain.member(dp.sklearn.Aligned(data.X, [0]))
    assert dp.sklearn.make_project_X(domain, dp.symmetric_distance())(data) is data.X


def test_lifted_clamp_is_invariant_on_retained_rows_after_insertion():
    dp.enable_features("contrib")
    X_domain, y_domain, weight_domain = _domains(size=None)
    domain = dp.sklearn.aligned_domain(X_domain, y_domain, weight_domain, y_domain)
    clamp = dp.numpy.make_np_clamp(X_domain, dp.symmetric_distance(), norm=1.0, p=2)
    lift = dp.sklearn.make_lift_X(domain, dp.symmetric_distance(), clamp)
    before = dp.sklearn.Aligned(np.array([[0.5, 0.0], [3.0, 4.0]]), [0, 1], [0.5, 1.0], [2, 3])
    after = dp.sklearn.Aligned(
        np.vstack([before.X, [1e150, -1e150]]), [0, 1, 2], [0.5, 1.0, 1.5], [2, 3, 4]
    )
    out_before, out_after = lift(before), lift(after)
    np.testing.assert_array_equal(out_before.X, out_after.X[:2])
    for name in ("y", "sample_weight", "groups"):
        assert getattr(out_before, name) == getattr(out_after, name)[:2]
    assert lift.map(1) == 1
