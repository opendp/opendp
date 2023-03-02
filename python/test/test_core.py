import pytest

from opendp.typing import *
from opendp.mod import enable_features
enable_features('floating-point', 'contrib')


def test_type_getters():
    from opendp.transformations import make_sized_bounded_mean
    transformation = make_sized_bounded_mean(size=9, bounds=(0., 10.), T=float)
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    from opendp.measurements import make_base_discrete_laplace
    measurement = make_base_discrete_laplace(scale=1.5)
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    from opendp.transformations import make_count
    from opendp.measurements import make_base_laplace, make_base_discrete_laplace
    enable_features("floating-point", "contrib")

    data = [1, 2, 3, 4, 5]
    count = make_count(TIA=int, TO=int)
    print("count:", count(data))

    base_laplace = make_base_laplace(scale=1.)
    print("base laplace:", base_laplace(10.))

    base_dl = make_base_discrete_laplace(scale=0.5)
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
    from opendp.mod import binary_search_chain, binary_search_param, enable_features
    from opendp.transformations import make_clamp, make_bounded_resize, make_sized_bounded_mean
    from opendp.measurements import make_base_laplace
    enable_features("contrib")

    pre = (
        make_clamp(bounds=(0., 1.)) >>
        make_bounded_resize(size=10, bounds=(0., 1.), constant=0.) >>
        make_sized_bounded_mean(size=10, bounds=(0., 1.))
    )
    chain = binary_search_chain(lambda s: pre >> make_base_laplace(scale=s), d_in=1, d_out=1.)
    assert chain.check(1, 1.)

    scale = binary_search_param(lambda s: pre >> make_base_laplace(scale=s), d_in=1, d_out=1.)
    assert scale - 0.1 < 1e-8


def test_supporting_elements():
    from opendp.transformations import make_clamp
    clamper = make_clamp((0, 2))
    print(clamper.input_domain)
    print(clamper.input_domain.carrier_type)
    print(clamper.output_domain)
    print(clamper.output_domain.carrier_type)
    print(clamper.input_metric)
    print(clamper.input_metric.distance_type)
    print(clamper.output_metric)
    print(clamper.output_metric.distance_type)

    from opendp.measurements import make_base_laplace
    mechanism = make_base_laplace(1.)
    print(mechanism.input_domain)
    print(mechanism.input_domain.carrier_type)
    print(mechanism.input_metric)
    print(mechanism.input_metric.distance_type)
    print(mechanism.output_measure)
    print(mechanism.output_measure.distance_type)


def test_member():
    from opendp.transformations import make_clamp
    clamper = make_clamp((0, 2))
    assert clamper.input_domain.member([1])
    assert not clamper.output_domain.member([4, 1])

    from opendp.measurements import make_base_laplace
    mechanism = make_base_laplace(1.)
    assert not mechanism.input_domain.member(float("NaN"))

def test_new_domain():
    from opendp.domains import all_domain, bounded_domain, vector_domain, sized_domain
    domain = all_domain(i32)
    assert domain.member(3)
    domain = all_domain(f64)
    assert not domain.member(float("nan"))
    print(domain)

    domain = bounded_domain((1, 2))
    assert domain.member(2)
    assert not domain.member(3)
    print(domain)

    domain = vector_domain(all_domain(i32))
    assert domain.member([2])
    print(domain)
    domain = vector_domain(bounded_domain((2, 3), i32))
    assert domain.member([2])
    assert not domain.member([2, 4])
    print(domain)
    
    domain = sized_domain(vector_domain(all_domain(i32)), 10)
    assert domain.member([1] * 10)
    print(domain)
    domain = sized_domain(vector_domain(bounded_domain((2., 7.))), 10)
    assert domain.member([3.] * 10)
    assert not domain.member([1.] * 10)
    print(domain)
