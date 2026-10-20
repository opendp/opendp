import pytest

from opendp.extras.polars import Bound
from opendp.mod import ExtrinsicDistance, FrameDistance, SymmetricIdDistance
import opendp.prelude as dp


def test_l01inf_distance():
    domain = dp.vector_domain(dp.atom_domain(T=float))
    metric = dp.l01inf_distance(dp.symmetric_distance())
    assert metric != str(metric)
    trans = dp.t.make_user_transformation(
        domain,
        metric,
        domain,
        metric,
        function=lambda x: x,
        stability_map=lambda d_in: d_in,
    )

    assert trans.map((3, 4, 3)) == (3, 4, 3)


def test_user_metric_total_cmp_native_distance():
    m_comp = dp.c.make_adaptive_composition(
        input_domain=dp.atom_domain(T=bool),
        input_metric=dp.user_distance("user distance"),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_mids=[1.0],
    )
    assert m_comp.map(0) == 1.0
    assert m_comp.map(1) == 1.0
    with pytest.raises(
        dp.OpenDPException,
        match="d_in from the privacy map must be no greater than the d_in",
    ):
        m_comp.map(2)
    with pytest.raises(dp.OpenDPException, match="not comparable"):
        m_comp.map(float("nan"))


def test_user_metric_total_cmp_custom_distance():
    class Dist:
        def __lt__(self, other):
            raise ValueError("comparison failed!")

    m_comp = dp.c.make_adaptive_composition(
        input_domain=dp.atom_domain(T=bool),
        input_metric=dp.user_distance("user distance", ["other", "data"]),
        output_measure=dp.max_divergence(),
        d_in=Dist(),
        d_mids=[1.0],
    )
    with pytest.raises(dp.OpenDPException, match="comparison failed!"):
        m_comp.map(Dist())

    assert isinstance(m_comp.input_metric, ExtrinsicDistance)
    assert m_comp.input_metric.cast(list) == ["other", "data"]
    assert m_comp.input_metric.descriptor == ["other", "data"]

    with pytest.raises(ValueError, match="metric descriptor must be a int"):
        m_comp.input_metric.cast(int)


def test_bound():
    pytest.importorskip("polars")
    zero_way = dp._get_bound([Bound(by=["A"], per_group=2)], ["A", "B"])
    assert zero_way == Bound(by=["A", "B"], per_group=2)
