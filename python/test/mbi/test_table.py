import re
from opendp.extras.mbi import (
    Fixed,
    AIM,
    MST,
    ContingencyTable,
    Count,
    Marginals,
    Sequential,
)
from opendp.extras.mbi._table import (
    _get_null_index,
    _increasing,
    _make_oneway_marginals,
    _unique,
    _with_null,
)
from opendp.extras.mbi._utilities import mirror_descent, SelectPrefixQuery
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
    lookup_C[None] = np.float64(0)  # type: ignore[index]

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
        std = table.std(("A",))
        assert isinstance(std, float)
        # a coarser marginal that covers "A" (e.g. ("A", "B")) can now
        # legitimately inform this estimate, not just a direct ("A",) release,
        # so this is a loose sanity bound rather than a tight calibration
        assert 0 < std < 20
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
    cov_syn = np.cov(np.stack(synthetic_list))

    def test_cov(clique):
        idx = {"A": 0, "B": 1, "C": 2}
        i, j = idx[clique[0]], idx[clique[-1]]
        assert abs(cov[i, j] - cov_syn[i, j]) < 0.2, (
            f"{clique} cov drifted: {cov[i, j]=:.3f}, {cov_syn[i, j]=:.3f}"
        )

    # in an MRF, a missing 2-way marginal does not imply independence
    # max-entropy does not guarantee spurious correlations won't be formed on undetermined pairs
    test_cov(("A", "B"))
    for clique in (cl for cl in table.marginals.cliques() if 0 < len(cl) <= 2):
        test_cov(clique)


def test_contingency_table_int_cuts():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    exact = np.array([100, 200, 400, 300, 200])

    lm = LinearMeasurement(exact, clique=("A",), stddev=0.01)
    model = mirror_descent(Domain(("A",), (5,)), [lm])
    A_cuts = pl.Series("A", [0, 2, 4, 6])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a", "b", "c", "d", "e"])},
        cuts={"A": A_cuts},
        # Exercise the backwards-compatible dict-to-Marginals conversion.
        marginals={("A",): lm},  # type: ignore[arg-type]
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

    A_exact = np.array([3, 5])
    B_exact = np.array([1, 3, 4])
    A_keys = pl.Series("A", ["a1", "a2"])
    B_keys = pl.Series("B", ["b1", "b2", "b3"])

    A_lm = LinearMeasurement(A_exact, clique=("A",))
    B_lm = LinearMeasurement(B_exact, clique=("B",))

    model = mirror_descent(Domain(("A", "B"), (2, 3)), [A_lm, B_lm])
    table = ContingencyTable(
        keys={"A": A_keys, "B": B_keys},
        cuts={},
        marginals=Marginals({("A",): [A_lm], ("B",): [B_lm]}),
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

    cliques_before = set(table.marginals.cliques())

    # expand the contingency table to also include ("B", "C")
    table2: ContingencyTable = (
        context.query(rho=0.05)
        .select("B", "C")
        .contingency_table(keys={"C": list(range(20))}, table=table)
        .release()
    )

    assert table2.schema == {"A": pl.Int64, "B": pl.Int64, "C": pl.Int64}

    # reusing `table` as input to a later query must not mutate it
    assert set(table.marginals.cliques()) == cliques_before
    assert set(table2.marginals.cliques()) > cliques_before


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
            marginals=Marginals(),
            model=get_model({"A": 3}),
        )


def test_contingency_table_missing_cut():
    pytest.importorskip("mbi")
    with pytest.raises(ValueError, match='"B" in cuts is not present in keys'):
        ContingencyTable(
            keys={"A": [1, 2, 3]},
            cuts={"B": [1, 2, 3]},
            marginals=Marginals(),
            model=get_model({"A": 3}),
        )


def test_contingency_table_misshapen_cut():
    pytest.importorskip("mbi")
    msg = '"B" keyset length (3) must be one greater than the number of cuts (3)'
    with pytest.raises(ValueError, match=re.escape(msg)):
        ContingencyTable(
            keys={"A": [1, 2, 3], "B": ["a", "b", "c"]},
            cuts={"B": [1, 2, 3]},
            marginals=Marginals(),
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

    message = "delta (1e-08) must be zero because keys and cuts span all columns"
    with pytest.raises(ValueError, match=re.escape(message)):
        context.query(epsilon=1.0, delta=1e-8).contingency_table(keys={"A": [1]})

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )

    message = (
        "delta (None) must be nonzero because keys and cuts don't span all columns"
    )
    with pytest.raises(ValueError, match=re.escape(message)):
        context.query(epsilon=1.0).contingency_table()

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
    )

    context.query(epsilon=1.0, delta=1e-7).contingency_table(
        algorithm=AIM(oneway_split=0.9)
    )


def test_make_oneway_marginals_releases_total():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]
    from opendp.domains import _lazyframe_from_domain

    input_domain = dp.lazyframe_domain(
        [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
    )
    m_oneway, _, _ = _make_oneway_marginals(
        input_domain,
        dp.frame_distance(dp.symmetric_distance()),
        dp.approximate(dp.max_divergence()),
        d_in=[dp.polars.Bound(per_group=1)],
        d_out=(0.5, 1e-8),
        keys={},
        plan=_lazyframe_from_domain(input_domain),
        unknown_only=False,
        has_prior_total=False,
        release_total=True,
    )

    _, releases = m_oneway(pl.LazyFrame({"A": [0]}))
    assert any(release.clique == () for release in releases)


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

    # tests fit when no columns have keys, using the fixed identity marginal
    # to constrain the total
    table: ContingencyTable = (
        context.query(epsilon=1.0, delta=1e-8)
        .contingency_table(algorithm=Fixed(queries=[Count(("A",))], oneway_split=0.9))
        .release()
    )

    assert 950 < table.project([]) < 1050


def test_fixed_unkeyed_does_not_release_zero_way_total():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1] * 1000}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-8),
    )

    table: ContingencyTable = (
        context.query(epsilon=1.0, delta=1e-8)
        .contingency_table(algorithm=Fixed(queries=[Count(("A",))], oneway_split=0.9))
        .release()
    )

    assert () not in table.marginals.by_clique
    fixed_measurement = table.marginals.by_clique[("A",)][-1]
    # The fixed workload receives the remaining 10% of the budget.
    assert fixed_measurement.stddev == pytest.approx(10 * 2**0.5)


def test_contingency_table_std_ignores_zero_way_total():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    A_lm = LinearMeasurement(np.array([3.0, 5.0]), clique=("A",), stddev=1.0)
    total_lm = LinearMeasurement(np.array([8.0]), clique=(), stddev=0.01)

    model = mirror_descent(Domain(("A",), (2,)), [A_lm, total_lm])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"])},
        cuts={},
        marginals=Marginals({("A",): [A_lm], (): [total_lm]}),
        model=model,
    )

    # a total constrains the sum of cells, not each cell directly,
    # so its precision shouldn't count towards the std of an individual cell
    assert table.std(("A",)) == pytest.approx(1.0)


def test_contingency_table_std_sums_independent_identity_measurements():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    # a partial (prefix) query contributes no per-cell precision, so only
    # `full`'s precision should count -- this is only reachable now that a
    # clique can hold more than one independent measurement
    prefix = LinearMeasurement(
        np.array([3.0]), clique=("A",), stddev=1.0, query=SelectPrefixQuery(1)
    )
    full = LinearMeasurement(np.array([3.0, 5.0]), clique=("A",), stddev=2.0)

    model = mirror_descent(Domain(("A",), (2,)), [prefix, full])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"])},
        cuts={},
        marginals=Marginals({("A",): [prefix, full]}),
        model=model,
    )
    assert table.std(("A",)) == pytest.approx(full.stddev)

    # two independent full identity measurements over the same clique each
    # contribute precision independently, summing by inverse variance
    full2 = LinearMeasurement(np.array([3.0, 5.0]), clique=("A",), stddev=4.0)
    model2 = mirror_descent(Domain(("A",), (2,)), [full, full2])
    table2 = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"])},
        cuts={},
        marginals=Marginals({("A",): [full, full2]}),
        model=model2,
    )
    expected_std = (1 / full.stddev**2 + 1 / full2.stddev**2) ** -0.5
    assert table2.std(("A",)) == pytest.approx(expected_std)


def test_contingency_table_std_larger_measurement_informs_smaller_projection():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    # a complete identity measurement over (A, B) informs the projection
    # onto the smaller clique (A,), by summing independent noisy cells
    AB_lm = LinearMeasurement(
        np.array([1.0, 2.0, 3.0, 4.0]), clique=("A", "B"), stddev=2.0
    )
    domain = Domain(("A", "B"), (2, 2))
    model = mirror_descent(domain, [AB_lm])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"]), "B": pl.Series("B", ["b1", "b2"])},
        cuts={},
        marginals=Marginals({("A", "B"): [AB_lm]}),
        model=model,
    )

    expected = AB_lm.stddev * (domain.size(("A", "B")) / domain.size(("A",))) ** 0.5
    assert table.std(("A",)) == pytest.approx(expected)


def test_contingency_table_std_smaller_measurement_does_not_inform_larger_projection():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    # a measurement over (A,) alone cannot provide per-cell uncertainty
    # for the larger clique (A, B)
    A_lm = LinearMeasurement(np.array([1.0, 2.0]), clique=("A",), stddev=1.0)
    model = mirror_descent(Domain(("A", "B"), (2, 2)), [A_lm])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"]), "B": pl.Series("B", ["b1", "b2"])},
        cuts={},
        marginals=Marginals({("A",): [A_lm]}),
        model=model,
    )

    with pytest.raises(ValueError, match="are not covered by the query set"):
        table.std(("A", "B"))


def test_contingency_table_std_of_total():
    pytest.importorskip("mbi")
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    # any complete identity measurement informs the zero-way total, since
    # a nonempty clique is always a superset of the empty attrs set
    A_lm = LinearMeasurement(np.array([3.0, 5.0]), clique=("A",), stddev=1.0)
    domain = Domain(("A",), (2,))
    model = mirror_descent(domain, [A_lm])
    table = ContingencyTable(
        keys={"A": pl.Series("A", ["a1", "a2"])},
        cuts={},
        marginals=Marginals({("A",): [A_lm]}),
        model=model,
    )

    expected = A_lm.stddev * domain.size(("A",)) ** 0.5
    assert table.std(()) == pytest.approx(expected)


def test_unique():
    pytest.importorskip("polars")
    import polars as pl  # type: ignore[import-not-found]

    with pytest.raises(ValueError, match='cuts must be unique: "col" has duplicates'):
        _unique(pl.Series("col", ["A", "A"]), "cuts")


def test_increasing():
    pytest.importorskip("polars")
    import polars as pl  # type: ignore[import-not-found]

    message = 'cuts must be strictly increasing: "col" is not strictly increasing'
    with pytest.raises(ValueError, match=message):
        _increasing(pl.Series("col", [1, 2, 3, 3]))


def test_with_null():
    pytest.importorskip("polars")
    import polars as pl  # type: ignore[import-not-found]
    from polars.testing import assert_series_equal

    assert_series_equal(_with_null(pl.Series(["a", "b"])), pl.Series(["a", "b", None]))
    assert_series_equal(
        _with_null(pl.Series(["a", None, "b"])), pl.Series(["a", None, "b"])
    )


def test_get_null_index():
    pytest.importorskip("polars")
    import polars as pl  # type: ignore[import-not-found]

    assert _get_null_index(pl.Series(["a", "b", None])) == 2
    assert _get_null_index(pl.Series(["a", None, "b"])) == 1
    assert _get_null_index(pl.Series(["a", "b"])) is None
