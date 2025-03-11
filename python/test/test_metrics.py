from opendp.extras.polars import GroupBound
from opendp.mod import MultiDistance, SymmetricIdDistance
import opendp.prelude as dp
import pytest


def test_partition_distance():
    domain = dp.vector_domain(dp.atom_domain(T=float))
    metric = dp.partition_distance(dp.symmetric_distance())
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

    group_bound = GroupBound(by=[], max_partition_contributions=1)
    metric = dp.unit_of(contributions=[group_bound], identifier="C")[0]
    assert isinstance(metric, MultiDistance)
    assert isinstance(metric.inner_metric, SymmetricIdDistance)
    assert metric.inner_metric.identifier.meta.serialize() == pl.col.C.meta.serialize()


def test_multi_distance():
    pytest.importorskip("polars")
    metric = dp.multi_distance(dp.symmetric_distance())
    # not a valid domain/metric pairing, but behind honest-but-curious
    t_dummy = dp.t.make_identity(dp.atom_domain(T=int), metric)
    d_in = [GroupBound(by=["A"], max_partition_contributions=10)]
    assert t_dummy.map(d_in) == d_in


def test_group_bound():
    pl = pytest.importorskip("polars")
    left = GroupBound(by=[pl.col.A, "B"], max_partition_contributions=10)
    right = GroupBound(by=[pl.col.B, "A"], max_partition_contributions=10)
    assert left == right
    assert not left == GroupBound(by=[pl.col.B], max_partition_contributions=10)
    assert not left == str(right)
