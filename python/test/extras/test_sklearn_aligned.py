import numpy as np
import pytest

import opendp.prelude as dp


dp.enable_features("contrib")


@pytest.mark.parametrize(
    "dtype, T, y, rows",
    [
        (np.float32, dp.f32, None, 2),
        (np.float64, dp.f64, [0, 1], 2),
        (np.int32, dp.i32, np.zeros(0, dtype=np.int32), 0),
    ],
)
def test_sklearn_fit_domain_inference(dtype, T, y, rows):
    data = dp.sklearn.SklearnFit(np.ones((rows, 4), dtype=dtype), y)
    domain = dp.domain_of(data, infer=True)
    desc = domain.cast(dp.sklearn.SklearnFitDomain)
    assert (desc.X.descriptor.T, desc.X.descriptor.num_columns) == (T, 4)
    assert desc.X.descriptor.size is None and desc.X.descriptor.norm is None
    assert desc.y == (None if y is None else dp.vector_domain(dp.atom_domain(T=int)))
    assert domain.member(data)


@pytest.mark.parametrize("supervised", [False, True])
def test_context_infers_sklearn_fit_domain(supervised):
    data = dp.sklearn.SklearnFit(np.ones((2, 2)), [0, 1] if supervised else None)
    context = dp.Context.compositor(
        data,
        dp.unit_of(contributions=2),
        dp.loss_of(epsilon=1),
        split_evenly_over=1,
    )
    domain = context.accountant.input_domain
    desc = domain.cast(dp.sklearn.SklearnFitDomain)
    assert desc.X.descriptor.size is None
    assert (desc.y is not None) == supervised
    assert domain.member(data)
    assert context.query()._chain == (domain, dp.symmetric_distance())
    assert context.query()._d_in == 2


def test_sklearn_fit_membership_and_row_alignment():
    X_domain = dp.numpy.array2_domain(T=float, size=2, num_columns=2, nan=False)
    y_domain = dp.vector_domain(dp.atom_domain(T=int), size=2)
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    value = dp.sklearn.SklearnFit(np.ones((2, 2)), [0, 1])
    assert domain.member(value)
    assert not domain.member(value.X)
    assert not domain.member(dp.sklearn.SklearnFit(value.X))
    assert not domain.member(dp.sklearn.SklearnFit(value.X, [0]))
    with pytest.warns(UserWarning, match="must have exactly 2 rows"):
        assert not domain.member(dp.sklearn.SklearnFit(np.ones((1, 2)), [0, 1]))
    X_only = dp.sklearn.sklearn_fit_domain(X_domain)
    assert X_only.member(dp.sklearn.SklearnFit(value.X))
    assert not X_only.member(value)
    unbounded = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2),
        dp.vector_domain(dp.atom_domain(T=int)),
    )
    assert unbounded.member(dp.sklearn.SklearnFit(np.ones((0, 2)), []))
    assert not unbounded.member(dp.sklearn.SklearnFit(np.ones((1, 2)), [0, 1]))


@pytest.mark.parametrize("sparse_type", ["csr_matrix", "csr_array"])
def test_sklearn_fit_membership_with_sparse_features(sparse_type):
    from opendp._internal import _extrinsic_domain

    sparse = pytest.importorskip("scipy.sparse")
    X_domain = _extrinsic_domain("SparseMatrix", lambda value: sparse.issparse(value))
    domain = dp.sklearn.sklearn_fit_domain(
        X_domain, dp.vector_domain(dp.atom_domain(T=int))
    )
    X = getattr(sparse, sparse_type)(np.ones((2, 2)))
    assert domain.member(dp.sklearn.SklearnFit(X, [0, 1]))
    assert not domain.member(dp.sklearn.SklearnFit(X, [0]))
    assert domain.member(
        dp.sklearn.SklearnFit(getattr(sparse, sparse_type)((0, 2)), [])
    )
