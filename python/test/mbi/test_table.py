import re
from opendp.extras.mbi import Fixed, AIM, MST, ContingencyTable, Count, Sequential
from opendp.extras.mbi._table import _get_null_index, _increasing, _unique, _with_null
from opendp.extras.mbi._utilities import mirror_descent
import pytest
import opendp.prelude as dp
import warnings


@pytest.mark.parametrize(
    "privacy_loss", [{"epsilon": 5.0}, {"rho": 0.2}], ids=["DP", "zCDP"]
)
@pytest.mark.parametrize("approximate", [True, False], ids=["approximate", "pure"])
@pytest.mark.parametrize(
    "algorithm",
    [
        AIM(queries=2),
        MST(),
        Fixed(queries=[Count(("C",)), Count(("A", "B"))]),
        Sequential(algorithms=[Fixed(queries=[Count(("A", "B"))]), AIM()]),
    ],
    ids=["AIM", "MST", "Fixed", "Sequential"],
)
def test_fit_effectiveness(algorithm, privacy_loss, approximate):
    pytest.importorskip("mbi")
    # mutating `parameter` interferes with other pytest parameterizations
    privacy_loss = privacy_loss.copy()

    cuts = {"A": [-2, -1, 0, 1, 2]}
    keys: dict[str, list] = {"B": ["a", "b", "c", "d", "e", "f"]}
    if approximate:
        privacy_loss["delta"] = 1e-8
    else:
        keys["C"] = [0, 1, 2, 3, 4, 5, 6, 7]

    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    cov = np.array([[1.0, 0.75, 0.0], [0.75, 1.0, 0.0], [0.0, 0.0, 1.0]])

    data_np = np.random.multivariate_normal(mean=[0.0] * 3, cov=cov, size=10_000)

    categories_B = list("abcdef")
    edges_B = np.linspace(-2, 2, num=5)
    edges_C = np.linspace(-3, 3, num=7)
    data_dict = {
        # continuous float
        "A": data_np.T[0],
        # categorical string
        "B": np.array(categories_B)[np.digitize(data_np.T[1], bins=edges_B)],
        # categorical int
        "C": np.digitize(data_np.T[2], bins=edges_C),
    }
    data_lf = pl.LazyFrame(data_dict)
    lookup_B = {"a": -2.5, "b": -1.5, "c": -0.5, "d": 0.5, "e": 1.5, "f": 2.5, None: 0}
    lookup_C = dict(zip(range(8), np.linspace(-3.5, 3.5, 8)))
    lookup_C[None] = 0  # type: ignore[index]

    roundtrip_np = np.stack(
        [
            data_dict["A"],
            [lookup_B[i] for i in data_dict["B"]],
            [lookup_C[i] for i in data_dict["C"]],
        ]
    )
    assert np.linalg.norm(cov - np.cov(roundtrip_np)) < 0.5

    context = dp.Context.compositor(
        data=data_lf,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(**privacy_loss),
    )

    table: ContingencyTable = (
        context.query(**privacy_loss)
        .contingency_table(cuts=cuts, keys=keys, algorithm=algorithm)
        .release()
    )

    try:
        std = table.std("A")
        assert isinstance(std, float)
        assert 1 < std < 5
    except ValueError:
        pass

    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        synthetic_df = table.synthesize()

    synthetic_list = [
        synthetic_df["A"],
        [lookup_B[c] for c in synthetic_df["B"]],
        [lookup_C[c] for c in synthetic_df["C"]],
    ]
    synthetic_cov = np.cov(np.stack(synthetic_list))
    assert np.linalg.norm(cov - synthetic_cov) < 0.5, f"\n{synthetic_cov}"


def test_contingency_table_int_cuts():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    exact = [100, 200, 400, 300, 200]

    lm = LinearMeasurement(exact, clique=("A",), stddev=0.01)
    model = mirror_descent(Domain(("A",), (5,)), [lm])
    A_cuts = pl.Series("A", [0, 2, 4, 6])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a", "b", "c", "d", "e"])},
        cuts={"A": A_cuts},
        marginals={("A",): lm},
        model=model,
    )

    assert table.schema == {"A": pl.String}
    assert np.allclose(table.project(("A",)), exact)

    frequencies = table.synthesize()["A"].cut(A_cuts).value_counts()
    residuals = frequencies.sort("A")["count"] - pl.Series(exact)

    assert all(r == 0 for r in residuals)


def test_contingency_table_project():
    pytest.importorskip("mbi")
    from mbi import LinearMeasurement, Domain  # type: ignore[import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]
    from polars.testing import assert_frame_equal  # type: ignore[import-not-found]

    A_exact = [3, 5]
    B_exact = [1, 3, 4]
    A_keys = pl.Series("A", ["a1", "a2"])
    B_keys = pl.Series("B", ["b1", "b2", "b3"])

    A_lm = LinearMeasurement(A_exact, clique=("A",))
    B_lm = LinearMeasurement(B_exact, clique=("B",))

    model = mirror_descent(Domain(("A", "B"), (2, 3)), [A_lm, B_lm])
    table = ContingencyTable(
        keys={"A": A_keys, "B": B_keys},
        cuts={},
        marginals={("A",): A_lm, ("B",): B_lm},
        model=model,
    )

    assert np.allclose(table.project("A"), A_exact)
    assert np.allclose(table.project("B"), B_exact)

    AB_exact = [[0.375, 1.125, 1.5], [0.625, 1.875, 2.5]]
    assert np.allclose(table.project(("A", "B")), AB_exact)

    assert_frame_equal(
        table.project_melted("A"),
        pl.DataFrame([A_keys, pl.Series("len", A_exact).cast(float)]),
    )
    assert_frame_equal(
        table.project_melted("B"),
        pl.DataFrame([B_keys, pl.Series("len", B_exact).cast(float)]),
    )

    expected = {
        "A": ["a1"] * 3 + ["a2"] * 3,
        "B": ["b1", "b2", "b3"] * 2,
        "len": np.array(AB_exact).ravel().astype(float),
    }
    assert_frame_equal(table.project_melted(("A", "B")), pl.DataFrame(expected))


def test_make_contingency_table_multi_fit():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    data = {
        "A": list(range(5)) * 100,
        "B": list(range(10)) * 50,
        "C": list(range(20)) * 25,
    }
    context = dp.Context.compositor(
        data=pl.LazyFrame(data),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=0.2),
    )

    table: ContingencyTable = (
        context.query(rho=0.15)
        .select("A", "B")
        .contingency_table(
            keys={"A": list(range(5)), "B": list(range(10))},
            algorithm=Fixed(queries=[Count(("A", "B"))]),
        )
        .release()
    )

    # expand the contingency table to also include ("B", "C")
    table2: ContingencyTable = (
        context.query(rho=0.05)
        .select("B", "C")
        .contingency_table(keys={"C": list(range(20))}, table=table)
        .release()
    )

    assert table2.schema == {"A": pl.Int64, "B": pl.Int64, "C": pl.Int64}


def test_make_contingency_table_invalid_d_out():
    pytest.importorskip("mbi")
    message = "d_out type ((f64, f64)) must be f64"
    with pytest.raises(ValueError, match=re.escape(message)):
        dp.mbi.make_contingency_table(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            dp.frame_distance(dp.symmetric_distance()),
            dp.max_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=(1.0, 1e-8),
        )


def get_model(domain: dict[str, int]):
    from mbi import CliqueVector, MarkovRandomField, Domain  # type: ignore[import-not-found]

    clique_vector = CliqueVector(domain=Domain.fromdict(domain), cliques=[], arrays={})
    return MarkovRandomField(potentials=clique_vector, marginals=clique_vector)


def test_contingency_table_invalid_shape():
    pytest.importorskip("mbi")
    with pytest.raises(
        ValueError, match="Model domain must match key attrs and sizes."
    ):
        ContingencyTable(
            keys={},
            cuts={},
            marginals={},
            model=get_model({"A": 3}),
        )


def test_contingency_table_missing_cut():
    pytest.importorskip("mbi")
    with pytest.raises(ValueError, match='"B" in cuts is not present in keys'):
        ContingencyTable(
            keys={"A": [1, 2, 3]},
            cuts={"B": [1, 2, 3]},
            marginals={},
            model=get_model({"A": 3}),
        )


def test_contingency_table_misshapen_cut():
    pytest.importorskip("mbi")
    msg = '"B" keyset length (3) must be one greater than the number of cuts (3)'
    with pytest.raises(ValueError, match=re.escape(msg)):
        ContingencyTable(
            keys={"A": [1, 2, 3], "B": ["a", "b", "c"]},
            cuts={"B": [1, 2, 3]},
            marginals={},
            model=get_model({"A": 3, "B": 3}),
        )


def test_contingency_table_delta():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
    )

    message = "delta must be zero because keys and cuts span all columns"
    with pytest.raises(ValueError, match=message):
        context.query(epsilon=1.0, delta=1e-8).contingency_table(keys={"A": [1]})

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )

    message = "delta must be nonzero because keys and cuts don't span all columns"
    with pytest.raises(ValueError, match=message):
        context.query(epsilon=1.0).contingency_table()

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
    )

    context.query(epsilon=1.0, delta=1e-7).contingency_table(
        algorithm=AIM(oneway_split=0.9)
    )


def test_contingency_table_len():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    context = dp.Context.compositor(
        data=pl.LazyFrame({"len": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )

    message = 'input_domain must not contain a column named "len"'
    with pytest.raises(ValueError, match=message):
        context.query(epsilon=1.0).contingency_table(keys={"len": [1]})


def test_contingency_table_minimum_variance_weighted_total():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1] * 1000}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
    )

    # tests fit when no columns have keys, so total is estimated directly
    table: ContingencyTable = (
        context.query(epsilon=1.0, delta=1e-8)
        .contingency_table(algorithm=Fixed(queries=[Count(("A",))], oneway_split=0.9))
        .release()
    )

    assert 950 < table.project([]) < 1050


def test_unique():
    pytest.importorskip("pl")
    import polars as pl  # type: ignore[import-not-found]

    with pytest.raises(ValueError, match='cuts must be unique: "col" has duplicates'):
        _unique(pl.Series("col", ["A", "A"]), "cuts")


def test_increasing():
    pytest.importorskip("pl")
    import polars as pl  # type: ignore[import-not-found]

    message = 'cuts must be strictly increasing: "col" is not strictly increasing'
    with pytest.raises(ValueError, match=message):
        _increasing(pl.Series("col", [1, 2, 3, 3]))


def test_with_null():
    pytest.importorskip("pl")
    import polars as pl  # type: ignore[import-not-found]

    assert _with_null(pl.Series(["a", "b"])) == pl.Series(["a", "b", None])
    assert _with_null(pl.Series(["a", None, "b"])) == pl.Series(["a", None, "b"])


def test_get_null_index():
    pytest.importorskip("pl")
    import polars as pl  # type: ignore[import-not-found]

    assert _get_null_index(pl.Series(["a", "b", None])) == 2
    assert _get_null_index(pl.Series(["a", None, "b"])) == 1
    assert _get_null_index(pl.Series(["a", "b"])) is None
