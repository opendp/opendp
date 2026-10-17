import pytest

import opendp.prelude as dp
from opendp.extras.scipy import CSRArray2Domain, csr_array2_domain

np = pytest.importorskip("numpy")
sparse = pytest.importorskip("scipy.sparse")

dp.enable_features("contrib")


@pytest.mark.parametrize(
    ("T", "dtype", "expected"),
    [(bool, np.bool_, "bool"), (np.int32, np.int32, "i32"),
     ("i64", np.int64, "i64"), (np.float32, np.float32, "f32"),
     (float, np.float64, "f64")],
)
def test_csr_array2_domain_atom_type(T, dtype, expected):
    domain = csr_array2_domain(3, max_rows=2, T=T)
    assert domain.cast(CSRArray2Domain) == CSRArray2Domain(3, 2, expected)
    assert domain.member(sparse.csr_array([[1, 0, 2]], dtype=dtype))
    assert domain.member(sparse.csr_array((0, 3), dtype=dtype))
    other_dtype = np.float64 if expected != "f64" else np.int32
    assert not domain.member(sparse.csr_array([[1, 0, 2]], dtype=other_dtype))


@pytest.mark.parametrize(
    "dtype, T", [(np.bool_, "bool"), (np.int32, "i32"), (np.int64, "i64"),
                 (np.float32, "f32"), (np.float64, "f64")],
)
@pytest.mark.parametrize("rows", [0, 2])
def test_csr_domain_inference_leaves_row_count_unspecified(dtype, T, rows):
    data = sparse.csr_array(np.ones((rows, 3), dtype=dtype))
    domain = dp.domain_of(data, infer=True)
    assert domain.cast(CSRArray2Domain) == CSRArray2Domain(3, None, T)
    for size in (0, 1, 5):
        assert domain.member(sparse.csr_array(np.ones((size, 3), dtype=dtype)))
    assert not domain.member(sparse.csr_array(np.ones((rows, 4), dtype=dtype)))
    other_dtype = np.float64 if T != "f64" else np.int32
    assert not domain.member(sparse.csr_array(np.ones((rows, 3), dtype=other_dtype)))


def test_csr_inference_rejects_one_dimensional_arrays():
    try:
        data = sparse.csr_array([True, False, True])
    except ValueError:
        pytest.skip("installed scipy does not support 1D CSR arrays")
    if data.ndim != 1:
        pytest.skip("installed scipy normalizes 1D inputs to 2D CSR arrays")
    with pytest.raises(dp.UnknownTypeException):
        dp.domain_of(data, infer=True)


@pytest.mark.parametrize("with_target", [False, True])
@pytest.mark.parametrize("rows", [0, 2])
def test_context_infers_sparse_sklearn_fit_domain(with_target, rows):
    y = np.zeros(rows, dtype=np.int32) if with_target else None
    data = dp.sklearn.SklearnFit(sparse.csr_array(np.ones((rows, 3), dtype=bool)), y)
    context = dp.Context.compositor(
        data, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1), split_evenly_over=1,
    )
    domain = context.accountant.input_domain
    desc = domain.cast(dp.sklearn.SklearnFitDomain)
    assert desc.X.cast(CSRArray2Domain) == CSRArray2Domain(3, None, "bool")
    if with_target:
        assert desc.y == dp.vector_domain(dp.atom_domain(T=int))
        assert desc.y.size is None
    else:
        assert desc.y is None
    for size in (0, 1, 5):
        X = sparse.csr_array(np.ones((size, 3), dtype=bool))
        targets = [0] * size if with_target else None
        assert domain.member(dp.sklearn.SklearnFit(X, targets))
        if with_target:
            assert not domain.member(dp.sklearn.SklearnFit(X, [0] * (size + 1)))
    assert context.query()._chain == (domain, dp.symmetric_distance())
    assert context.current_privacy_loss() == []


def test_csr_array2_domain_normalizes_atom_type():
    assert csr_array2_domain(3, max_rows=None, T=bool) == csr_array2_domain(
        3, max_rows=None, T="bool"
    )
    assert csr_array2_domain(3, max_rows=None, T=np.float32) != csr_array2_domain(
        3, max_rows=None, T=np.float64
    )
