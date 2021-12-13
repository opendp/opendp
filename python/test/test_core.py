import pytest

from opendp.mod import enable_features
enable_features('floating-point', 'contrib')


def test_type_getters():
    from opendp.trans import make_sized_bounded_mean
    transformation = make_sized_bounded_mean(size=9, bounds=(0., 10.), T=float)
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    from opendp.meas import make_base_discrete_laplace
    measurement = make_base_discrete_laplace(scale=1.5)
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    from opendp.trans import make_count
    from opendp.meas import make_base_laplace, make_base_discrete_laplace
    enable_features("floating-point", "contrib")

    data = [1, 2, 3, 4, 5]
    count = make_count(TIA=int, TO=int)
    print("count:", count(data))

    base_laplace = make_base_laplace(scale=1.)
    print("base laplace:", base_laplace(10.))

    base_discrete_laplace = make_base_discrete_laplace(scale=0.5)
    print("base_discrete_laplace:", base_discrete_laplace(1))

    chain = count >> base_discrete_laplace
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
    with pytest.raises(AssertionError):
        binary_search(lambda x: x > 5., (0., 5.))
    assert binary_search(lambda x: x > 0, (0, 1)) == 1
    assert binary_search(lambda x: x < 1, (0, 1)) == 0
    with pytest.raises(AssertionError):
        binary_search(lambda x: x < 1, (0, 0))

    assert binary_search(lambda x: x > 5, bounds=(0, 10)) == 6
    assert binary_search(lambda x: x < 5, bounds=(0, 10)) == 4
    assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. < 1e-8
    assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. > -1e-8


def test_bisect_chain():
    from opendp.mod import binary_search_chain, binary_search_param, enable_features
    from opendp.trans import make_clamp, make_bounded_resize, make_sized_bounded_mean
    from opendp.meas import make_base_laplace
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
