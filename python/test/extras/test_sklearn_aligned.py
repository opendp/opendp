import numpy as np
import pytest

import opendp.prelude as dp


@pytest.fixture(autouse=True)
def _enable_contrib():
    dp.enable_features("contrib")


def _domains(size=2):
    return (
        dp.numpy.array2_domain(T=float, size=size, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=int), size=size),
    )


def test_sklearn_fit_membership_and_optional_fields():
    X_domain, y_domain = _domains()
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    value = dp.sklearn.SklearnFit(np.ones((2, 2)), [0, 1])
    assert domain.cast(dp.sklearn.SklearnFitDomain) == dp.sklearn.SklearnFitDomain(X_domain, y_domain)
    assert domain.member(value)
    assert not domain.member(dp.sklearn.SklearnFit(value.X))
    assert not domain.member(value.X)
    X_only = dp.sklearn.sklearn_fit_domain(X_domain)
    assert X_only.member(dp.sklearn.SklearnFit(value.X))
    assert not X_only.member(value)


def test_sklearn_fit_membership_requires_runtime_row_alignment():
    X_domain, y_domain = _domains(size=None)
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    for rows in (0, 1, 3):
        assert domain.member(dp.sklearn.SklearnFit(np.ones((rows, 2)), [0] * rows))
    assert not domain.member(dp.sklearn.SklearnFit(np.ones((2, 2)), [0]))
    assert not domain.member(dp.sklearn.SklearnFit(np.ones((1, 2)), [0, 1]))


def test_sklearn_fit_uses_one_symmetric_row_adjacency():
    X_domain, y_domain = _domains()
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    data = dp.sklearn.SklearnFit(np.ones((2, 2)), [0, 1])
    context = dp.Context.compositor(
        data, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=domain, split_evenly_over=1,
    )
    assert context.query()._chain == (domain, dp.symmetric_distance())
    assert context.query()._d_in == 1
    assert domain.member(data)


@pytest.mark.parametrize("sparse_type", ["csr_matrix", "csr_array"])
def test_sklearn_fit_membership_with_sparse_features(sparse_type):
    from opendp._internal import _extrinsic_domain

    sparse = pytest.importorskip("scipy.sparse")
    X_domain = _extrinsic_domain("SparseMatrix", lambda value: sparse.issparse(value))
    domain = dp.sklearn.sklearn_fit_domain(X_domain, dp.vector_domain(dp.atom_domain(T=int)))
    X = getattr(sparse, sparse_type)(np.ones((2, 2)))
    assert domain.member(dp.sklearn.SklearnFit(X, [0, 1]))
    assert not domain.member(dp.sklearn.SklearnFit(X, [0]))
    assert not domain.member(dp.sklearn.SklearnFit(X, [0, 1, 2]))
    empty = getattr(sparse, sparse_type)((0, 2))
    assert domain.member(dp.sklearn.SklearnFit(empty, []))
