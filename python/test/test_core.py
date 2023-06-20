import pytest

import opendp.prelude as dp
dp.enable_features('floating-point', 'contrib')


def test_type_getters():
    transformation = dp.t.make_mean(
        dp.vector_domain(dp.atom_domain((0., 10.)), 9), 
        dp.symmetric_distance())
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    measurement = dp.m.make_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=1.5)
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    data = [1, 2, 3, 4, 5]
    count = dp.t.make_count(dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    print("count:", count(data))

    base_dl = count.output_space >> dp.m.then_laplace(scale=0.5)
    print("base_dl:", base_dl(1))

    chain = count >> base_dl
    print("chained measurement check:", chain.check(d_in=1, d_out=1000., debug=True))

    print("evaluate chain:", chain(data))

def test_bisect():
    from opendp.mod import binary_search

    for i in range(100):
        assert binary_search(lambda x: x < i + 1, (0, 100)) == i
        assert binary_search(lambda x: x > i, (0, 100)) == i + 1

        assert -(binary_search(lambda x: x < i + 1, (0., 100.)) - (i + 1)) < 1e-8
        assert binary_search(lambda x: x > i, (0., 100.)) - i < 1e-8


def test_bisect_edge():
    from opendp.mod import binary_search
    with pytest.raises(ValueError):
        binary_search(lambda x: x > 5., (0., 5.))
    assert binary_search(lambda x: x > 0, (0, 1)) == 1
    assert binary_search(lambda x: x < 1, (0, 1)) == 0
    with pytest.raises(ValueError):
        binary_search(lambda x: x < 1, (0, 0))

    assert binary_search(lambda x: x > 5, bounds=(0, 10)) == 6
    assert binary_search(lambda x: x < 5, bounds=(0, 10)) == 4
    assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. < 1e-8
    assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. > -1e-8


def test_bisect_chain():
    pre = (
        (dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()) >>
        dp.t.then_clamp(bounds=(0., 1.)) >>
        dp.t.then_resize(size=10, constant=0.) >>
        dp.t.then_mean()
    )
    chain = dp.binary_search_chain(lambda s: pre >> dp.m.then_laplace(scale=s), d_in=1, d_out=1.)
    assert chain.check(1, 1.)

    scale = dp.binary_search_param(lambda s: pre >> dp.m.then_laplace(scale=s), d_in=1, d_out=1.)
    assert scale - 0.1 < 1e-8


def test_supporting_elements():
    from opendp.transformations import make_clamp
    from opendp.domains import atom_domain, vector_domain
    from opendp.metrics import symmetric_distance

    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()

    clamper = make_clamp(input_domain, input_metric, (0, 2))
    print(clamper.input_domain)
    print(clamper.input_domain.carrier_type)
    print(clamper.output_domain)
    print(clamper.output_domain.carrier_type)
    print(clamper.input_metric)
    print(clamper.input_metric.distance_type)
    print(clamper.output_metric)
    print(clamper.output_metric.distance_type)

    from opendp.measurements import make_base_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    mechanism = make_base_laplace(atom_domain(T=float), absolute_distance(T=float), 1.)
    print(mechanism.input_domain)
    print(mechanism.input_domain.carrier_type)
    print(mechanism.input_metric)
    print(mechanism.input_metric.distance_type)
    print(mechanism.output_measure)
    print(mechanism.output_measure.distance_type)


def test_function():
    from opendp.measurements import make_base_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    mechanism = make_base_laplace(atom_domain(T=float), absolute_distance(T=float), 1.)
    pow = 4 # add noise 2^pow times
    for _ in range(pow):
        mechanism = mechanism >> mechanism.function
    print(mechanism(0.))



def test_member():
    from opendp.domains import atom_domain, vector_domain
    from opendp.metrics import symmetric_distance
    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()

    from opendp.transformations import make_clamp
    clamper = make_clamp(input_domain, input_metric, (0, 2))
    assert clamper.input_domain.member([1])
    assert not clamper.output_domain.member([4, 1])

    from opendp.measurements import make_base_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    mechanism = make_base_laplace(atom_domain(T=float), absolute_distance(T=float), 1.)
    assert not mechanism.input_domain.member(float("NaN"))

def test_new_domain():
    from opendp.domains import atom_domain, vector_domain
    domain = atom_domain(T=dp.i32)
    assert domain.member(3)
    domain = atom_domain(T=dp.f64)
    assert not domain.member(float("nan"))

    domain = atom_domain((1, 2))
    assert domain.member(2)
    assert not domain.member(3)
    print(domain)

    domain = vector_domain(atom_domain(T=dp.i32))
    assert domain.member([2])
    print(domain)
    domain = vector_domain(atom_domain((2, 3)))
    assert domain.member([2])
    assert not domain.member([2, 4])
    print(domain)
    
    domain = vector_domain(atom_domain(T=dp.i32), 10)
    assert domain.member([1] * 10)
    print(domain)
    domain = vector_domain(atom_domain((2., 7.)), 10)
    assert domain.member([3.] * 10)
    assert not domain.member([1.] * 10)
    print(domain)

    null_domain = atom_domain(nullable=True, T=float)
    print(null_domain)
    assert null_domain.member(float("nan"))

    not_null_domain = atom_domain(nullable=False, T=float)
    print(not_null_domain)
    assert not not_null_domain.member(float("nan"))
