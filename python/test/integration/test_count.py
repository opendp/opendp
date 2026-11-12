import opendp.prelude as dp


def test_count():
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    preprocess = space >> dp.t.then_count()
    noisy_count = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_count.check(1, 1.0)
    print(noisy_count(list(map(str, range(40)))))


def test_count_distinct():
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    preprocess = space >> dp.t.then_count_distinct()
    noisy_count = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_count.check(1, 1.0)
    print(noisy_count(list(map(str, range(40)))))


def test_float_count():
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    preprocess = space >> dp.t.then_count(TO=float)
    data = list(map(str, range(40)))
    print((preprocess >> dp.m.then_laplace(1.0))(data))
    print((preprocess >> dp.m.then_gaussian(1.0))(data))
