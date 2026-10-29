import opendp.prelude as dp
from opendp.extras.numpy import then_np_clamp
import pytest

np = pytest.importorskip("numpy")


def test_clamp():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.random.default_rng(1).normal(size=(100_000, 10))

    row_norms = np.linalg.norm(data, axis=1, keepdims=True)
    expected = data / np.maximum(row_norms, 1.0)

    np.testing.assert_allclose(trans(data), expected)


@pytest.mark.parametrize("p", [1, 2])
def test_clamp_memory_layout_invariant(p):
    data = np.random.default_rng(1).normal(size=(4, 16))
    domain = dp.numpy.array2_domain(T=float, num_columns=16)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=p)

    expected = trans(data)
    for layout in (np.asfortranarray(data), np.repeat(data, 2, axis=1)[:, ::2]):
        np.testing.assert_array_equal(layout, data)
        np.testing.assert_array_equal(trans(layout), expected)


def test_clamp_nan_inf():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.array([[np.inf, 0.0], [1.0, 1.0], [np.nan, 0.0]])

    expected = np.array([[1.0, 0.0], [2**-0.5, 2**-0.5], [0.0, 0.0]])

    np.testing.assert_allclose(trans(data), expected)


@pytest.mark.parametrize("num_columns", [None, 2])
def test_clamp_zero_length(num_columns):
    domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = domain, dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    empty = np.zeros((0, 2))
    assert domain.member(empty)
    assert trans(empty).shape == (0, 2)


@pytest.mark.parametrize(
    "dtype,T",
    [("int32", dp.i32), ("float32", dp.f32), ("float64", dp.f64)],
)
def test_clamp_output_domain(dtype, T):
    data = np.array([[1, 1], [1, 0]], dtype=dtype)
    domain = dp.numpy.array2_domain(T=T, size=2, num_columns=2, cardinalities=[1, 2])
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2)

    result = trans(data)

    assert result.dtype == np.float64
    assert trans.output_domain.descriptor.T == dp.f64
    assert trans.output_domain.descriptor.cardinalities is None
    assert len(np.unique(result[:, 0])) == 2
    assert trans.output_domain.member(result)


def test_clamp_origin_validation_and_normalization():
    unknown_width = dp.numpy.array2_domain(T=float)
    with pytest.raises(ValueError, match="specify num_columns"):
        (unknown_width, dp.symmetric_distance()) >> then_np_clamp(
            norm=1.0, p=2, origin=np.array([1.0])
        )
    with pytest.raises(ValueError, match="scalar origin must be zero"):
        (unknown_width, dp.symmetric_distance()) >> then_np_clamp(
            norm=1.0, p=2, origin=1.0
        )

    domain = dp.numpy.array2_domain(T=float, num_columns=2)
    with pytest.raises(ValueError, match="scalar or one-dimensional"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(
            norm=1.0, p=2, origin=np.array([[1.0, 2.0]])
        )
    with pytest.raises(ValueError, match="origin must be finite"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2, origin=np.inf)
    with pytest.raises(ValueError, match="num_columns=2"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(
            norm=1.0, p=2, origin=np.array([1.0])
        )

    with pytest.raises(ValueError, match="scalar origin must be zero"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2, origin=5.0)

    vector = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=1.0, p=2, origin=[1.0, 2.0]
    )
    assert vector.output_domain.member(vector(np.array([[3.0, 4.0]])))

    for origin in [
        True,
        [True, False],
        ["1", "2"],
        [1 + 0j, 2 + 0j],
        [[1.0], [2.0, 3.0]],
    ]:
        with pytest.raises(ValueError, match="origin must be numeric"):
            (domain, dp.symmetric_distance()) >> then_np_clamp(
                norm=1.0, p=2, origin=origin
            )


@pytest.mark.parametrize("p", [True, 1.0, 2.0, 3])
def test_clamp_rejects_invalid_p(p):
    domain = dp.numpy.array2_domain(T=float, num_columns=1)
    with pytest.raises(ValueError, match="integer 1 or 2"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=p)


@pytest.mark.parametrize("norm", [True, 0.0, -1.0, np.inf, -np.inf, np.nan])
def test_clamp_rejects_invalid_norm(norm):
    domain = dp.numpy.array2_domain(T=float, num_columns=1)
    with pytest.raises(ValueError, match="norm"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=norm, p=1)


@pytest.mark.parametrize("norm", [np.nextafter(0.0, 1.0), 1e300])
def test_clamp_extreme_norms_do_not_warn(norm):
    import warnings

    data = np.array([[1.0], [0.0]], dtype=np.float32)
    domain = dp.numpy.array2_domain(T=dp.f32, num_columns=1)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=norm, p=1)

    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        result = trans(data)

    assert caught == []
    assert result.dtype == np.float64


def test_clamp_large_norm_domain_membership():
    # Computing squared magnitudes directly would overflow here.
    norm = 1e300
    domain = dp.numpy.array2_domain(T=float, norm=norm, p=2, num_columns=2)
    assert domain.member(np.array([[0.6e300, 0.6e300]]))
    with pytest.warns(UserWarning, match="row norm"):
        assert not domain.member(np.array([[norm, norm]]))


@pytest.mark.parametrize("p", [1, 2])
def test_clamp_extreme_norms_in_output_domain(p):
    tiny = np.nextafter(0.0, 1.0)
    max_float = np.finfo(float).max
    data = np.array(
        [
            [max_float, max_float],
            [-max_float, max_float],
            [tiny, tiny],
            [2 * tiny, 2 * tiny],
            [1e-300, -1e-300],
            [1e308, 1e308],
            [np.inf, 0.0],
            [np.nan, -np.inf],
        ]
    )
    domain = dp.numpy.array2_domain(T=float, num_columns=2)

    for norm in [tiny, 3 * tiny, 1e-200, 1.0, 1e300, max_float]:
        trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=norm, p=p)
        result = trans(data)
        assert np.isfinite(result).all()
        assert trans.output_domain.member(result)

    # A one-dimensional subnormal bound remains representable after clipping.
    scalar_domain = dp.numpy.array2_domain(T=float, num_columns=1)
    scalar_trans = (scalar_domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=tiny, p=p
    )
    assert scalar_trans(np.array([[1.0]]))[0, 0] == tiny


@pytest.mark.parametrize(
    "p,norm,origin,data",
    [
        (2, 1.0, [1.0, 1.0, 1.0], [[0.0, 0.0, 1e-5]]),
        (1, np.finfo(float).max, [1e308], [[np.inf]]),
        (2, 1.0, [1e10, -1e10], [[0.0, 0.0]]),
    ],
)
def test_clamp_extreme_origin_in_output_domain(p, norm, origin, data):
    domain = dp.numpy.array2_domain(T=float, num_columns=len(data[0]))
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=norm, p=p, origin=origin
    )
    result = trans(np.array(data))
    assert np.isfinite(result).all()
    assert trans.output_domain.member(result)


@pytest.mark.parametrize("T", [str, bool])
def test_clamp_rejects_non_numeric_domain(T):
    domain = dp.numpy.array2_domain(T=T, num_columns=1)
    with pytest.raises(ValueError, match="element type must be numeric"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2)


def test_clamp_rejects_non_symmetric_metric():
    domain = dp.numpy.array2_domain(T=float, num_columns=1)
    with pytest.raises(ValueError, match="symmetric distance"):
        (domain, dp.l1_distance(T=float)) >> then_np_clamp(norm=1.0, p=2)


@pytest.mark.parametrize(
    "origin,norm,data",
    [
        ([1e16], 1e16, 1.0),
        (0.0, 1.0, 0.5),
        ([1e308], np.finfo(float).max, 1.0),
    ],
)
def test_clamp_preserves_admissible_rows(origin, norm, data):
    domain = dp.numpy.array2_domain(T=float, num_columns=1)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=norm, p=1, origin=origin
    )
    value = np.array([[data]])
    assert trans.output_domain.member(value)
    np.testing.assert_array_equal(trans(value), value)


def test_clamp_snapshots_origin():
    origin = np.array([1.0, 2.0])
    domain = dp.numpy.array2_domain(T=float, num_columns=2)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=1.0, p=2, origin=origin
    )
    data = np.array([[3.0, 4.0]])
    expected = trans(data)

    origin[:] = np.nan  # Mutation after construction must not alter the clamp.
    stored = trans.output_domain.descriptor.origin
    np.testing.assert_array_equal(stored, [1.0, 2.0])
    np.testing.assert_array_equal(trans(data), expected)
    assert trans.output_domain.member(expected)


@pytest.mark.parametrize(
    "p,data",
    [
        (1, [9.001852141268024, 8.724114521437599]),
        (2, [9.359521548405217, 8.399783871312854]),
    ],
)
def test_clamp_nonzero_origin_avoids_rounding_fallback(p, data):
    origin = np.array([10.0, 10.0])
    domain = dp.numpy.array2_domain(T=float, num_columns=2)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=1.0, p=p, origin=origin
    )
    result = trans(np.array([data]))
    assert not np.array_equal(result[0], origin)
    assert trans.output_domain.member(result)


def test_clamp_exceptional_row_is_rowwise():
    origin = np.array([1e308])
    domain = dp.numpy.array2_domain(T=float, num_columns=1)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=np.finfo(float).max, p=1, origin=origin
    )
    admissible = np.array([[1e308]])
    exceptional = np.array([[np.inf]])
    result = trans(np.vstack([admissible, exceptional]))

    np.testing.assert_array_equal(result[:1], trans(admissible))
    assert result[1, 0] > origin[0]
    assert trans.output_domain.member(result)


@pytest.mark.parametrize("shape", [(10_000, 8), (8, 10_000)])
def test_clamp_large_arrays(shape):
    domain = dp.numpy.array2_domain(T=float, num_columns=shape[1])
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2)
    data = np.full(shape, 2.0)

    result = trans(data)

    assert result.shape == shape
    assert trans.output_domain.member(result)


def test_clamp_validation_errors_include_bad_value():
    domain = dp.numpy.array2_domain(T=float, num_columns=1)

    with pytest.raises(ValueError, match=r"norm must be a real number, got True"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=True, p=1)

    with pytest.raises(ValueError, match=r"order p must be the integer 1 or 2, got 3"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=3)

    with pytest.raises(ValueError, match=r"origin must be numeric, got 'bad'"):
        (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=1, origin="bad")
