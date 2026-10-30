from dataclasses import FrozenInstanceError

import pytest
from opendp.extras.mbi import Count, AIM
import re
from opendp._internal import _extrinsic_distance
import opendp.prelude as dp
from opendp.extras.mbi._utilities import (
    Marginals,
    SelectPrefixQuery,
    _noise_sensitivity,
    _snapshot_bounds,
    get_cardinalities,
    get_scale,
    get_std,
    identity_query_precision,
    make_noise_marginal,
    make_stable_marginals,
    mirror_descent,
    typed_dict_distance,
    typed_dict_domain,
)

from ..helpers import ids


def test_count_post_init():
    with pytest.raises(ValueError):
        Count(("A",), weight=-1)

    by = ["A"]
    count = Count(by, weight=1.0)  # type: ignore[arg-type]
    by[0] = "B"
    assert count.by == ("A",)
    with pytest.raises(FrozenInstanceError):
        count.weight = 2.0  # type: ignore[misc]


def test_algorithm_post_init():
    with pytest.raises(ValueError):
        AIM(oneway_split=2.0)

    with pytest.raises(ValueError):
        AIM(oneway="other")  # type: ignore[arg-type]


def test_typed_dict_domain():
    domain = typed_dict_domain({"A": dp.atom_domain(T=int)})

    with pytest.warns(Warning, match="data must be a dict"):
        domain.member(1)

    with pytest.raises(Warning, match="data must share key-set with domain"):
        domain.member(dict())

    with pytest.raises(Warning, match="does not belong to carrier type"):
        domain.member(dict(A=True))

    assert domain.member(dict(A=1))


def test_typed_dict_domain_snapshots_mapping():
    domains = {"A": dp.atom_domain(T=int)}
    domain = typed_dict_domain(domains)

    domains["A"] = dp.atom_domain(T=str)
    domains["B"] = dp.atom_domain(T=int)
    assert domain.member({"A": 1})
    with pytest.warns(Warning, match="does not belong to carrier type"):
        domain.member({"A": "changed"})


def test_snapshot_bounds():
    pl = pytest.importorskip("polars")

    by = [pl.col("A")]
    bound = dp.polars.Bound(by=by, per_group=2, num_groups=3)
    bounds = [bound]
    snapshot = _snapshot_bounds(bounds)

    by[0] = pl.col("B")
    bound.per_group = 9
    bounds.clear()

    assert len(snapshot) == 1
    assert snapshot[0].by[0].meta.output_name() == "A"
    assert snapshot[0].per_group == 2
    assert snapshot[0].num_groups == 3


def test_get_std():
    message = "output_measure (RenyiDivergence) must be"
    with pytest.raises(ValueError, match=re.escape(message)):
        get_std(dp.renyi_divergence(), 1.0)


def test_get_scale():
    assert get_scale(dp.max_divergence(), 4.0, 2.0) == 0.5
    assert get_scale(dp.zero_concentrated_divergence(), 8.0, 4.0) == 1.0

    message = "output_measure (RenyiDivergence) must be"
    with pytest.raises(ValueError, match=re.escape(message)):
        get_scale(dp.renyi_divergence(), 1.0, 1.0)


@pytest.mark.parametrize(
    "domain, message",
    [
        (
            dp.atom_domain(T=int),
            "input_domain must be dp.LazyFrameDomain",
        ),
        (
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.array_domain(dp.atom_domain(T=int), width=4))]
            ),
            "input_domain columns must contain atomic data",
        ),
        (
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            "input_domain columns must be bounded",
        ),
        (
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(bounds=(-1, 3)))]
            ),
            "input_domain columns must be lower bounded by zero",
        ),
    ],
    ids=ids,
)
def test_get_cardinalities(domain, message):
    with pytest.raises(Exception, match=re.escape(message)):
        get_cardinalities(domain)


def test_make_stable_marginals():
    pytest.importorskip("mbi")

    msg = "input_metric (DiscreteDistance()) must be frame_distance"
    with pytest.raises(ValueError, match=re.escape(msg)):
        make_stable_marginals(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T="i32"))]),
            dp.discrete_distance(),
            cliques=[("A",)],
        )

    with pytest.raises(ValueError, match="input_domain columns must be bounded"):
        make_stable_marginals(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T="i32"))]),
            dp.frame_distance(dp.symmetric_distance()),
            cliques=[("A",)],
        )


def test_make_stable_marginals_snapshots_cliques():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    cliques = [["A"]]
    transformation = make_stable_marginals(
        dp.lazyframe_domain(
            [
                dp.series_domain(c, dp.atom_domain(T="i32", bounds=(0, 1)))
                for c in ("A", "B")
            ]
        ),
        dp.frame_distance(dp.symmetric_distance()),
        cliques,  # type: ignore[arg-type]
    )
    bounds = [dp.polars.Bound(per_group=1)]
    expected = transformation.map(bounds)

    cliques[0][0] = "B"
    cliques.append(["B"])
    assert transformation.map(bounds) == expected
    assert set(transformation(pl.LazyFrame({"A": [0], "B": [1]}))) == {("A",)}


def test_make_stable_marginals_clips_u32_counts(monkeypatch):
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    domain = dp.lazyframe_domain(
        [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 0)))]
    )
    transformation = make_stable_marginals(
        domain,
        dp.frame_distance(dp.symmetric_distance()),
        cliques=[("A",)],
    )

    # Avoid materializing more than i32::MAX rows while still exercising the
    # UInt32-to-Int32 conversion performed by the marginal transformation.
    counts = pl.DataFrame(
        {
            "A": pl.Series("A", [0], dtype=pl.UInt32),
            "len": pl.Series("len", [np.iinfo(np.uint32).max], dtype=pl.UInt32),
        }
    )
    monkeypatch.setattr(pl, "collect_all", lambda _: [counts])

    marginals = transformation(pl.LazyFrame({"A": [0]}))
    assert marginals[("A",)].tolist() == [np.iinfo(np.int32).max]


def test_marginal_sensitivities_select_and_measure():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    clique = ("A",)
    marginals = make_stable_marginals(
        dp.lazyframe_domain(
            [dp.series_domain("A", dp.atom_domain(T="i32", bounds=(0, 8)))]
        ),
        dp.frame_distance(dp.symmetric_distance()),
        [clique],
    )
    distance = marginals.map(
        [
            dp.polars.Bound(per_group=9),
            dp.polars.Bound(by=[pl.col("A")], num_groups=9, per_group=1),
        ]
    )
    assert distance == {clique: (9, 9, 1)}
    assert marginals.map([dp.polars.Bound(per_group=0)]) == {clique: (0, 0, 0)}

    # A per-clique bound alone does not bound total contributions.
    with pytest.raises(
        Exception, match="input bounds must specify total contributions"
    ):
        marginals.map([dp.polars.Bound(by=[pl.col("A")], num_groups=9, per_group=1)])

    laplace = make_noise_marginal(
        *marginals.output_space, dp.max_divergence(), clique, 1
    )
    gaussian = make_noise_marginal(
        *marginals.output_space, dp.zero_concentrated_divergence(), clique, 1
    )
    assert laplace.map(distance) == 9.0
    assert gaussian.map(distance) == pytest.approx(4.5)
    assert gaussian.map({clique: (2, 2, 1)}) == pytest.approx(1.0)


def test_noise_sensitivity_rounds_up():
    sensitivity = _noise_sensitivity(dp.zero_concentrated_divergence(), (2, 2, 1))
    numerator, denominator = sensitivity.as_integer_ratio()
    assert numerator**2 >= 2 * denominator**2
    assert sensitivity == pytest.approx(2**0.5)


def test_make_noise_marginal():
    pytest.importorskip("mbi")
    kwargs = dict(
        input_domain=typed_dict_domain(
            {("A",): dp.numpy.arrayd_domain(shape=(1, 2), T="u32")}
        ),
        input_metric=typed_dict_distance(
            dp.l01inf_distance(dp.absolute_distance(T="i32"))
        ),
        output_measure=dp.max_divergence(),
        clique=("A",),
        scale=1.0,
    )

    def kwargs_without(*without):
        return {k: v for k, v in kwargs.items() if k not in without}

    with pytest.raises(ValueError, match="domain descriptor must be a TypedDictDomain"):
        make_noise_marginal(
            input_domain=dp.numpy.array2_domain(T="f64"),
            **kwargs_without("input_domain"),
        )

    with pytest.raises(ValueError, match="domain descriptor must be a NPArrayDDomain"):
        make_noise_marginal(
            input_domain=typed_dict_domain({("A",): dp.atom_domain(T="bool")}),
            **kwargs_without("input_domain"),
        )

    with pytest.raises(
        ValueError, match="metric descriptor must be a TypedDictDistance"
    ):
        make_noise_marginal(
            input_metric=_extrinsic_distance("DummyDomain"),
            **kwargs_without("input_metric"),
        )

    msg = (
        "input_metric's inner metric (L1Distance(f64)) must be "
        "L01InfDistance(AbsoluteDistance(i32))"
    )
    with pytest.raises(ValueError, match=re.escape(msg)):
        make_noise_marginal(
            input_metric=typed_dict_distance(dp.l1_distance(T="f64")),
            **kwargs_without("input_metric"),
        )

    m_noise = make_noise_marginal(**kwargs)  # type: ignore[arg-type]
    assert m_noise.map({("A",): (1, 1, 1)}) == 1.0


def test_marginal_measurements_add_preserves_atomic_queries():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    with pytest.raises(
        ValueError, match="each new marginal must be of type LinearMeasurement"
    ):
        Marginals().add(False)

    partial = LinearMeasurement(
        np.array([1.0]),
        clique=("A",),
        stddev=2.0,
        query=SelectPrefixQuery(1),
    )
    full = LinearMeasurement(np.array([2.0, 3.0]), clique=("A",), stddev=1.0)
    repeated = LinearMeasurement(np.array([4.0, 5.0]), clique=("A",), stddev=3.0)
    measurements = Marginals({("A",): [partial]}).add(full, repeated)

    # Independent observations sharing a clique remain separate, regardless
    # of whether their query operators are equal.
    group = measurements.by_clique[("A",)]
    assert len(group) == 3
    assert group[0] is partial
    assert group[1] is full
    assert group[2] is repeated
    assert measurements.cliques() == [("A",)]
    assert identity_query_precision(full) == 1.0
    assert identity_query_precision(partial) == 0.0


def test_marginal_measurements_add_does_not_mutate():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    lm = LinearMeasurement(np.array([1.0]), clique=("A",), stddev=1.0)
    original = Marginals({("A",): [lm]})

    new_lm = LinearMeasurement(np.array([5.0, 6.0]), clique=("B",), stddev=1.0)
    updated = original.add(new_lm)

    assert original.cliques() == [("A",)]
    assert len(original.by_clique[("A",)]) == 1
    assert updated.cliques() == [("A",), ("B",)]


def test_latent_remainder_linear_measurement():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    measurement = LinearMeasurement(
        np.array([30.0]),
        clique=("A",),
        stddev=0.01,
        query=SelectPrefixQuery(1),
    )
    total = LinearMeasurement(np.array([100.0]), clique=(), stddev=0.01)
    model = mirror_descent(Domain(("A",), (2,)), [measurement, total])

    assert np.allclose(model.project(("A",)).values, [30.0, 70.0], atol=0.1)


def test_zero_way_measurement_retains_nonpositive_value():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    from mbi.estimation import minimum_variance_unbiased_total  # type: ignore[import-untyped,import-not-found]

    total = LinearMeasurement(np.array([-5.0]), clique=(), stddev=10.0)

    # The raw noisy total may be nonpositive; only the optimizer's mass is
    # floored to a valid positive value.
    assert total.noisy_measurement.item() == -5.0
    assert minimum_variance_unbiased_total([total]) >= 1.0

    oneway = LinearMeasurement(np.array([1.0, 2.0]), clique=("A",), stddev=1.0)
    model = mirror_descent(Domain(("A",), (2,)), [total, oneway])
    assert float(model.total) >= 1.0
