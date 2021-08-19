import pytest

from opendp.mod import enable_features
enable_features('floating-point')


def test_type_getters():
    from opendp.trans import make_bounded_mean
    transformation = make_bounded_mean(lower=0., upper=10., n=9, T=float)
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    from opendp.meas import make_base_geometric
    measurement = make_base_geometric(scale=1.5)
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    from opendp.trans import make_count
    from opendp.meas import make_base_laplace, make_base_geometric
    enable_features("floating-point")

    data = [1, 2, 3, 4, 5]
    count = make_count(TIA=int, TO=int)
    print("count:", count(data))

    base_laplace = make_base_laplace(scale=1.)
    print("base laplace:", base_laplace(10.))

    base_geometric = make_base_geometric(scale=0.5)
    print("base_geometric:", base_geometric(1))

    chain = count >> base_geometric
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


def test_bisect_chain():
    from opendp.mod import binary_search, binary_search_chain, binary_search_param
    from opendp.trans import make_clamp, make_resize_bounded, make_bounded_mean
    from opendp.meas import make_base_laplace
    pre = (
        # make_clamp(lower=0., upper=1.) >>
        make_resize_bounded(constant=0., length=10, lower=0., upper=1.) >>
        make_bounded_mean(lower=0., upper=1., n=10)
    )
    # chain = bisect_chain(
    #     lambda s: pre >> make_base_laplace(scale=s),
    #     bounds=(0., 10.), d_in=1, d_out=1.)
    # assert chain.check(1, 1.)

    # scale = bisect_param(
    #     lambda s: pre >> make_base_laplace(scale=s),
    #     bounds=(0., 10.), d_in=1, d_out=1.)

    print(binary_search(lambda s: (pre >> make_base_laplace(scale=s)).check(1, 1.), (0., 10.)))
    print((pre >> make_base_laplace(scale=1.25)).check(1, 1.))
    print((pre >> make_base_laplace(scale=0.1)).check(1, 1.))

    # print((pre >> make_base_laplace(scale=0.1)).check(2, 1.))
    # print(scale)


# test_bisect_chain()
test_bisect()
