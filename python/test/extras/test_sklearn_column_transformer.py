"""Privacy-aware ColumnTransformer tests.

These test the real OpenDP constructors, not an independently simulated
privacy accountant. Polars-only cases are skipped when Polars is not installed.
"""

import numpy as np
import pytest

pytest.importorskip("sklearn")
from sklearn.base import (
    BaseEstimator,
    ClassNamePrefixFeaturesOutMixin,
    OneToOneFeatureMixin,
    clone,
)
from sklearn.frozen import FrozenEstimator
from sklearn.preprocessing import StandardScaler

import opendp.prelude as dp
from opendp.mod import OpenDPException
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin

dp.enable_features("contrib")


def _numeric_features(features):
    if isinstance(features, dp.DataFrameDomain):
        return dp.numpy.make_dataframe_to_numpy(features, dp.symmetric_distance())
    return None


@pytest.fixture(autouse=True)
def allow_test_rowwise_callbacks():
    from opendp.mod import GLOBAL_FEATURES

    enabled = "honest-but-curious" in GLOBAL_FEATURES
    dp.enable_features("honest-but-curious")
    yield
    if not enabled:
        dp.disable_features("honest-but-curious")


class _ToyDPModel(MeasurementMixin, TransformationMixin, BaseEstimator):
    """Constant private parameters with declared NumPy rowwise inference."""

    def __init__(self, kind="label", value=0.5):
        self.kind = kind
        self.value = value

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit_numpy(input_domain, input_metric)
        desc = numeric.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
        if desc.X.descriptor.num_columns is None:
            raise ValueError("Expected a fixed public feature width")
        value = float(self.value)
        measurement = dp.m.make_user_measurement(
            numeric.output_domain,
            numeric.output_metric,
            output_measure,
            lambda data: value,
            lambda d: d_out * d / numeric.map(d_in),
            TO=float,
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        self.value_ = float(release)

    def transform(self, X):
        values = np.asarray(X)
        if self.kind == "label":
            return (values[:, :1] >= self.value_).astype(np.int32)
        if self.kind == "distance":
            return np.minimum(np.abs(values[:, :1] - self.value_), 1.0)
        raise ValueError("Unknown public kind")

    def get_feature_names_out(self, input_features=None):
        self.value_
        return np.asarray([self.kind], dtype=object)

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "transform":
            raise ValueError("Only declared rowwise methods may be used")
        self.value_
        _numeric_features(input_domain)
        if self.kind == "label":
            return dp.numpy.array2_domain(T=dp.i32, num_columns=1)
        if self.kind == "distance":
            return dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True)
        raise ValueError("Unknown public kind")


class _StableRowwise(TransformationMixin, BaseEstimator):
    """A test-only rowwise stable stage without private fitting."""


class _FrozenRowwise(_StableRowwise):
    def __init__(self, scale=2.0):
        self.scale = scale

    def transform(self, X):
        return float(self.scale) * np.asarray(X)[:, :1]

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "transform":
            raise ValueError(f"Unsupported method {method!r}")
        _numeric_features(input_domain)
        return dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True)


def _context():
    X = np.array(
        [
            [0.2, 2.0, 10.0],
            [0.8, 3.0, 20.0],
            [0.1, 4.0, 30.0],
        ]
    )
    original = dp.sklearn.SklearnFit(X, [1, 0, 1])
    context = dp.Context.compositor(
        original,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2.0),
        split_evenly_over=1,
    )
    return context, X


def _rowwise(model, domain):
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    return _make_rowwise_transform(domain, dp.symmetric_distance(), model)


def test_column_transformer_children_need_trusted_inference():
    context, X = _context()
    supported = dp.sklearn.ColumnTransformer([("keep", "passthrough", [0])]).fit(X)
    unchecked = dp.sklearn.ColumnTransformer(
        [("untrusted", _FrozenRowwise(), [0])]
    ).fit(X)
    dp.disable_features("honest-but-curious")
    try:
        assert _rowwise(supported, context.accountant.input_domain).map(1) == 1
        with pytest.raises(OpenDPException, match="honest-but-curious"):
            _rowwise(unchecked, context.accountant.input_domain)
    finally:
        dp.enable_features("honest-but-curious")


def test_column_transformer_private_fit_and_hstack():
    context, X = _context()
    source = dp.sklearn.ColumnTransformer(
        [
            ("cluster", _ToyDPModel("label"), [0, 1]),
            ("distances", _ToyDPModel("distance"), [0, 1]),
            ("original", "passthrough", [2]),
        ]
    )
    fitted = context.query().sklearn(source).release()
    assert fitted is not source
    assert context.current_privacy_loss() == [2.0]
    assert not hasattr(source.transformers[0][1], "value_")
    assert not hasattr(source.transformers[1][1], "value_")
    assert fitted.named_transformers_["cluster"].value_ == 0.5
    assert fitted.named_transformers_["distances"].value_ == 0.5
    expected = np.column_stack(
        [
            (X[:, 0] >= 0.5).astype(np.int32),
            np.abs(X[:, 0] - 0.5),
            X[:, 2],
        ]
    )
    np.testing.assert_array_equal(fitted.transform(X), expected)
    assert fitted.get_feature_names_out().tolist() == [
        "cluster__label",
        "distances__distance",
        "original__x2",
    ]
    trans = _rowwise(fitted, context.accountant.input_domain)
    assert trans.map(1) == 1
    out = trans(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    desc = trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(desc.X.cast(dp.numpy.NPArray2Domain), dp.numpy.NPArray2Domain)
    assert desc.X.descriptor.num_columns == 3
    assert str(desc.X.descriptor.T) == "f64"
    # A branch declares NaN-capable output, so the combined domain must allow it.
    assert desc.X.descriptor.nan is True
    np.testing.assert_array_equal(out.y, [1, 0, 1])
    np.testing.assert_array_equal(out.X, expected)
    assert out.X.dtype == np.float64
    assert trans.output_domain.member(out)


def test_polars_output_mixed_types_and_y():
    pl = pytest.importorskip("polars")
    context, X = _context()
    fitted = (
        context.query()
        .sklearn(
            dp.sklearn.ColumnTransformer(
                [
                    ("cluster", _ToyDPModel("label"), [0]),
                    ("distances", _ToyDPModel("distance"), [0]),
                ]
            ).set_output(transform="polars")
        )
        .release()
    )
    assert clone(fitted)._dp_output == "polars"
    trans = _rowwise(fitted, context.accountant.input_domain)
    descriptor = trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(descriptor.X, dp.DataFrameDomain)
    assert descriptor.X.columns == fitted.get_feature_names_out().tolist()
    output = trans(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    assert isinstance(output, dp.sklearn.SklearnFit)
    assert isinstance(output.X, pl.DataFrame)
    assert output.X.columns == descriptor.X.columns
    assert output.X.get_column("cluster__label").dtype == pl.Int32
    assert output.X.get_column("distances__distance").dtype == pl.Float64
    assert output.y == [1, 0, 1]
    public = fitted.transform(X)
    assert isinstance(public, pl.DataFrame)
    assert public.dtypes == [pl.Int32, pl.Float64]
    # Switching output formats does not change the fitted model.
    fitted.set_output(transform="default")
    numeric = fitted.transform(X)
    assert isinstance(numeric, np.ndarray)
    assert numeric.dtype == np.float64


def test_privacy_weights_and_clone_on_release():
    budgets = []

    class BudgetedModel(_ToyDPModel):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            budgets.append(d_out)
            return super().make_measurement(domain, metric, measure, d_in, d_out)

    context, X = _context()
    source = dp.sklearn.ColumnTransformer(
        [
            ("cluster", BudgetedModel("label"), [0]),
            ("distances", BudgetedModel("distance"), [0]),
        ],
        privacy_weights={"cluster": 1, "distances": 3},
    )
    fitted = context.query().sklearn(source).release()
    assert fitted.privacy_weights == {"cluster": 1, "distances": 3}
    assert budgets == pytest.approx([0.5, 1.5])
    assert fitted.named_transformers_["distances"].value_ == 0.5


def test_reject_untrusted_transformers_before_accounting():
    class Duck(BaseEstimator):
        def transform(self, X):
            return X

        def rowwise_output_domain(self, domain, *, method):
            return domain

    context, _ = _context()
    features = context.accountant.input_domain.cast(dp.sklearn.SklearnFitDescriptor).X
    for step in (StandardScaler(), Duck()):
        model = dp.sklearn.ColumnTransformer([("unsafe", step, [0])])
        with pytest.raises(TypeError, match="rowwise transform"):
            model.rowwise_output_domain(features, method="transform")
        with pytest.raises(TypeError, match="rowwise transform"):
            context.query().sklearn(model)
    assert context.current_privacy_loss() == []


def test_private_branches_require_rowwise_methods_before_accounting():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    class NoRowwise(_ToyDPModel):
        rowwise_output_domain = None  # type: ignore[assignment]

    context, _ = _context()
    for stage in (NoRowwise(), LogisticRegression()):
        model = dp.sklearn.ColumnTransformer([("private", stage, [0])])
        with pytest.raises(TypeError, match="rowwise transform"):
            context.query().sklearn(model)
    assert context.current_privacy_loss() == []


def test_private_branch_with_invalid_rowwise_contract_fails_after_accounting():
    class InvalidRowwise(_ToyDPModel):
        def rowwise_output_domain(self, input_domain, *, method):
            self.value_  # Requires the private fitting release.
            raise ValueError("Invalid fitted rowwise contract")

    context, _ = _context()
    model = dp.sklearn.ColumnTransformer(
        [
            ("invalid_rowwise", InvalidRowwise(), [0]),
        ]
    )
    query = context.query().sklearn(model)
    assert context.current_privacy_loss() == []
    with pytest.raises(OpenDPException, match="Invalid fitted rowwise contract"):
        query.release()

    # Rust reserved the only privacy allowance before executing the failing
    # fitted branch. Python's reporting of failed calls is fixed separately.
    with pytest.raises(OpenDPException):
        context.query().sklearn(
            dp.sklearn.ColumnTransformer([("another", _ToyDPModel(), [0])])
        ).release()


def test_remainder_passthrough_and_name_collisions():
    context, X = _context()
    fitted = (
        context.query()
        .sklearn(
            dp.sklearn.ColumnTransformer(
                [
                    ("selected", _ToyDPModel("label"), [0]),
                ],
                remainder="passthrough",
            )
        )
        .release()
    )
    got = fitted.transform(X)
    assert got.shape == (3, 3)
    assert fitted.get_feature_names_out().tolist() == [
        "selected__label",
        "remainder__x1",
        "remainder__x2",
    ]
    assert fitted.transformers_[-1] == ("remainder", "passthrough", (1, 2))
    assert fitted.named_transformers_["remainder"] == "passthrough"
    with pytest.raises(ValueError, match="conflict"):
        dp.sklearn.ColumnTransformer(
            [
                ("remainder", _ToyDPModel("label"), [0]),
            ],
            remainder="passthrough",
        ).make_measurement(
            context.accountant.input_domain,
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )


def test_heterogeneous_dataframe_selectors_and_numeric_child():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame(
        {
            "y": pl.Series([0.2, 0.8], dtype=pl.Float64),
            "income": pl.Series([1.0, 2.0], dtype=pl.Float64),
            "region": pl.Series(["E", "W"], dtype=pl.String),
            "response": pl.Series([0, 1], dtype=pl.Int32),
        }
    )
    schema = dp.dataframe_domain(
        [
            dp.series_domain(
                name,
                dp.atom_domain(T=dtype, nan=False)
                if dtype == dp.f64
                else dp.atom_domain(T=dtype),
            )
            for name, dtype in (
                ("y", dp.f64),
                ("income", dp.f64),
                ("region", str),
                ("response", dp.i32),
            )
        ]
    )
    ctx = dp.Context.compositor(
        frame,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=schema,
        split_evenly_over=1,
    )
    model = dp.sklearn.ColumnTransformer(
        [
            ("cluster", _ToyDPModel("label"), ["y", "income"]),
            ("region", "passthrough", ["region"]),
        ]
    ).set_output(transform="polars")
    fitted = ctx.query().sklearn(model, y="response").release()
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain(
            [
                dp.series_domain("y", dp.atom_domain(T=dp.f64, nan=False)),
                dp.series_domain("income", dp.atom_domain(T=dp.f64, nan=False)),
                dp.series_domain("region", dp.atom_domain(T=str)),
            ]
        ),
        dp.series_domain("response", dp.atom_domain(T=dp.i32)),
    )
    trans = _rowwise(fitted, domain)
    target = frame.get_column("response")
    output = trans(dp.sklearn.SklearnFit(frame.drop("response"), target))
    assert isinstance(output, dp.sklearn.SklearnFit)
    assert isinstance(output.X, pl.DataFrame)
    assert output.X.columns == fitted.get_feature_names_out().tolist()
    assert output.X.get_column("region__region").to_list() == ["E", "W"]
    assert output.X.dtypes == [pl.Int32, pl.String]
    assert output.y is not None
    assert output.y.name == "response"
    assert output.y is target
    public = fitted.transform(frame.drop("response"))
    assert public.columns == ["cluster__label", "region__region"]
    assert public["region__region"].to_list() == ["E", "W"]

    assert ctx.current_privacy_loss() == [1.0]


def test_frozen_and_private_branches_can_mix():
    ctx, X = _context()
    trained = (
        ctx.query()
        .sklearn(
            dp.sklearn.ColumnTransformer(
                [
                    ("cluster", _ToyDPModel("label"), [0]),
                ]
            )
        )
        .release()
        .named_transformers_["cluster"]
    )
    frozen = FrozenEstimator(trained)

    other, X = _context()
    combined = dp.sklearn.ColumnTransformer(
        [
            ("old", frozen, [0]),
            ("new", _ToyDPModel("distance"), [0]),
            ("original", "passthrough", [1]),
        ]
    )
    query = other.query().sklearn(combined)
    fitted = query.release()
    old = fitted.named_transformers_["old"]
    assert old is frozen
    assert old.estimator is trained
    assert old.estimator.value_ == 0.5
    result = fitted.transform(X)
    assert isinstance(result, np.ndarray)
    assert result.shape == (3, 3)
    assert other.current_privacy_loss() == [2.0]


def test_all_frozen_column_transformer_is_stable_query():
    ctx, X = _context()
    frozen = dp.sklearn.ColumnTransformer(
        [
            ("left", _FrozenRowwise(scale=2.0), [0]),
            ("right", _FrozenRowwise(scale=3.0), [1]),
        ]
    )
    assert frozen._has_private_fit() is False
    query = ctx.query().sklearn(frozen, method="transform")
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == []


def test_nested_rowwise_column_transformers():
    ctx, X = _context()
    inner = dp.sklearn.ColumnTransformer(
        [
            ("first", "passthrough", [0]),
        ]
    )
    outer = dp.sklearn.ColumnTransformer(
        [
            ("nested", inner, [0, 1]),
            ("last", "passthrough", [2]),
        ]
    )
    input_domain = ctx.accountant.input_domain
    output_domain = outer.rowwise_output_domain(
        input_domain.cast(dp.sklearn.SklearnFitDescriptor).X, method="transform"
    )
    assert output_domain.cast(dp.numpy.NPArray2Domain).num_columns == 2
    np.testing.assert_array_equal(outer.transform(X), X[:, [0, 2]])
    trans = _rowwise(outer, input_domain)
    data = dp.sklearn.SklearnFit(X, [1, 0, 1])
    result = trans(data)
    np.testing.assert_array_equal(result.X, X[:, [0, 2]])
    np.testing.assert_array_equal(result.y, data.y)
    assert trans.output_domain.member(result)
    empty = trans(dp.sklearn.SklearnFit(np.empty((0, 3)), []))
    assert empty.X.shape == (0, 2)
    assert trans.output_domain.member(empty)


def test_boolean_and_negative_selectors_agree_between_fit_and_inference():
    context, X = _context()
    model = dp.sklearn.ColumnTransformer(
        [
            ("selected", _ToyDPModel("label"), [True, False, True]),
            ("last", "passthrough", [-1]),
        ]
    )
    fitted = context.query().sklearn(model).release()
    transform = _rowwise(fitted, context.accountant.input_domain)
    output = transform(dp.sklearn.SklearnFit(X, [1, 0, 1]))
    expected = np.column_stack(
        [
            (X[:, 0] >= 0.5).astype(np.int32),
            X[:, -1],
        ]
    )
    np.testing.assert_array_equal(output.X, expected)
    np.testing.assert_array_equal(output.y, [1, 0, 1])
    assert fitted.get_feature_names_out().tolist() == [
        "selected__label",
        "last__x2",
    ]
    assert transform.output_domain.member(output)


def test_stable_only_refit_validates_fresh_configuration():
    X = np.ones((3, 2))
    model = dp.sklearn.ColumnTransformer(
        [
            ("stable", _FrozenRowwise(), [0]),
        ]
    ).fit(X)
    fitted = model.transformers_[0][1]

    # A failed refit must not validate stale fitted children or replace them.
    model.transformers = [("unsafe", StandardScaler(), [0])]
    with pytest.raises(TypeError, match="rowwise transform"):
        model.fit(X)
    assert model.transformers_[0][1] is fitted

    model.transformers = [("stable", _FrozenRowwise(scale=3.0), [1])]
    model.fit(X)
    np.testing.assert_array_equal(model.transform(X), 3 * X[:, 1:2])


def test_child_feature_names_use_sklearn_conventions():
    class Projection(ClassNamePrefixFeaturesOutMixin, _StableRowwise):
        @property
        def _n_features_out(self):
            return 1

        def transform(self, X):
            return np.asarray(X)[:, :1]

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64, num_columns=1)

    X = np.ones((2, 2))
    fitted = dp.sklearn.ColumnTransformer(
        [("project", Projection(), [0]), ("keep", "passthrough", [1])]
    ).fit(X)
    assert fitted.get_feature_names_out(["left", "right"]).tolist() == [
        "project__projection0",
        "keep__right",
    ]

    class NoNames(Projection):
        def get_feature_names_out(self, input_features=None):
            return None

    unnamed = dp.sklearn.ColumnTransformer([("unused", NoNames(), [0])]).fit(X)
    assert unnamed.get_feature_names_out().size == 0

    with pytest.raises(ValueError, match="Feature names must match the output width"):
        unnamed.set_output(transform="polars").rowwise_output_domain(
            dp.numpy.array2_domain(T=dp.f64, num_columns=2),
            method="transform",
        )

    class AlternateNames(Projection):
        def get_feature_names_out(self, input_features=None):
            return np.asarray(["advertised"], dtype=object)

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.dataframe_domain(
                [dp.series_domain("actual", dp.atom_domain(T=dp.f64))]
            )

        def transform(self, X):
            pl = pytest.importorskip("polars")
            return pl.DataFrame({"actual": np.asarray(X)[:, 0]})

    renamed = (
        dp.sklearn.ColumnTransformer([("branch", AlternateNames(), [0])])
        .set_output(transform="polars")
        .fit(X)
    )
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    )
    trans = _rowwise(renamed, domain)
    result = trans(dp.sklearn.SklearnFit(X))
    expected_names = renamed.get_feature_names_out().tolist()
    assert expected_names == ["branch__advertised"]
    assert (
        trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor).X.columns
        == expected_names
    )
    assert result.X.columns == expected_names
    assert trans.output_domain.member(result)
    assert renamed.transform(X).columns == expected_names


def test_feature_names_out_accepts_public_input_names():
    X = np.ones((2, 3))
    model = dp.sklearn.ColumnTransformer(
        [
            ("selected", "passthrough", [0, 2]),
            ("unused", "drop", [1]),
        ]
    ).fit(X)
    assert model.get_feature_names_out(["first", "second", "third"]).tolist() == [
        "selected__first",
        "selected__third",
    ]
    with pytest.raises(ValueError, match="length equal to number of features"):
        model.get_feature_names_out(["first"])

    pl = pytest.importorskip("polars")
    frame = pl.DataFrame({"age": [1.0, 2.0], "income": [2.0, 3.0]})
    named = (
        dp.sklearn.ColumnTransformer(
            [
                ("keep", "passthrough", ["age"]),
            ]
        )
        .set_output(transform="polars")
        .fit(frame)
    )
    assert named.get_feature_names_out(["age", "income"]).tolist() == ["keep__age"]
    with pytest.raises(ValueError, match="feature_names_in_"):
        named.get_feature_names_out(["income", "age"])

    class Identity(OneToOneFeatureMixin, _StableRowwise):
        def transform(self, X):
            return np.asarray(X)

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64, num_columns=2)

    one_to_one = dp.sklearn.ColumnTransformer(
        [("identity", Identity(), ["age", "income"])]
    ).fit(frame)
    assert one_to_one.get_feature_names_out().tolist() == [
        "identity__age",
        "identity__income",
    ]


def test_public_selector_rejections_and_supported_forms():
    from opendp.extras.sklearn.compose import _selector

    assert _selector("first", 2, names=["first", "second"]) == (0,)
    assert _selector(slice(1, 3), 3) == (1, 2)
    assert _selector(np.int64(1), 3) == (1,)
    with pytest.raises(ValueError, match="Unknown feature"):
        _selector("missing", 2, names=["first", "second"])
    with pytest.raises(ValueError, match="Boolean selector"):
        _selector([True], 2)
    with pytest.raises(TypeError, match="public integer"):
        _selector(["first"], 2)
    with pytest.raises(ValueError, match="out of bounds"):
        _selector([2], 2)
    with pytest.raises(ValueError, match="Repeated"):
        _selector([0, 0], 2)
    with pytest.raises(ValueError, match="Repeated"):
        _selector(["first", "first"], 2, names=["first", "second"])
    with pytest.raises(TypeError, match="public positional"):
        _selector(lambda X: [0], 2)


def test_public_domain_and_column_output_rejections():
    from opendp.extras.sklearn.compose import _input_features, _output_columns

    class VectorOutput(_StableRowwise):
        def transform(self, X):
            return np.asarray(X)[:, 0]

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.vector_domain(dp.atom_domain(T=dp.f64))

    features = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    with pytest.raises(TypeError, match="output must be"):
        dp.sklearn.ColumnTransformer(
            [("unsupported", VectorOutput(), [0])]
        ).rowwise_output_domain(features, method="transform")

    unknown_width = dp.numpy.array2_domain(T=dp.f64)
    with pytest.raises(ValueError, match="public number"):
        _input_features(unknown_width)
    with pytest.raises(ValueError, match="symmetric row distance"):
        dp.sklearn.ColumnTransformer([("keep", "passthrough", [0])]).make_measurement(
            _context()[0].accountant.input_domain,
            dp.discrete_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )
    with pytest.raises(ValueError, match="public output width"):
        _output_columns(unknown_width, "unknown")
    with pytest.raises(ValueError, match="at least one column"):
        _output_columns(dp.dataframe_domain([]), "empty")


def test_public_column_transformer_configuration_errors():
    _, X = _context()
    features = dp.numpy.array2_domain(T=dp.f64, num_columns=3)

    for branches, options, pattern in [
        ([("keep", "passthrough", [0])], {"remainder": "unknown"}, "remainder"),
        (
            [("keep", "passthrough", [0])],
            {"verbose_feature_names_out": "bad"},
            "verbose_feature_names_out",
        ),
        (
            [("same", "passthrough", [0]), ("same", "passthrough", [1])],
            {},
            "not unique",
        ),
        ([("invalid", "unknown", [0])], {}, "Unsupported transformer"),
        ([("a__b", "passthrough", [0])], {}, "contain __"),
        ([(0, "passthrough", [0])], {}, "names must be strings"),
        ([("transformers", "passthrough", [0])], {}, "conflict"),
        ([("empty", _FrozenRowwise(), [])], {}, "selects no input columns"),
    ]:
        model = dp.sklearn.ColumnTransformer(branches, **options)
        with pytest.raises((ValueError, TypeError), match=pattern):
            model.rowwise_output_domain(features, method="transform")

    stable = dp.sklearn.ColumnTransformer([("keep", "passthrough", [0])])
    with pytest.raises(ValueError, match="no rowwise method"):
        stable.rowwise_output_domain(features, method="predict")
    with pytest.raises(ValueError, match="Only 'default'"):
        stable.set_output(transform="unknown")
    with pytest.raises(Exception, match="not fitted"):
        stable.get_feature_names_out()
    with pytest.raises(ValueError, match="does not consume"):
        stable.fit(
            X,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
        )

    stable.fit(X)
    with pytest.raises(ValueError, match="fitted width"):
        stable.rowwise_output_domain(
            dp.numpy.array2_domain(T=dp.f64, num_columns=2),
            method="transform",
        )

    only_dropped = dp.sklearn.ColumnTransformer([("discard", "drop", [0])])
    with pytest.raises(ValueError, match="at least one column"):
        only_dropped.rowwise_output_domain(features, method="transform")

    colliding = dp.sklearn.ColumnTransformer(
        [("left", "passthrough", [0]), ("right", "passthrough", [0])],
        verbose_feature_names_out=False,
    )
    with pytest.raises(ValueError, match="column names must be unique"):
        colliding.rowwise_output_domain(features, method="transform")


def test_column_transformer_inference_output_validation():
    nullable = dp.dataframe_domain(
        [dp.series_domain("nullable", dp.option_domain(dp.atom_domain(T=dp.f64)))]
    )
    text = dp.dataframe_domain([dp.series_domain("text", dp.atom_domain(T=str))])
    model = dp.sklearn.ColumnTransformer([("keep", "passthrough", [0])])
    with pytest.raises(TypeError, match="Nullable output"):
        model.rowwise_output_domain(nullable, method="transform")
    with pytest.raises(TypeError, match="numeric columns"):
        model.rowwise_output_domain(text, method="transform")

    X = np.array([[1.0, 2.0], [3.0, 4.0]])
    with pytest.raises(ValueError, match="Call fit"):
        model.transform(X)
    model.fit(X)
    np.testing.assert_array_equal(model.transform(X), X[:, :1])

    class OneDimensional(_StableRowwise):
        def transform(self, X):
            return np.asarray(X)[:, 0]

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64, num_columns=1)

    one = dp.sklearn.ColumnTransformer([("one", OneDimensional(), [0])]).fit(X)
    np.testing.assert_array_equal(one.transform(X), X[:, :1])
    with pytest.raises(AttributeError, match="does not provide get_feature_names_out"):
        one.get_feature_names_out()


def test_branch_result_must_match_public_output_width():
    class BadWidth(_StableRowwise):
        def transform(self, X):
            return np.ones((len(X), 2))

        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64, num_columns=1)

    X = np.ones((2, 2))
    model = dp.sklearn.ColumnTransformer([("bad", BadWidth(), [0])]).fit(X)
    with pytest.raises(ValueError, match="output width"):
        model.transform(X)


def test_standalone_polars_transformer_preserves_nullable_schema():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame(
        {
            "optional": pl.Series([1.0, None], dtype=pl.Float64),
            "complete": pl.Series([2.0, 3.0], dtype=pl.Float64),
        }
    )
    model = dp.sklearn.ColumnTransformer(
        [
            ("optional", "passthrough", ["optional"]),
            ("complete", "passthrough", ["complete"]),
        ]
    ).set_output(transform="polars")
    model.fit(frame)
    result = model.transform(frame)
    assert result.columns == ["optional__optional", "complete__complete"]
    assert result["optional__optional"].to_list() == [1.0, None]
    assert result["complete__complete"].to_list() == [2.0, 3.0]


def test_public_only_estimator_and_composition_checks():
    class PublicOnly(MeasurementMixin, BaseEstimator):
        def _has_private_fit(self):
            return False

        def make_measurement(self, *args):
            raise AssertionError("Public-only stages must not fit")

        def _ingest_release(self, release):
            raise AssertionError("Public-only stages must not ingest")

    context, _ = _context()
    with pytest.raises(
        TypeError, match="Stable inference requires TransformationMixin"
    ):
        context.query().sklearn(PublicOnly())

    # Private fitting via the sklearn .fit entrypoint is itself budgeted.
    private = dp.sklearn.ColumnTransformer([("private", _ToyDPModel(), [0])])
    with pytest.raises(TypeError, match="Private fit requires"):
        private.fit(np.ones((2, 3)))
    _, X = _context()
    assert (
        private.fit(
            X,
            y=[1, 0, 1],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=2.0),
            domain=context.accountant.input_domain,
        )
        is private
    )
    assert private.named_transformers_["private"].value_ == 0.5


def test_numpy_runtime_does_not_import_polars(monkeypatch):
    import builtins

    original = builtins.__import__

    def no_polars(name, *args, **kwargs):
        if name == "polars" or name.startswith("polars."):
            raise ImportError("Polars is intentionally unavailable")
        return original(name, *args, **kwargs)

    model = dp.sklearn.ColumnTransformer(
        [
            ("keep", "passthrough", [0, 1]),
        ]
    )
    X = np.ones((3, 2))
    monkeypatch.setattr(builtins, "__import__", no_polars)
    model.fit(X)
    np.testing.assert_array_equal(model.transform(X), X)
