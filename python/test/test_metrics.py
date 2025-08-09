from opendp.extras.polars import Bound
from opendp.mod import ExtrinsicDistance, FrameDistance, SymmetricIdDistance
import opendp.prelude as dp
import pytest


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


def test_identifier_distance():
    pl = pytest.importorskip("polars")

    metric = dp.unit_of(contributions=1, identifier="A")[0]
    assert metric.identifier.meta.serialize() == pl.col.A.meta.serialize()

    group_bound = Bound(by=[], per_group=1)
    metric = dp.unit_of(contributions=[group_bound], identifier="C")[0]
    assert isinstance(metric, FrameDistance)
    assert isinstance(metric.inner_metric, SymmetricIdDistance)
    assert metric.inner_metric.identifier.meta.serialize() == pl.col.C.meta.serialize()


def test_frame_distance():
    pytest.importorskip("polars")
    metric = dp.frame_distance(dp.symmetric_distance())
    # not a valid domain/metric pairing, but behind honest-but-curious
    t_dummy = dp.t.make_identity(dp.atom_domain(T=int), metric)
    d_in = [Bound(by=["A"], per_group=10)]
    assert t_dummy.map(d_in) == d_in


def test_group_bound():
    pl = pytest.importorskip("polars")
    left = Bound(by=[pl.col.A, "B"], per_group=10)
    right = Bound(by=[pl.col.B, "A"], per_group=10)
    assert left == right
    assert not left == Bound(by=[pl.col.B], per_group=10)
    assert not left == str(right)


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
