
def test_polars_dataframe():
    pl = pytest.importorskip("polars")
    val_in = pl.DataFrame(
        {
            "A": [1] * 100,
            "B": ["X"] * 100,
            "C": [True] * 100,
        }
    )
    obj = py_to_c(val_in, AnyObjectPtr, "DataFrame")
    val_out = c_to_py(obj)
    assert val_out.equals(val_in)


def test_polars_expr():
    pl = pytest.importorskip("polars")
    val_in = pl.all()
    obj = py_to_c(val_in, AnyObjectPtr, "Expr")
    val_out = c_to_py(obj)
    assert str(val_out) == str(val_in)