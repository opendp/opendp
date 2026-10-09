import pytest
import opendp.prelude as dp
from opendp.extras.polars import Bound
from opendp.mod import FrameDistance, SymmetricIdDistance

pl = pytest.importorskip("polars")


def test_identifier_distance():
    metric = dp.unit_of(contributions=1, identifier="A")[0]
    assert metric.identifier.meta.serialize() == pl.col.A.meta.serialize()

    group_bound = Bound(by=[], per_group=1)
    metric = dp.unit_of(contributions=[group_bound], identifier="C")[0]
    assert isinstance(metric, FrameDistance)
    assert isinstance(metric.inner_metric, SymmetricIdDistance)
    assert metric.inner_metric.identifier.meta.serialize() == pl.col.C.meta.serialize()


def test_frame_distance():
    metric = dp.frame_distance(dp.symmetric_distance())
    # not a valid domain/metric pairing, but behind honest-but-curious
    t_dummy = dp.t.make_identity(dp.atom_domain(T=int), metric)
    d_in = [Bound(by=["A"], per_group=10)]
    assert t_dummy.map(d_in) == d_in


def test_group_bound():
    left = Bound(by=[pl.col.A, "B"], per_group=10)
    right = Bound(by=[pl.col.B, "A"], per_group=10)
    assert left == right
    assert not left == Bound(by=[pl.col.B], per_group=10)
    assert not left == str(right)
