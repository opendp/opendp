import pytest
from opendp.extras.mbi import Count, AIM
from math import sqrt
import re
from opendp._internal import _extrinsic_distance
import opendp.prelude as dp
from opendp.extras.mbi._utilities import (
    get_cardinalities,
    get_std,
    make_noise_marginal,
    make_stable_marginals,
    typed_dict_distance,
    typed_dict_domain,
    weight_marginals,
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


def test_weight_marginals():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

    with pytest.raises(
        ValueError, match="each new marginal must be of type LinearMeasurement"
    ):
        weight_marginals({}, False)

    lm1 = LinearMeasurement([1], clique=("A",), stddev=1.0)
    lm2 = LinearMeasurement([2], clique=("A",), stddev=1.0)
    marginals = weight_marginals({("A",): lm1}, lm2)
    weighted: LinearMeasurement = marginals[("A",)]
    assert weighted.stddev == sqrt(1 / 2)

    assert np.array_equal(weighted.noisy_measurement, [1.5])
