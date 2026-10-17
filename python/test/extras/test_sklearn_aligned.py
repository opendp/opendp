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


@pytest.mark.parametrize("dtype, T", [(np.float32, dp.f32), (np.float64, dp.f64), (np.int32, dp.i32)])
@pytest.mark.parametrize("target", ["none", "numpy", "list"])
@pytest.mark.parametrize("rows", [0, 3])
def test_sklearn_fit_domain_inference(dtype, T, target, rows):
    X = np.ones((rows, 4), dtype=dtype)
    y = None if target == "none" else [0] * rows if target == "list" else np.zeros(rows, dtype=np.int32)
    if target == "list" and rows == 0:
        with pytest.raises(dp.UnknownTypeException, match="Cannot infer atomic type when empty"):
            dp.domain_of(dp.sklearn.SklearnFit(X, y), infer=True)
        return
    domain = dp.domain_of(dp.sklearn.SklearnFit(X, y), infer=True)
    desc = domain.cast(dp.sklearn.SklearnFitDomain)
    assert desc.X.descriptor.T == T
    assert desc.X.descriptor.num_columns == 4
    assert desc.X.descriptor.size is None
    assert desc.X.descriptor.norm is None
    if target == "none":
        assert desc.y is None
    else:
        assert desc.y == dp.vector_domain(dp.atom_domain(T=int))
        assert desc.y.size is None
    for size in (0, 1, 5):
        features = np.ones((size, 4), dtype=dtype)
        targets = None if y is None else [0] * size
        assert domain.member(dp.sklearn.SklearnFit(features, targets))
        if targets is not None:
            assert not domain.member(dp.sklearn.SklearnFit(features, targets + [0]))


@pytest.mark.parametrize("target, rows", [("none", 0), ("none", 3), ("numpy", 0), ("numpy", 3), ("list", 3)])
def test_context_infers_sklearn_fit_domain(target, rows):
    y = None if target == "none" else [0.0] * rows if target == "list" else np.zeros(rows)
    data = dp.sklearn.SklearnFit(np.ones((rows, 2)), y)
    context = dp.Context.compositor(
        data, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1), split_evenly_over=1,
    )
    domain = context.accountant.input_domain
    desc = domain.cast(dp.sklearn.SklearnFitDomain)
    assert desc.X.descriptor.size is None and desc.X.descriptor.num_columns == 2
    assert desc.X.descriptor.T == dp.f64
    assert (desc.y is not None) == (target != "none")
    assert domain.member(data)
    assert context.query()._chain == (domain, dp.symmetric_distance())
    assert context.current_privacy_loss() == []


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
