import pytest
from opendp.extras.mbi import Count, AIM
from math import sqrt
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


def test_marginal_measurements_add_combines_same_query():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    with pytest.raises(
        ValueError, match="each new marginal must be of type LinearMeasurement"
    ):
        Marginals().add(False)

    lm1 = LinearMeasurement(np.array([1]), clique=("A",), stddev=1.0)
    lm2 = LinearMeasurement(np.array([2]), clique=("A",), stddev=1.0)
    measurements = Marginals({("A",): [lm1]}).add(lm2)
    [weighted] = measurements.by_clique[("A",)]
    assert weighted.stddev == sqrt(1 / 2)

    assert np.array_equal(weighted.noisy_measurement, [1.5])


def test_marginal_measurements_add_keeps_distinct_queries_independent():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    partial = LinearMeasurement(
        np.array([1.0]),
        clique=("A",),
        stddev=2.0,
        query=SelectPrefixQuery(1),
    )
    full = LinearMeasurement(
        np.array([2.0, 3.0]), clique=("A",), stddev=1.0
    )

    measurements = Marginals({("A",): [partial]}).add(full)

    # a clique only groups measurements for indexing and model structure --
    # distinct query operators remain independent observations, even when
    # their cliques match, rather than being silently combined
    group = measurements.by_clique[("A",)]
    assert len(group) == 2
    assert group[0] is partial
    assert group[1] is full
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


def test_marginal_measurements_add_merges_zero_way_totals():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    early = LinearMeasurement(np.array([90.0]), clique=(), stddev=10.0)
    late = LinearMeasurement(np.array([120.0]), clique=(), stddev=5.0)

    measurements = Marginals({(): [early]}).add(late)
    [combined] = measurements.by_clique[()]

    expected_var = 1 / (1 / 10.0**2 + 1 / 5.0**2)
    expected_mean = (90.0 / 10.0**2 + 120.0 / 5.0**2) * expected_var

    assert combined.stddev == pytest.approx(sqrt(expected_var))
    assert np.allclose(combined.noisy_measurement, [expected_mean])
    # a later, noisier total does not overwrite a precise earlier one
    assert combined.stddev < late.stddev


def test_zero_way_measurement_retains_nonpositive_value():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    from mbi.estimation import minimum_variance_unbiased_total  # type: ignore[import-untyped,import-not-found]

    total = LinearMeasurement(np.array([-5.0]), clique=(), stddev=10.0)

    # the raw noisy measurement is preserved, even though it's implausible
    assert total.noisy_measurement.item() == -5.0

    # but the model mass supplied to the optimizer is floored to be positive
    assert minimum_variance_unbiased_total([total]) >= 1.0

    # a zero-way measurement is always accompanied by at least one marginal
    # that covers every domain attribute, as it is in practice
    oneway = LinearMeasurement(np.array([1.0, 2.0]), clique=("A",), stddev=1.0)
    model = mirror_descent(Domain(("A",), (2,)), [total, oneway])
    assert float(model.total) >= 1.0


def test_atomic_measurements_sum_losses():
    """Passing two atomic measurements over the same clique to MBI is
    equivalent to summing their individual losses -- the invariant that
    justifies keeping independent observations atomic instead of stacking
    them into a single composite query."""
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import (  # type: ignore[import-untyped,import-not-found]
        CliqueVector,
        Domain,
        Factor,
        LinearMeasurement,
        marginal_loss,
    )

    dom = Domain(("A",), (2,))
    m1 = LinearMeasurement(np.array([1.0, 2.0]), clique=("A",), stddev=1.0)
    m2 = LinearMeasurement(np.array([3.0, 4.0]), clique=("A",), stddev=2.0)

    mu = CliqueVector(dom, [("A",)], {("A",): Factor(dom, np.array([1.5, 2.5]))})

    loss_both = marginal_loss.from_linear_measurements([m1, m2], domain=dom)
    loss1 = marginal_loss.from_linear_measurements([m1], domain=dom)
    loss2 = marginal_loss.from_linear_measurements([m2], domain=dom)

    assert float(loss_both(mu)) == pytest.approx(float(loss1(mu)) + float(loss2(mu)))


def test_repeated_fit_preserves_prior_and_later_observations():
    """A later full marginal over a previously-partial clique remains visible
    to MBI's total estimator, and doesn't overwrite the earlier prefix
    observation -- the scenario that motivated the measurement multimap."""
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    from mbi.estimation import minimum_variance_unbiased_total  # type: ignore[import-untyped,import-not-found]

    # 1. initial unknown-key prefix release
    prefix = LinearMeasurement(
        np.array([30.0]), clique=("A",), stddev=1.0, query=SelectPrefixQuery(1)
    )
    measurements = Marginals({("A",): [prefix]})

    # 2. explicit zero-way total release
    total = LinearMeasurement(np.array([100.0]), clique=(), stddev=1.0)
    measurements = measurements.add(total)

    # 3. later full marginal over the same clique
    full = LinearMeasurement(np.array([30.0, 70.0]), clique=("A",), stddev=1.0)
    measurements = measurements.add(full)

    # both prefix and full observations remain present
    group = measurements.by_clique[("A",)]
    assert len(group) == 2
    assert any(m is prefix for m in group)
    assert any(m is full for m in group)

    # the later full marginal contributes to total estimation, alongside the
    # zero-way total; the earlier partial observation is correctly ignored
    flattened = measurements.flatten()
    estimated_total = minimum_variance_unbiased_total(flattened)
    assert estimated_total == pytest.approx(minimum_variance_unbiased_total([total, full]))

    # 4. refit from all accumulated measurements
    model = mirror_descent(Domain(("A",), (2,)), flattened)
    assert float(model.total) >= 1.0
