from opendp.extras.contingency import Fixed, AIM, MST, ContingencyTable, Count
import pytest
import opendp.prelude as dp


def assert_synthetic_is_similar(sampler):
    np = pytest.importorskip("numpy")
    pl = pytest.importorskip("polars")

    cov = np.array([[1.0, 0.75, 0.0], [0.75, 1.0, 0.0], [0.0, 0.0, 1.0]])

    data = np.random.multivariate_normal(
        mean=[0.0] * 3,
        cov=cov,
        size=10_000,
    )

    categories_B = list("abcdef")
    edges_B = np.linspace(-2, 2, num=5)
    edges_C = np.linspace(-3, 3, num=7)
    data = {
        # continuous float
        "A": data.T[0],
        # categorical string
        "B": np.array(categories_B)[np.digitize(data.T[1], bins=edges_B)],
        # categorical int
        "C": np.digitize(data.T[2], bins=edges_C),
    }
    lf = pl.LazyFrame(data)
    lookup_B = {"a": -2.5, "b": -1.5, "c": -0.5, "d": 0.5, "e": 1.5, "f": 2.5}
    lookup_C = {0: -3.5, 1: -2.5, 2: -1.5, 3: 0.5, 4: 0.5, 5: 1.5, 6: 2.5, 7: 3.5}
    data_roundtrip = np.stack(
        [
            data["A"],
            [lookup_B[i] for i in data["B"]],
            [lookup_C[i] for i in data["C"]],
        ]
    )
    assert np.linalg.norm(cov - np.cov(data_roundtrip)) < 0.5

    data = sampler(lf)

    data_roundtrip = np.stack(
        [
            data["A"],
            [lookup_B[c] for c in data["B"]],
            [lookup_C[c] for c in data["C"]],
        ]
    )
    assert np.linalg.norm(cov - np.cov(data_roundtrip)) < 0.5


@pytest.mark.parametrize(
    "config,parameter,approximate",
    [
        AIM(),
        MST(),
        Fixed(queries=[Count(["A"]), Count(["B"]), Count(["C"]), Count(["A", "B"])]),
    ],
    [
        {"epsilon": 1.0},
        {"rho": 0.5},
    ],
    [True, False]
)
def test_contingency_table(config, parameter, approximate):
    cuts = {"A": [-2, -1, 0, 1, 2]}
    if approximate:
        parameter["delta"] = 1e-8
    else:
        cuts["C"] = [-3, -2, -1, 0, 1, 2, 3]

    def synthesize(lf):
        context = dp.Context.compositor(
            data=lf,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(**parameter),
        )

        table: ContingencyTable = (
            context.query(**parameter)
            .contingency_table(
                cuts=cuts,
                keys={"B": ["a", "b", "c", "d", "e", "f"]},
                config=config,
            )
            .release()
        )

        with pytest.warns(DeprecationWarning):
            return table.synthesize(rows=100_000)

    assert_synthetic_is_similar(synthesize)


import numpy as np

np.set_printoptions(precision=4, threshold=None)
dp.enable_features("contrib", "rust-stack-trace")
test_contingency_table(AIM(), {"rho": 0.5}, approximate=False)
# test_contingency_table(MST(), {"rho": 0.5})
# test_contingency_table(Fixed(queries=[Count(["A", "B"])]), {"epsilon": 1.0})
