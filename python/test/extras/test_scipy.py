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


def test_csr_array2_domain_normalizes_atom_type():
    assert csr_array2_domain(3, max_rows=None, T=bool) == csr_array2_domain(
        3, max_rows=None, T="bool"
    )
    assert csr_array2_domain(3, max_rows=None, T=np.float32) != csr_array2_domain(
        3, max_rows=None, T=np.float64
    )
