import pytest
from opendp.extras.mbi import Count, AIM
import re
from opendp._internal import _extrinsic_distance
import opendp.prelude as dp
from opendp.extras.mbi._utilities import (
    Marginals,
    SelectPrefixQuery,
    get_cardinalities,
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


def test_get_std():
    message = "output_measure (RenyiDivergence) must be"
    with pytest.raises(ValueError, match=re.escape(message)):
        get_std(dp.renyi_divergence(), 1.0)


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
            dp.l1_distance(T="u32"),
            cliques=[("A",)],
        )

    with pytest.raises(ValueError, match="input_domain columns must be bounded"):
        make_stable_marginals(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T="i32"))]),
            dp.frame_distance(dp.symmetric_distance()),
            dp.l1_distance(T="u32"),
            cliques=[("A",)],
        )

    with pytest.raises(
        ValueError,
        match=re.escape("inner_output_metric (L1Distance(i32)) must be in"),
    ):
        make_stable_marginals(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(T="i32", bounds=(0, 10)))]
            ),
            dp.frame_distance(dp.symmetric_distance()),
            dp.l1_distance(T="i32"),
            cliques=[("A",)],
        )


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
        dp.l1_distance(T="u32"),
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


def test_make_noise_marginal():
    pytest.importorskip("mbi")
    kwargs = dict(
        input_domain=typed_dict_domain(
            {("A",): dp.numpy.arrayd_domain(shape=(1, 2), T="u32")}
        ),
        input_metric=typed_dict_distance(dp.l1_distance(T="u32")),
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

    msg = "input_metric's inner metric (L1Distance(f64)) doesn't match the output_measure's associated metric (L1Distance(u32))"
    with pytest.raises(ValueError, match=re.escape(msg)):
        make_noise_marginal(
            input_metric=typed_dict_distance(dp.l1_distance(T="f64")),
            **kwargs_without("input_metric"),
        )

    m_noise = make_noise_marginal(**kwargs)  # type: ignore[arg-type]
    assert m_noise.map({("A",): 1}) == 1.0


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
    repeated = LinearMeasurement(
        np.array([4.0, 5.0]), clique=("A",), stddev=3.0
    )
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
