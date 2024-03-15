import pytest

import opendp.prelude as dp

dp.enable_features("floating-point", "contrib", "honest-but-curious")


def test_type_getters():
    transformation = dp.t.make_mean(
        dp.vector_domain(dp.atom_domain((0.0, 10.0)), 9), dp.symmetric_distance()
    )
    assert transformation.input_distance_type == "u32"
    assert transformation.output_distance_type == "f64"
    assert transformation.input_carrier_type == "Vec<f64>"

    measurement = dp.m.make_laplace(
        dp.atom_domain(T=int), dp.absolute_distance(T=int), scale=1.5
    )
    assert measurement.input_distance_type == "i32"
    assert measurement.output_distance_type == "f64"
    assert measurement.input_carrier_type == "i32"


def test_chain():
    data = [1, 2, 3, 4, 5]
    count = dp.t.make_count(
        dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    )
    print("count:", count(data))

    base_dl = count.output_space >> dp.m.then_laplace(scale=0.5)
    print("base_dl:", base_dl(1))

    chain = count >> base_dl
    print("chained measurement check:", chain.check(d_in=1, d_out=1000.0, debug=True))

    print("evaluate chain:", chain(data))


def test_bisect():
    from opendp.mod import binary_search

    for i in range(100):
        assert binary_search(lambda x: x < i + 1, (0, 100)) == i
        assert binary_search(lambda x: x > i, (0, 100)) == i + 1

        assert -(binary_search(lambda x: x < i + 1, (0.0, 100.0)) - (i + 1)) < 1e-8
        assert binary_search(lambda x: x > i, (0.0, 100.0)) - i < 1e-8


def test_bisect_edge():
    from opendp.mod import binary_search

    with pytest.raises(ValueError):
        binary_search(lambda x: x > 5.0, (0.0, 5.0))
    assert binary_search(lambda x: x > 0, (0, 1)) == 1
    assert binary_search(lambda x: x < 1, (0, 1)) == 0
    with pytest.raises(ValueError):
        binary_search(lambda x: x < 1, (0, 0))

    assert binary_search(lambda x: x > 5, bounds=(0, 10)) == 6
    assert binary_search(lambda x: x < 5, bounds=(0, 10)) == 4
    assert binary_search(lambda x: x > 5.0, bounds=(0.0, 10.0)) - 5.0 < 1e-8
    assert binary_search(lambda x: x > 5.0, bounds=(0.0, 10.0)) - 5.0 > -1e-8


def test_type_hinting():
    from opendp.mod import binary_search
    assert binary_search(lambda x: x > 0, (0, 1), int, True)[0] == 1
    assert binary_search(lambda x: x > 0, (0, 1), T=int, return_sign=True)[0] == 1
    assert binary_search(lambda x: x > 0, bounds=(0, 1), return_sign=True)[0] == 1
    assert binary_search(lambda x: x > 0, return_sign=True)[0] == 5e-324


def test_bisect_chain():
    pre = (
        (dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance())
        >> dp.t.then_clamp(bounds=(0.0, 1.0))
        >> dp.t.then_resize(size=10, constant=0.0)
        >> dp.t.then_mean()
    )
    chain = dp.binary_search_chain(
        lambda s: pre >> dp.m.then_laplace(scale=s), d_in=1, d_out=1.0
    )
    assert chain.check(1, 1.0)

    scale = dp.binary_search_param(
        lambda s: pre >> dp.m.then_laplace(scale=s), d_in=1, d_out=1.0
    )
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

    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance

    mechanism = make_laplace(atom_domain(T=float), absolute_distance(T=float), 1.0)
    print(mechanism.input_domain)
    print(mechanism.input_domain.carrier_type)
    print(mechanism.input_metric)
    print(mechanism.input_metric.distance_type)
    print(mechanism.output_measure)
    print(mechanism.output_measure.distance_type)


def test_function():
    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    from opendp.transformations import make_identity

    mechanism = make_laplace(atom_domain(T=float), absolute_distance(T=float), 1.0)
    pow = 4  # add noise 2^pow times
    for _ in range(pow):
        mechanism = mechanism >> mechanism.function

    # Exercise postprocessing transformation
    transformation = make_identity(atom_domain(T=float), absolute_distance(T=float))
    mechanism = mechanism >> transformation
    print(mechanism(0.0))


def test_member():
    from opendp.domains import atom_domain, vector_domain
    from opendp.metrics import symmetric_distance

    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()

    from opendp.transformations import make_clamp

    clamper = make_clamp(input_domain, input_metric, (0, 2))
    assert clamper.input_domain.member([1])
    assert not clamper.output_domain.member([4, 1])

    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance

    mechanism = make_laplace(atom_domain(T=float), absolute_distance(T=float), 1.0)
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
    domain = vector_domain(atom_domain((2.0, 7.0)), 10)
    assert domain.member([3.0] * 10)
    assert not domain.member([1.0] * 10)
    print(domain)

    null_domain = atom_domain(nullable=True, T=float)
    print(null_domain)
    assert null_domain.member(float("nan"))

    not_null_domain = atom_domain(nullable=False, T=float)
    print(not_null_domain)
    assert not not_null_domain.member(float("nan"))


def test_user_domain():
    from datetime import datetime

    def datetime_domain(months):
        """The domain of datetimes, restricted by user-defined months"""
        assert isinstance(months, set)
        return dp.user_domain(
            identifier=f"DatetimeDomain(months={months})",
            member=lambda x: isinstance(x, datetime) and x.month in months,
            descriptor=months,
        )

    domain = datetime_domain(months={1, 2, 3, 4})
    assert str(domain).startswith("DatetimeDomain")

    element = datetime.strptime("03/17/20 4:32:34", "%m/%d/%y %H:%M:%S")
    assert domain.member(element)
    assert not domain.member("A")

    # MEMORY CHECK: try to access data which would have fallen out-of-scope and been freed
    import gc

    gc.collect() # if refcount is incorrect, then accessing .descriptor will trigger a use-after-free

    # can retrieve the descriptor for use in further analysis
    assert domain.descriptor == {1, 2, 3, 4}

    # nest inside a vector domain
    vec_domain = dp.vector_domain(domain)
    assert vec_domain.member([datetime.now()])
    trans = dp.t.make_identity(vec_domain, dp.symmetric_distance())
    misc_data = [1, datetime.now(), "abc", 1j + 2]
    assert trans(misc_data) == misc_data
    assert not vec_domain.member(misc_data)

    # nest inside a hashmap domain
    map_domain = dp.map_domain(dp.atom_domain(T=str), domain)
    assert map_domain.member({"A": datetime.now(), "B": datetime.now()})
    trans = dp.t.make_identity(map_domain, dp.symmetric_distance())
    misc_data = {"A": datetime.now(), "C": 1j + 2}  # type: ignore[assignment]
    assert trans(misc_data) == misc_data
    assert not map_domain.member(misc_data)


def test_extrinsic_free():
    space = dp.user_domain("anything", lambda _: True), dp.symmetric_distance()

    sc_meas = space >> dp.c.then_sequential_composition(
        dp.max_divergence(T=float),
        d_in=1,
        d_mids=[1.0],
    )

    # pass in something that gets a new id(), so as to have a refcount that can drop to zero
    qbl = sc_meas([])
    # at this point []'s refcount is zero, but has not been freed yet, because the gc has not run
    # however, a pointer to [] is stored inside qbl

    query = space >> dp.m.then_user_measurement(
        dp.max_divergence(T=float),
        lambda x: x,
        lambda _: 0.0,
    )

    import gc

    # frees the memory behind [] (if the refcount is zero)
    gc.collect()

    # use-after-free
    qbl(query)

    # this test will pass if Queryable extends the lifetime of [] by holding a reference to it


def test_user_distance():
    from datetime import datetime, timedelta

    # create custom transformation
    trans = dp.t.make_user_transformation(
        dp.vector_domain(dp.user_domain("DatetimeDomain()", lambda x: isinstance(x, datetime))),
        dp.user_distance("sum of millisecond distances"),
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        lambda arg: sum(datetime.timestamp(x) for x in arg),
        lambda d_in: d_in.total_seconds() * 1000,
    )

    data = [datetime.now(), datetime.now()]
    assert trans(data) == sum(datetime.timestamp(x) for x in data)

    d_in = timedelta(days=2.4, seconds=45.2)
    assert trans.map(d_in) == d_in.total_seconds() * 1000

    # create custom measurement
    meas = dp.m.make_user_measurement(
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        dp.user_divergence("tCDP"),
        lambda _: 0.0,
        # clearly not actually tCDP
        lambda d_in: lambda omega: d_in * omega * 2,
    )

    assert meas(2.0) == 0.0
    assert meas.map(2.0)(3.0) == 12.0
