from opendp.transformations import (
    make_metric_bounded,
    make_metric_unbounded,
    make_ordered_random,
    make_unordered,
)
from opendp.domains import *
from opendp.typing import *
from opendp.mod import enable_features
enable_features("contrib")


def test_ordering():
    data = [1, 2, 3]
    domain = vector_domain(atom_domain((0, 3)))
    ord_trans = make_ordered_random(domain)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_unordered(domain)
    assert len(ident_trans(data)) == 3

def test_sized_ordering():
    data = [1, 2, 3]
    domain = vector_domain(atom_domain(T=i32), 3)
    ord_trans = make_ordered_random(domain)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_unordered(domain)
    assert len(ident_trans(data)) == 3

def test_sized_bounded_ordering():
    data = [1, 2, 3]
    domain = vector_domain(atom_domain((0, 3)), 3)
    ord_trans = make_ordered_random(domain)
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> make_unordered(domain)
    assert len(ident_trans(data)) == 3

def test_bounded():
    data = [1, 2, 3]

    domain = vector_domain(atom_domain(T=i32), 3)
    bdd_trans = make_metric_bounded(domain)
    assert len(bdd_trans(data)) == 3

    ident_trans = bdd_trans >> make_metric_unbounded(domain)
    assert len(ident_trans(data)) == 3
