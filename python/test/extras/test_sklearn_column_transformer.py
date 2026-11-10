"""Privacy-aware ColumnTransformer tests.

These test the real OpenDP constructors, not an independently simulated
privacy accountant. Polars-only cases are skipped when Polars is not installed.
"""

import numpy as np
import pytest

pytest.importorskip("sklearn")
from sklearn.base import BaseEstimator, clone
from sklearn.preprocessing import StandardScaler

import opendp.prelude as dp
from opendp._internal import _make_transformation
from opendp.extras.sklearn._estimator import DPEstimator, DPTransformer

dp.enable_features("contrib")


def _numeric_features(input_domain):
    features = input_domain.cast(dp.sklearn.SklearnFitDescriptor).X
    if isinstance(features, dp.DataFrameDomain):
        return dp.numpy.make_dataframe_to_numpy(features, dp.symmetric_distance())
    return None


class _ToyDPModel(DPTransformer):
    """Constant private parameters with certified NumPy rowwise inference."""

    def __init__(self, kind="label", value=0.5):
        self.kind = kind
        self.value = value

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit(
            input_domain, input_metric, as_numpy=True,
        )
        desc = numeric.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
        if desc.X.descriptor.num_columns is None:
            raise ValueError("Expected a fixed public feature width")
        value = float(self.value)
        measurement = dp.m.make_user_measurement(
            *numeric.output_space, output_measure,
            lambda data: value,
            lambda d: d_out * d / numeric.map(d_in),
            TO=float,
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        self.value_ = float(release)

    def make_transform(self, input_domain, input_metric):
        self.value_
        return _make_transformation(
            input_domain, input_metric, input_domain, input_metric,
            lambda data: data, lambda d: d,
        )

    def _make_rowwise_method(self, method, input_domain):
        if method not in {"predict", "transform"}:
            raise ValueError("Only certified methods may be used")
        return self.make_rowwise(input_domain)

    def make_rowwise(self, input_domain):
        from opendp.extras.sklearn._frozen import _RowwiseSpec

        value = self.value_
        numeric = _numeric_features(input_domain)

        def array(x):
            return numeric(x) if numeric is not None else x

        if self.kind == "label":
            return _RowwiseSpec(
                lambda x: (array(x)[:, :1] >= value).astype(np.int32),
                dp.numpy.array2_domain(T=dp.i32, num_columns=1),
            )
        if self.kind == "distance":
            return _RowwiseSpec(
                lambda x: np.minimum(np.abs(array(x)[:, :1] - value), 1.0),
                dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True),
            )
        raise ValueError("Unknown public kind")


class _StableRowwise(DPEstimator):
    """A test-only estimator whose rowwise operations need no fitting."""

    def _has_private_fit(self):
        return False

    def make(self, *args, **kwargs):
        raise NotImplementedError

    def _ingest_release(self, release):
        raise NotImplementedError


class _FrozenRowwise(_StableRowwise):
    def __init__(self, scale=2.0):
        self.scale = scale

    def make_rowwise(self, input_domain):
        from opendp.extras.sklearn._frozen import _RowwiseSpec

        scale = float(self.scale)
        numeric = _numeric_features(input_domain)
        return _RowwiseSpec(
            lambda x: scale * (numeric(x) if numeric is not None else x)[:, :1],
            dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True),
        )


def _context():
    X = np.array([
        [0.2, 2.0, 10.0],
        [0.8, 3.0, 20.0],
        [0.1, 4.0, 30.0],
    ])
    original = dp.sklearn.SklearnFit(X, [1, 0, 1])
    context = dp.Context.compositor(
        original,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2.0),
        split_evenly_over=1,
    )
    return context, X


def test_column_transformer_private_fit_and_hstack():
    context, X = _context()
    source = dp.sklearn.ColumnTransformer([
        ("cluster", _ToyDPModel("label"), [0, 1]),
        ("distances", _ToyDPModel("distance"), [0, 1]),
        ("original", "passthrough", [2]),
    ])
    fitted = context.query().sklearn(source).release()
    assert fitted is not source
    assert context.current_privacy_loss() == [2.0]
    assert not hasattr(source.transformers[0][1], "value_")
    assert not hasattr(source.transformers[1][1], "value_")
    assert fitted.named_transformers_["cluster"].value_ == 0.5
    assert fitted.named_transformers_["distances"].value_ == 0.5
    expected = np.column_stack([
        (X[:, 0] >= 0.5).astype(np.int32),
        np.abs(X[:, 0] - 0.5),
        X[:, 2],
    ])
    np.testing.assert_array_equal(fitted.transform(X), expected)
    assert fitted.get_feature_names_out().tolist() == [
        "cluster__x0", "distances__x0", "original__x2",
    ]
    trans = fitted.make_transform(
        context.accountant.input_domain, dp.symmetric_distance()
    )
    assert trans.map(1) == 1
    out = trans(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    desc = trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(desc.X.cast(dp.numpy.NPArray2Domain), dp.numpy.NPArray2Domain)
    assert desc.X.descriptor.num_columns == 3
    assert str(desc.X.descriptor.T) == "f64"
    assert desc.X.descriptor.nan is False
    np.testing.assert_array_equal(out.y, [1, 0, 1])
    np.testing.assert_array_equal(out.X, expected)
    assert out.X.dtype == np.float64
    assert trans.output_domain.member(out)


def test_polars_output_mixed_types_and_y():
    pl = pytest.importorskip("polars")
    context, X = _context()
    fitted = context.query().sklearn(
        dp.sklearn.ColumnTransformer([
            ("cluster", _ToyDPModel("label"), [0]),
            ("distances", _ToyDPModel("distance"), [0]),
        ]).set_output(transform="polars")
    ).release()
    assert clone(fitted)._dp_output == "polars"
    trans = fitted.make_transform(
        context.accountant.input_domain, dp.symmetric_distance()
    )
    descriptor = trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(descriptor.X, dp.DataFrameDomain)
    assert descriptor.X.columns == ["cluster__x0", "distances__x0"]
    output = trans(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    assert isinstance(output, dp.sklearn.SklearnFit)
    assert isinstance(output.X, pl.DataFrame)
    assert output.X.get_column("cluster__x0").dtype == pl.Int32
    assert output.X.get_column("distances__x0").dtype == pl.Float64
    assert output.y == [1, 0, 1]
    assert trans.output_domain.member(output)
    public = fitted.transform(X)
    assert isinstance(public, pl.DataFrame)
    assert public.dtypes == [pl.Int32, pl.Float64]
    # Switching output formats does not change the fitted model.
    fitted.set_output(transform="default")
    numeric = fitted.transform(X)
    assert isinstance(numeric, np.ndarray)
    assert numeric.dtype == np.float64


def test_named_dataframe_output_domain_is_supported():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn._frozen import _RowwiseSpec

    class Named(_StableRowwise):
        def make_rowwise(self, input_domain):
            return _RowwiseSpec(
                lambda x: pl.DataFrame({
                    "score": pl.Series(x[:, 0].astype(np.float64), dtype=pl.Float64),
                }),
                dp.dataframe_domain([
                    dp.series_domain(
                        "score", dp.atom_domain(T=dp.f64, nan=True),
                    ),
                ]),
            )

    ctx, X = _context()
    model = dp.sklearn.ColumnTransformer([
        ("named", Named(), [0]),
    ]).set_output(transform="polars")
    trans = model.make_transform(ctx.accountant.input_domain, dp.symmetric_distance())
    output = trans(dp.sklearn.SklearnFit(X))
    assert output.X.columns == ["named__score"]
    np.testing.assert_allclose(output.X["named__score"].to_numpy(), X[:, 0])
    assert trans.output_domain.member(output)


def test_unsupported_rowwise_output_domain_is_rejected():
    from opendp.extras.sklearn._frozen import _RowwiseSpec

    class VectorOutput(_StableRowwise):
        def make_rowwise(self, input_domain):
            return _RowwiseSpec(
                lambda x: x[:, 0],
                dp.vector_domain(dp.atom_domain(T=dp.f64)),
            )

    ctx, _ = _context()
    with pytest.raises(TypeError, match="output must be"):
        dp.sklearn.ColumnTransformer([
            ("unsupported", VectorOutput(), [0]),
        ]).make_transform(ctx.accountant.input_domain, dp.symmetric_distance())


def test_public_unfitted_rowwise_composition_accepts_named_polars():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame({"age": [1.0, 2.0]})
    model = dp.sklearn.ColumnTransformer([
        ("scaled", _FrozenRowwise(scale=2.0), ["age"]),
    ])
    np.testing.assert_array_equal(model.transform(frame), [[2.0], [4.0]])


def test_transform_only_composition_without_private_fit():
    context, X = _context()
    composite = dp.sklearn.ColumnTransformer([
        ("a", _FrozenRowwise(scale=2.0), [0]),
        ("b", _FrozenRowwise(scale=3.0), [1]),
    ])
    trans = composite.make_transform(
        context.accountant.input_domain, dp.symmetric_distance()
    )
    assert trans.map(1) == 1
    actual = trans(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    np.testing.assert_allclose(actual.X, np.column_stack([X[:, 0] * 2, X[:, 1] * 3]))
    assert actual.y == [1, 0, 1]
    assert context.current_privacy_loss() == []


def test_privacy_weights_and_clone_on_release():
    context, X = _context()
    source = dp.sklearn.ColumnTransformer([
        ("cluster", _ToyDPModel("label"), [0]),
        ("distances", _ToyDPModel("distance"), [0]),
    ], privacy_weights={"cluster": 1, "distances": 3})
    fitted = context.query().sklearn(source).release()
    assert fitted.privacy_weights == {"cluster": 1, "distances": 3}
    assert source.transformers[0][1].value == 0.5
    assert not hasattr(source.transformers[0][1], "value_")
    assert fitted.named_transformers_["distances"].value_ == 0.5


def test_reject_uncertified_transformers():
    context, X = _context()
    unsafe = dp.sklearn.ColumnTransformer([
        ("scale", StandardScaler(), [0, 1]),
    ])
    with pytest.raises((TypeError, ValueError)):
        context.query().sklearn(unsafe)

    with pytest.raises((TypeError, ValueError)):
        dp.sklearn.ColumnTransformer([
            ("scale", _FrozenRowwise(), lambda X: [0]),
        ]).make_transform(context.accountant.input_domain, dp.symmetric_distance())

    class _UntrustedDuck(BaseEstimator):
        def make_rowwise(self, input_domain):
            from opendp.extras.sklearn._frozen import _RowwiseSpec
            return _RowwiseSpec(lambda X: X, input_domain)

    with pytest.raises(TypeError, match="not certified rowwise"):
        dp.sklearn.ColumnTransformer([
            ("duck", _UntrustedDuck(), [0]),
        ]).make_transform(
            context.accountant.input_domain, dp.symmetric_distance(),
        )

    class _Uncertified(DPTransformer):
        def __init__(self):
            pass

        def make(self, domain, metric, measure, d_in, d_out):
            raise NotImplementedError

        def _ingest_release(self, release):
            pass

        def make_transform(self, domain, metric):
            return _make_transformation(
                domain, metric, domain, metric, lambda data: data, lambda d: d
            )

    uncertified = dp.sklearn.ColumnTransformer([
        ("bad", _Uncertified(), [0]),
    ])
    with pytest.raises(TypeError, match="not certified rowwise"):
        # The wrapper deliberately refuses to infer row alignment from a
        # vanilla DPTransformer.make_transform.
        uncertified.make_transform(
            context.accountant.input_domain, dp.symmetric_distance()
        )

    with pytest.raises(TypeError, match="not certified rowwise"):
        context.query().sklearn(uncertified)
    assert context.current_privacy_loss() == []



def test_no_accidental_fit_transform_without_privacy_budget():
    model = dp.sklearn.ColumnTransformer([
        ("label", _ToyDPModel("label"), [0]),
    ])
    with pytest.raises(NotImplementedError):
        model.fit_transform(np.ones((3, 2)))
    with pytest.raises(TypeError):
        model.fit(np.ones((3, 2)))


def test_remainder_passthrough_and_name_collisions():
    context, X = _context()
    fitted = context.query().sklearn(
        dp.sklearn.ColumnTransformer([
            ("selected", _ToyDPModel("label"), [0]),
        ], remainder="passthrough")
    ).release()
    got = fitted.transform(X)
    assert got.shape == (3, 3)
    assert fitted.get_feature_names_out().tolist() == [
        "selected__x0", "remainder__x1", "remainder__x2"
    ]
    with pytest.raises(ValueError, match="reserved"):
        dp.sklearn.ColumnTransformer([
            ("remainder", _ToyDPModel("label"), [0]),
        ], remainder="passthrough").make(
            context.accountant.input_domain, dp.symmetric_distance(),
            dp.max_divergence(), 1, 1.0
        )

def test_heterogeneous_dataframe_selectors_and_numeric_child():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame({
        "age": pl.Series([0.2, 0.8], dtype=pl.Float64),
        "income": pl.Series([1.0, 2.0], dtype=pl.Float64),
        "region": pl.Series(["E", "W"], dtype=pl.String),
        "target": pl.Series([0, 1], dtype=pl.Int32),
    })
    schema = dp.dataframe_domain([
        dp.series_domain(name, dp.atom_domain(T=dtype, nan=False) if dtype == dp.f64
                         else dp.atom_domain(T=dtype))
        for name, dtype in (
            ("age", dp.f64), ("income", dp.f64),
            ("region", str), ("target", dp.i32),
        )
    ])
    ctx = dp.Context.compositor(
        frame, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=schema, split_evenly_over=1,
    )
    model = dp.sklearn.ColumnTransformer([
        ("cluster", _ToyDPModel("label"), ["age", "income"]),
        ("region", "passthrough", ["region"]),
    ]).set_output(transform="polars")
    fitted = ctx.query().sklearn(model, y="target").release()
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain([
            dp.series_domain("age", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("income", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("region", dp.atom_domain(T=str)),
        ]),
        dp.series_domain("target", dp.atom_domain(T=dp.i32)),
    )
    trans = fitted.make_transform(domain, dp.symmetric_distance())
    target = frame.get_column("target")
    output = trans(dp.sklearn.SklearnFit(frame.drop("target"), target))
    assert isinstance(output, dp.sklearn.SklearnFit)
    assert isinstance(output.X, pl.DataFrame)
    assert output.X.columns == ["cluster__x0", "region__region"]
    assert output.X.get_column("region__region").to_list() == ["E", "W"]
    assert output.X.dtypes == [pl.Int32, pl.String]
    assert output.y.name == "target"
    assert output.y is target
    public = fitted.transform(frame.drop("target"))
    assert public.columns == ["cluster__x0", "region__region"]
    assert public["region__region"].to_list() == ["E", "W"]

    assert ctx.current_privacy_loss() == [1.0]


def test_frozen_and_private_branches_can_mix():
    ctx, X = _context()
    trained = ctx.query().sklearn(
        dp.sklearn.ColumnTransformer([
            ("cluster", _ToyDPModel("label"), [0]),
        ])
    ).release().named_transformers_["cluster"]
    frozen = trained.as_transformer("predict")

    other, X = _context()
    combined = dp.sklearn.ColumnTransformer([
        ("old", frozen, [0]),
        ("new", _ToyDPModel("distance"), [0]),
        ("original", "passthrough", [1]),
    ])
    fitted = other.query().sklearn(combined).release()
    assert fitted.named_transformers_["old"] is frozen
    result = fitted.transform(X)
    assert isinstance(result, np.ndarray)
    assert result.shape == (3, 3)
    assert other.current_privacy_loss() == [2.0]


def test_all_frozen_column_transformer_is_stable_query():
    ctx, X = _context()
    frozen = dp.sklearn.ColumnTransformer([
        ("left", _FrozenRowwise(scale=2.0), [0]),
        ("right", _FrozenRowwise(scale=3.0), [1]),
    ])
    assert frozen._has_private_fit() is False
    query = ctx.query().sklearn(frozen)
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == []

def test_polars_feature_named_y_does_not_collide_with_target():
    pl = pytest.importorskip("polars")
    domain = dp.dataframe_domain([
        dp.series_domain("y", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("response", dp.atom_domain(T=dp.i32)),
    ])
    frame = pl.DataFrame({
        "y": pl.Series([1.0, 2.0], dtype=pl.Float64),
        "response": pl.Series([0, 1], dtype=pl.Int32),
    })
    source = dp.sklearn.make_sklearn_fit(
        domain, dp.symmetric_distance(), y="response", as_numpy=False,
    )
    transformer = dp.sklearn.ColumnTransformer([
        ("keep", "passthrough", ["y"]),
    ]).set_output(transform="polars")
    transformed = transformer.make_transform(*source.output_space)
    result = transformed(source(frame))
    assert result.X.columns == ["keep__y"]
    assert result.y.name == "response"
    assert result.y.to_list() == [0, 1]
    assert transformed.output_domain.member(result)



def test_nested_rowwise_column_transformers():
    ctx, X = _context()
    inner = dp.sklearn.ColumnTransformer([
        ("first", "passthrough", [0]),
    ])
    outer = dp.sklearn.ColumnTransformer([
        ("nested", inner, [0, 1]),
        ("last", "passthrough", [2]),
    ])
    input_domain = ctx.accountant.input_domain
    spec = outer.make_rowwise(input_domain)
    np.testing.assert_array_equal(spec.function(X), X[:, [0, 2]])
    trans = outer.make_transform(input_domain, dp.symmetric_distance())
    data = dp.sklearn.SklearnFit(X, [1, 0, 1])
    result = trans(data)
    np.testing.assert_array_equal(result.X, X[:, [0, 2]])
    np.testing.assert_array_equal(result.y, data.y)
    assert trans.output_domain.member(result)
