import math

from opendp.mod import PrivacyCurve
import pytest

import opendp.prelude as dp
from opendp._internal import _extrinsic_domain, _extrinsic_distance, _extrinsic_divergence, _new_pure_function

from .helpers import ids

def test_version():
    assert dp.__version__.startswith('0.')


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
        (dp.vector_domain(dp.atom_domain(T=float, nan=False)), dp.symmetric_distance())
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
    assert str(clamper.input_domain) == 'VectorDomain(AtomDomain(T=i32))'
    assert str(clamper.input_domain.carrier_type) == 'Vec<i32>'
    assert str(clamper.output_domain) == 'VectorDomain(AtomDomain(bounds=[0, 2], T=i32))'
    assert str(clamper.output_domain.carrier_type) == 'Vec<i32>'
    assert str(clamper.input_metric) == 'SymmetricDistance()'
    assert str(clamper.input_metric.distance_type) == 'u32'
    assert str(clamper.output_metric) == 'SymmetricDistance()'
    assert str(clamper.output_metric.distance_type) == 'u32'

    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance

    mechanism = make_laplace(atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0)
    assert str(mechanism.input_domain) == 'AtomDomain(T=f64)'
    assert str(mechanism.input_domain.carrier_type) == 'f64'
    assert str(mechanism.input_metric) == 'AbsoluteDistance(f64)'
    assert str(mechanism.input_metric.distance_type) == 'f64'
    assert str(mechanism.output_measure) == 'MaxDivergence'
    assert str(mechanism.output_measure.distance_type) == 'f64'


def test_function():
    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    from opendp.transformations import make_identity

    mechanism = make_laplace(atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0)
    pow = 4  # add noise 2^pow times
    for _ in range(pow):
        mechanism = mechanism >> mechanism.function

    # Exercise postprocessing transformation
    transformation = make_identity(atom_domain(T=float, nan=False), absolute_distance(T=float))
    mechanism = mechanism >> transformation
    print("mechanism(0.0)", mechanism(0.0))


def test_privacy_profile():
    import math
    with pytest.warns():
        profile = dp.new_privacy_profile(lambda eps: math.exp(-eps))
    # formula is -ln(1e-7)
    assert profile.epsilon(delta=1e-7) == 16.11809565095832

def assert_unit_interval(value):
    assert 0.0 <= value <= 1.0


def exact_approxdp_beta(pairs, alpha):
    """Exact API beta(alpha) induced by approximate-DP pairs."""
    if alpha == 0.0:
        return 1.0
    if alpha == 1.0:
        return 0.0

    candidates = [0.0]

    for epsilon, delta in pairs:
        candidates.append(1.0 - delta - math.exp(epsilon) * alpha)

        base = max(1.0 - delta - alpha, 0.0)
        candidates.append(math.exp(-epsilon) * base)

    return max(candidates)


def test_profile_constructor_delta_and_epsilon():
    curve = PrivacyCurve(profile=lambda eps: math.exp(-eps))

    assert curve.delta(0.0) == pytest.approx(1.0)
    assert curve.delta(1.0) == pytest.approx(math.exp(-1.0))

    # Solves exp(-epsilon) <= 1e-7.
    assert curve.epsilon(1e-7) == pytest.approx(
        -math.log(1e-7),
        rel=0.0,
        abs=1e-12,
    )


def test_profile_constructor_beta_is_in_range():
    curve = PrivacyCurve(profile=lambda eps: math.exp(-eps))

    assert_unit_interval(curve.beta(0.0))
    assert_unit_interval(curve.beta(0.25))
    assert_unit_interval(curve.beta(0.5))
    assert_unit_interval(curve.beta(1.0))

    assert curve.beta(0.0) == 1.0
    assert curve.beta(1.0) == 0.0


def test_log_profile_constructor_delta_and_epsilon():
    curve = PrivacyCurve(log_profile=lambda eps: -eps)

    assert curve.delta(0.0) == pytest.approx(1.0)
    assert curve.delta(1.0) == pytest.approx(math.exp(-1.0))

    assert curve.epsilon(1e-7) == pytest.approx(
        -math.log(1e-7),
        rel=0.0,
        abs=1e-12,
    )


def test_approxdp_constructor_delta_and_epsilon():
    pairs = [(1.0, 0.1), (2.0, 0.01)]
    curve = PrivacyCurve(approxDP=pairs)

    assert curve.delta(1.0) == pytest.approx(0.1)
    assert curve.delta(2.0) == pytest.approx(0.01)

    assert curve.epsilon(0.1) == pytest.approx(1.0)
    assert curve.epsilon(0.01) == pytest.approx(2.0)


def test_approxdp_constructor_beta_is_conservative_and_tight():
    pairs = [(1.0, 0.1), (2.0, 0.01)]
    curve = PrivacyCurve(approxDP=pairs)

    for alpha in [0.0, 0.1, 0.5, 0.9, 1.0]:
        beta = curve.beta(alpha)
        expected = exact_approxdp_beta(pairs, alpha)

        assert_unit_interval(beta)

        # beta(alpha) should be downward-conservative.
        assert beta <= expected

        # But not meaningfully loose.
        assert beta == pytest.approx(expected, rel=0.0, abs=1e-14)


def test_tradeoff_constructor_beta_and_delta():
    curve = PrivacyCurve(tradeoff=lambda alpha: 1.0 - alpha)

    assert curve.beta(0.0) == 1.0
    assert curve.beta(0.3) == pytest.approx(0.7)
    assert curve.beta(1.0) == 0.0

    # beta(alpha) = 1 - alpha is the perfect-privacy tradeoff.
    assert curve.delta(0.0) == pytest.approx(0.0)
    assert curve.delta(1.0) == pytest.approx(0.0)


def test_symmetric_tradeoff_constructor_alpha_uses_symmetry():
    curve = PrivacyCurve(symmetric_tradeoff=lambda alpha: 1.0 - alpha)

    assert curve.alpha(0.0) == 1.0
    assert curve.alpha(0.25) == pytest.approx(0.75)
    assert curve.alpha(1.0) == 0.0


def test_nonsymmetric_tradeoff_alpha_inverts_beta_not_calls_beta():
    # Valid-looking non-symmetric decreasing convex tradeoff:
    # beta(alpha) = (1 - alpha)^2.
    curve = PrivacyCurve(tradeoff=lambda alpha: (1.0 - alpha) ** 2)

    # alpha(beta) should solve beta = (1 - alpha)^2.
    # For beta = 0.25, alpha = 0.5.
    #
    # This catches an accidental implementation that calls beta(beta),
    # which would return 0.5625 instead.
    assert curve.alpha(0.25) == pytest.approx(0.5, abs=1e-12)


def test_gdp_constructor_basic_queries():
    curve = PrivacyCurve(gaussianDP=1.0)

    assert curve.beta(0.0) == 1.0
    assert curve.beta(1.0) == 0.0

    assert_unit_interval(curve.beta(0.5))
    assert_unit_interval(curve.alpha(0.5))
    assert_unit_interval(curve.delta(0.5))


def test_zcdp_constructor_basic_queries():
    curve = PrivacyCurve(zCDP=0.5)

    assert curve.beta(0.0) == 1.0
    assert curve.beta(1.0) == 0.0
    assert_unit_interval(curve.beta(0.5))
    assert_unit_interval(curve.delta(1.0))


def test_renyidp_constructor_basic_queries():
    curve = PrivacyCurve(renyiDP=lambda alpha: 0.5 * alpha)

    assert curve.beta(0.0) == 1.0
    assert curve.beta(1.0) == 0.0
    assert_unit_interval(curve.beta(0.5))
    assert_unit_interval(curve.delta(1.0))


def test_multi_representation_constructor_prefers_profile_for_delta():
    curve = PrivacyCurve(
        profile=lambda eps: math.exp(-eps),
        tradeoff=lambda alpha: 1.0 - alpha,
    )

    assert curve.delta(1.0) == pytest.approx(math.exp(-1.0))


@pytest.mark.parametrize("epsilon", [-1.0, float("nan")])
def test_delta_rejects_invalid_epsilon(epsilon):
    curve = PrivacyCurve(tradeoff=lambda alpha: 1.0 - alpha)

    with pytest.raises(Exception):
        curve.delta(epsilon)


@pytest.mark.parametrize("delta", [-0.1, 1.1, float("nan")])
def test_epsilon_rejects_invalid_delta(delta):
    curve = PrivacyCurve(tradeoff=lambda alpha: 1.0 - alpha)

    with pytest.raises(Exception):
        curve.epsilon(delta)


@pytest.mark.parametrize("alpha", [-0.1, 1.1, float("nan")])
def test_beta_rejects_invalid_alpha(alpha):
    curve = PrivacyCurve(tradeoff=lambda alpha: 1.0 - alpha)

    with pytest.raises(Exception):
        curve.beta(alpha)


@pytest.mark.parametrize("beta", [-0.1, 1.1, float("nan")])
def test_alpha_rejects_invalid_beta(beta):
    curve = PrivacyCurve(tradeoff=lambda alpha: 1.0 - alpha)

    with pytest.raises(Exception):
        curve.alpha(beta)


def test_member():
    from opendp.domains import atom_domain, vector_domain
    from opendp.metrics import symmetric_distance

    int_10_domain = atom_domain(T=int, bounds=(0, 10))
    assert int_10_domain.member(0)
    assert not int_10_domain.member(100)
    with pytest.warns(UserWarning, match=r'inferred type is f64, expected i32'):
        assert not int_10_domain.member(0.0)
    with pytest.warns(UserWarning, match=r'inferred type is f64, expected i32'):
        assert not int_10_domain.member(100.0)

    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()

    from opendp.transformations import make_clamp

    clamper = make_clamp(input_domain, input_metric, (0, 2))
    assert clamper.input_domain.member([1])
    assert not clamper.output_domain.member([4, 1])

    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance

    mechanism = make_laplace(atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0)
    assert not mechanism.input_domain.member(float("NaN"))


def test_new_domain():
    from opendp.domains import atom_domain, vector_domain

    domain = atom_domain(T=dp.i32)
    assert domain.member(3)
    assert str(domain) == 'AtomDomain(T=i32)'

    domain = atom_domain(T=dp.f64, nan=False)
    assert not domain.member(float("nan"))
    assert str(domain) == 'AtomDomain(T=f64)'

    domain = atom_domain((1, 2))
    assert domain.member(2)
    assert not domain.member(3)
    assert str(domain) == 'AtomDomain(bounds=[1, 2], T=i32)'

    domain = vector_domain(atom_domain(T=dp.i32))
    assert domain.member([2])
    assert str(domain) == 'VectorDomain(AtomDomain(T=i32))'

    domain = vector_domain(atom_domain((2, 3)))
    assert domain.member([2])
    assert not domain.member([2, 4])
    assert str(domain) == 'VectorDomain(AtomDomain(bounds=[2, 3], T=i32))'

    domain = vector_domain(atom_domain(T=dp.i32), 10)
    assert domain.member([1] * 10)
    assert str(domain) == 'VectorDomain(AtomDomain(T=i32), size=10)'

    domain = vector_domain(atom_domain((2.0, 7.0)), 10)
    assert domain.member([3.0] * 10)
    assert not domain.member([1.0] * 10)
    assert str(domain) == 'VectorDomain(AtomDomain(bounds=[2.0, 7.0], nan=true, T=f64), size=10)'

    null_domain = atom_domain(nan=True, T=float)
    assert str(null_domain) == 'AtomDomain(nan=true, T=f64)'
    assert null_domain.member(float("nan"))

    not_null_domain = atom_domain(nan=False, T=float)
    assert str(not_null_domain) == 'AtomDomain(T=f64)'
    assert not not_null_domain.member(float("nan"))


@pytest.mark.parametrize(
    "struct",
    [
        dp.m.make_user_measurement(
            dp.atom_domain(T=int),
            dp.absolute_distance(T=int),
            dp.max_divergence(),
            lambda x: x,
            lambda _: 0.0,
        ), 
        dp.t.make_user_transformation(
            dp.atom_domain(T=int),
            dp.absolute_distance(T=int),
            dp.atom_domain(T=int),
            dp.absolute_distance(T=int),
            lambda x: x,
            lambda _: 0.0,
        ),
        dp.c.make_fully_adaptive_composition(
            dp.atom_domain(T=int),
            dp.absolute_distance(T=int),
            dp.max_divergence(),
        ),
        dp.user_distance(""),
        dp.user_divergence(""),
        dp.user_domain("", lambda _: True),
        dp.new_function(lambda x: x, TO="i32"),
    ],
    ids=ids,
)
def test_struct_iter(struct):
    with pytest.raises(ValueError):
        [*struct]

@pytest.mark.parametrize("new_domain", [dp.user_domain, _extrinsic_domain])
def test_custom_domain(new_domain):
    from datetime import datetime

    def datetime_domain(months):
        """The domain of datetimes, restricted by user-defined months"""
        assert isinstance(months, set)
        return new_domain(
            identifier=f"DatetimeDomain(months={months})",
            member=lambda x: isinstance(x, datetime) and x.month in months,
            descriptor=months,
        )

    domain = datetime_domain(months={1, 2, 3, 4})
    assert str(domain).startswith("DatetimeDomain")
    assert isinstance(domain, dp.ExtrinsicDomain)

    element = datetime.strptime("03/17/20 4:32:34", "%m/%d/%y %H:%M:%S")
    assert domain.member(element)
    assert not domain.member("A")

    # MEMORY CHECK: try to access data which would have fallen out-of-scope and been freed
    import gc

    gc.collect() # if refcount is incorrect, then accessing .descriptor will trigger a use-after-free

    # can retrieve the descriptor for use in further analysis
    assert domain.descriptor == {1, 2, 3, 4}
    # or retrieve the descriptor with type-checking
    assert domain.cast(set) == {1, 2, 3, 4}

    with pytest.raises(ValueError, match="domain descriptor must be a int"):
        assert domain.cast(int)

    # nest inside a vector domain
    vec_domain = dp.vector_domain(domain)
    january_1 = datetime.fromisoformat('2024-01-01')
    assert vec_domain.member([january_1])
    trans = dp.t.make_identity(vec_domain, dp.symmetric_distance())
    misc_data = [1, january_1, "abc", 1j + 2]
    assert trans(misc_data) == misc_data
    assert not vec_domain.member(misc_data)

    # nest inside a hashmap domain
    map_domain = dp.map_domain(dp.atom_domain(T=str), domain)
    assert map_domain.member({"A": january_1, "B": january_1})
    trans = dp.t.make_identity(map_domain, dp.symmetric_distance())
    misc_data = {"A": january_1, "C": 1j + 2}  # type: ignore[assignment]
    assert trans(misc_data) == misc_data
    assert not map_domain.member(misc_data)


def test_extrinsic_free():
    space = dp.user_domain("anything", lambda _: True), dp.symmetric_distance()
    query = space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: x,
        lambda _: 0.0,
    )

    sc_meas = space >> dp.c.then_adaptive_composition(
        dp.max_divergence(),
        d_in=1,
        d_mids=[1.0],
    )

    # pass in something that gets a new id(), so as to have a refcount that can drop to zero
    qbl = sc_meas([])
    # at this point []'s refcount is zero, but has not been freed yet, because the gc has not run
    # however, a pointer to [] is stored inside qbl

    import gc

    # frees the memory behind [] (if the refcount is zero)
    gc.collect()

    # use-after-free
    qbl(query)

    # this test will pass if Queryable extends the lifetime of [] by holding a reference to it


@pytest.mark.parametrize("new_distance,new_divergence", zip([dp.user_distance, _extrinsic_distance], [dp.user_divergence, _extrinsic_divergence]))
def test_custom_distance(new_distance, new_divergence):
    from datetime import datetime, timedelta

    # create custom transformation
    trans = dp.t.make_user_transformation(
        dp.vector_domain(dp.user_domain("DatetimeDomain()", lambda x: isinstance(x, datetime))),
        new_distance("sum of millisecond distances"),
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        lambda arg: sum(datetime.timestamp(x) for x in arg),
        lambda d_in: d_in.total_seconds() * 1000,
    )

    january_1 = datetime.fromisoformat('2024-01-01')
    data = [january_1, january_1]
    assert trans(data) == sum(datetime.timestamp(x) for x in data)

    d_in = timedelta(days=2.4, seconds=45.2)
    assert trans.map(d_in) == d_in.total_seconds() * 1000

    # create custom measurement
    meas = dp.m.make_user_measurement(
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        new_divergence("tCDP", {"kind": "tCDP"}),
        lambda _: 0.0,
        # clearly not actually tCDP
        lambda d_in: lambda omega: d_in * omega * 2,
    )

    assert meas(2.0) == 0.0
    assert meas.map(2.0)(3.0) == 12.0
    assert isinstance(meas.output_measure, dp.ExtrinsicDivergence)
    assert meas.output_measure.descriptor == {"kind": "tCDP"}
    assert meas.output_measure.cast(dict) == {"kind": "tCDP"}
    with pytest.raises(ValueError, match=r"measure descriptor must be a list"):
        meas.output_measure.cast(list)


def test_pure_function():
    fun = _new_pure_function(lambda x: x + 1, TO="i32")
    assert fun(1) == 2


def test_np_array_postprocessor():
    np = pytest.importorskip("numpy")
    fun = dp.as_array()

    result = fun(np.array([1, 2, 3], dtype=np.int32))

    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([1, 2, 3], dtype=np.int32))


def test_pointer_classes_dont_iter():
    import opendp.prelude as dp

    # since pointer classes like Domain, Transformation, etc. inherit from ctypes.POINTER,
    # __iter__ is inherited and attempts to unpack the data behind the pointer 
    # as if it were a pointer to an array of structs.

    # However, structs from OpenDP are opaque, so are zero-sized.
    # Python will infinitely yield the data directly behind the pointer, 
    # stepping forward by zero bytes each time.

    # We override __iter__ so as to make this infinite loop/lock impossible to accidentally trigger
    with pytest.raises(ValueError):
        [*dp.atom_domain(T=bool)]


def test_erfc():
    from opendp._data import erfc
    assert erfc(0.5) == 0.4795001222363462
