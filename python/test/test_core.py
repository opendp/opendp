from opendp.v1.typing import HammingDistance, L1Distance


def test_type_getters():
    from opendp.v1.trans import make_bounded_mean
    transformation = make_bounded_mean(lower=0., upper=10., n=9, MI=HammingDistance, T=float)
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    from opendp.v1.meas import make_base_geometric
    measurement = make_base_geometric(scale=1.5)
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    from opendp.v1.trans import make_count
    from opendp.v1.meas import make_base_laplace, make_base_geometric

    data = [1, 2, 3, 4, 5]
    count = make_count(MI=HammingDistance, TI=int, TO=int)
    print("count:", count(data))

    base_laplace = make_base_laplace(scale=1.)
    print("base laplace:", base_laplace(10.))

    base_geometric = make_base_geometric(scale=0.5)
    print("base_geometric:", base_geometric(1))

    chain = count >> base_geometric
    print("chained measurement check:", chain.check(d_in=1, d_out=1000., debug=True))

    print("evaluate chain:", chain(data))
