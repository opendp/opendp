import opendp.prelude as dp


def test_ordering():
    data = [1, 2, 3]
    domain = dp.vector_domain(dp.atom_domain((0, 3)))
    ord_trans = dp.t.make_ordered_random(domain, dp.symmetric_distance())
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> dp.t.then_unordered()
    assert len(ident_trans(data)) == 3

def test_sized_ordering():
    data = [1, 2, 3]
    domain = dp.vector_domain(dp.atom_domain(T=dp.i32), 3)
    ord_trans = dp.t.make_ordered_random(domain, dp.symmetric_distance())
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> dp.t.then_unordered()
    assert len(ident_trans(data)) == 3

def test_sized_bounded_ordering():
    data = [1, 2, 3]
    domain = dp.vector_domain(dp.atom_domain((0, 3)), 3)
    ord_trans = dp.t.make_ordered_random(domain, dp.symmetric_distance())
    assert len(ord_trans(data)) == 3

    ident_trans = ord_trans >> dp.t.then_unordered()
    assert len(ident_trans(data)) == 3

def test_bounded():
    data = [1, 2, 3]

    domain = dp.vector_domain(dp.atom_domain(T=dp.i32), 3)
    bdd_trans = dp.t.make_metric_bounded(domain, dp.symmetric_distance())
    assert len(bdd_trans(data)) == 3

    ident_trans = bdd_trans >> dp.t.then_metric_unbounded()
    assert len(ident_trans(data)) == 3
