from opendp.trans import (
    make_metric_bounded,
    make_metric_unbounded,
    make_ordered_random,
    make_unordered,
    make_sized_ordered_random,
    make_sized_bounded_ordered_random,
    make_sized_unordered,
    make_sized_bounded_unordered,
)
from opendp.mod import enable_features
from opendp.typing import *
enable_features("contrib")


def test_ordering():
    data = [1, 2, 3]
    ord_trans = make_ordered_random(i32)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_unordered(i32)
    assert len(ident_trans(data)) == 3

def test_sized_ordering():
    data = [1, 2, 3]
    ord_trans = make_sized_ordered_random(3, i32)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_sized_unordered(3, i32)
    assert len(ident_trans(data)) == 3

def test_sized_bounded_ordering():
    data = [1, 2, 3]
    ord_trans = make_sized_bounded_ordered_random(3, (3453, 23245), i32)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_sized_bounded_unordered(3, (3453, 23245), i32)
    assert len(ident_trans(data)) == 3

def test_bounded():
    data = [1, 2, 3]

    bdd_trans = make_metric_bounded(3, i32)
    assert len(bdd_trans(data)) == 3

    ident_trans = bdd_trans >> make_metric_unbounded(3, i32)
    assert len(ident_trans(data)) == 3
