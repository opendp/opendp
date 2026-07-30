import pytest

import opendp.prelude as dp
from opendp._internal import (
    _extrinsic_domain,
    _extrinsic_distance,
    _extrinsic_divergence,
    _new_pure_function,
)

from .helpers import ids


def test_version():
    assert dp.__version__.startswith("0.")


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

    with pytest.raises(dp.OpenDPException) as error:
        binary_search(lambda x: x > 5.0, (0.0, 5.0))
    assert error.value.variant == "Search"
    assert binary_search(lambda x: x > 0, (0, 1)) == 1
    assert binary_search(lambda x: x < 1, (0, 1)) == 0
    with pytest.raises(dp.OpenDPException) as error:
        binary_search(lambda x: x < 1, (0, 0))
    assert error.value.variant == "Search"

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
    assert str(clamper.input_domain) == "VectorDomain(AtomDomain(T=i32))"
    assert str(clamper.input_domain.carrier_type) == "Vec<i32>"
    assert (
        str(clamper.output_domain) == "VectorDomain(AtomDomain(bounds=[0, 2], T=i32))"
    )
    assert str(clamper.output_domain.carrier_type) == "Vec<i32>"
    assert str(clamper.input_metric) == "SymmetricDistance()"
    assert str(clamper.input_metric.distance_type) == "u32"
    assert str(clamper.output_metric) == "SymmetricDistance()"
    assert str(clamper.output_metric.distance_type) == "u32"

    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance

    mechanism = make_laplace(
        atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0
    )
    assert str(mechanism.input_domain) == "AtomDomain(T=f64)"
    assert str(mechanism.input_domain.carrier_type) == "f64"
    assert str(mechanism.input_metric) == "AbsoluteDistance(f64)"
    assert str(mechanism.input_metric.distance_type) == "f64"
    assert str(mechanism.output_measure) == "MaxDivergence"
    assert str(mechanism.output_measure.distance_type) == "f64"


def test_function():
    from opendp.measurements import make_laplace
    from opendp.domains import atom_domain
    from opendp.metrics import absolute_distance
    from opendp.transformations import make_identity

    mechanism = make_laplace(
        atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0
    )
    pow = 4  # add noise 2^pow times
    for _ in range(pow):
        mechanism = mechanism >> mechanism.function

    # Exercise postprocessing transformation
    transformation = make_identity(
        atom_domain(T=float, nan=False), absolute_distance(T=float)
    )
    mechanism = mechanism >> transformation
    print("mechanism(0.0)", mechanism(0.0))


def test_privacy_profile():
    from opendp.measures import new_privacy_profile
    from opendp.mod import PrivacyProfile
    from opendp._convert import py_to_c
    from opendp._lib import AnyObjectPtr, CallbackFnPtr
    import math

    with pytest.warns(DeprecationWarning, match="vector-valued"):
        profile = new_privacy_profile(lambda eps: math.exp(-eps))
    # formula is -ln(1e-7)
    assert profile.epsilon(delta=1e-7) == pytest.approx(16.11809565095832)

    direct = PrivacyProfile(delta=lambda eps: [math.exp(-value) for value in eps])
    assert direct.epsilon(delta=1e-7) == profile.epsilon(delta=1e-7)

    log_profile = PrivacyProfile(log_delta=lambda eps: [-value for value in eps])
    assert log_profile.delta(epsilon=1.0) == pytest.approx(math.exp(-1.0))

    assert log_profile.log_delta(10000.0) == -10000.0
    assert log_profile.epsilon_from_log_delta(-10000.0) == 10000.0
    assert log_profile.delta(10000.0) == math.ulp(0.0)
    assert py_to_c(log_profile, AnyObjectPtr, "PrivacyProfile") is not None

    slack_profile = PrivacyProfile(
        log_delta=lambda eps: [0.0 if value < 2.0 else -math.inf for value in eps],
        delta_slack=0.1,
    )
    assert slack_profile.delta(2.0) == 0.1
    assert slack_profile.epsilon(0.1) == 2.0
    assert slack_profile.epsilon(math.nextafter(0.1, 0.0)) == math.inf
    assert slack_profile.log_delta(0.0) == 0.0
    assert slack_profile.epsilon_from_log_delta(-10000.0) == math.inf

    with pytest.raises(TypeError, match="expected a callable callback"):
        py_to_c(1, CallbackFnPtr, "f64")
    with pytest.raises(TypeError, match="_ptr cannot be combined"):
        PrivacyProfile(delta=lambda eps: [0.0] * len(eps), _ptr=log_profile.curve)
    with pytest.raises(TypeError, match="expected exactly one"):
        PrivacyProfile()
    with pytest.raises(TypeError, match="expected exactly one"):
        PrivacyProfile(delta=lambda eps: eps, log_delta=lambda eps: eps)


@pytest.mark.parametrize("is_log", [False, True])
def test_privacy_profile_vector_forward_callbacks(is_log):
    import math
    from opendp.mod import PrivacyProfile

    calls = []

    def callback(values):
        assert isinstance(values, list)
        calls.append(values.copy())
        return [-value if is_log else math.exp(-value) for value in values]

    profile = (
        PrivacyProfile(log_delta=callback) if is_log else PrivacyProfile(delta=callback)
    )
    assert calls == []
    assert profile.delta(1.0) == pytest.approx(math.exp(-1.0))
    assert calls == [[1.0]]
    assert profile.delta([0.0, 1.0, 1.0]) == pytest.approx(
        [1.0, math.exp(-1.0), math.exp(-1.0)]
    )
    assert calls[-1] == [0.0, 1.0, 1.0]
    assert len(calls) == 2
    assert profile.log_delta([0.0, 1.0, 1.0]) == pytest.approx([0.0, -1.0, -1.0])
    assert len(calls) == 3
    assert profile.log_delta([1.0]) == pytest.approx([-1.0])
    assert profile.delta([]) == []
    with pytest.raises(TypeError):
        PrivacyProfile(callback)  # type: ignore[misc]


def test_privacy_profile_generic_inversion():
    import math
    from opendp.mod import PrivacyProfile

    calls = []

    def forward(values):
        assert isinstance(values, list) and len(values) == 1
        calls.append(values.copy())
        return [-value for value in values]

    profile = PrivacyProfile(log_delta=forward)
    assert profile.epsilon_from_log_delta(-10000.0) == 10000.0
    assert profile.epsilon_from_log_delta([-math.inf, -2.0, -2.0, -1.0, 0.0]) == [
        math.inf,
        2.0,
        2.0,
        1.0,
        0.0,
    ]
    assert profile.epsilon([0.0, 0.25, 0.25, 1.0]) == pytest.approx(
        [math.inf, -math.log(0.25), -math.log(0.25), 0.0]
    )
    for query, values in (
        (profile.epsilon, [0.0, 0.25, 0.25, 1.0]),
        (profile.epsilon_from_log_delta, [-math.inf, -10000.0, -2.0, -2.0, 0.0]),
    ):
        assert query(values) == [query(value) for value in values]
    slack = PrivacyProfile(log_delta=forward, delta_slack=0.25)
    assert slack.epsilon([0.1, 0.25, 0.25, 1.0]) == [math.inf, math.inf, math.inf, 0.0]
    before = len(calls)
    assert slack.epsilon([0.1, 1.0]) == [math.inf, 0.0]
    assert slack.epsilon([]) == []
    assert slack.epsilon_from_log_delta([]) == []
    assert len(calls) == before


def test_privacy_profile_numpy_scalar_queries():
    from opendp.mod import PrivacyProfile

    np = pytest.importorskip("numpy")

    def callback(values):
        assert isinstance(values, list) and len(values) == 1
        assert isinstance(values[0], float)
        return [-value for value in values]

    profile = PrivacyProfile(log_delta=callback)
    for scalar in (np.int64, np.float32, np.float64):
        for query, value in (
            (profile.delta, 1),
            (profile.log_delta, 1),
            (profile.epsilon, 0),
            (profile.epsilon_from_log_delta, -1),
        ):
            result = query(scalar(value))
            assert isinstance(result, float)
            assert result == query(float(value))


def test_privacy_profile_vector_validation_and_failures():
    from opendp.mod import PrivacyProfile, OpenDPException

    calls = []

    def callback(values):
        calls.append(values)
        return [0.0] * len(values)

    profile = PrivacyProfile(log_delta=callback)
    for method, values in [
        (profile.delta, [1.0, 0.0]),
        (profile.log_delta, [1.0, 0.0]),
        (profile.epsilon, [0.2, 0.1]),
        (profile.epsilon_from_log_delta, [-1.0, -2.0]),
    ]:
        with pytest.raises(OpenDPException, match="nondecreasing"):
            method(values)
    assert calls == []
    for length in [0, 1, 3]:
        profile = PrivacyProfile(log_delta=lambda _: [0.0] * length)
        for method, values in [
            (profile.delta, [0.0, 1.0]),
            (profile.log_delta, [0.0, 1.0]),
            (profile.epsilon, [0.1, 0.2]),
            (profile.epsilon_from_log_delta, [-2.0, -1.0]),
        ]:
            # Generic inversion checks singleton forward callback outputs.
            if length == 1 and method in (
                profile.epsilon,
                profile.epsilon_from_log_delta,
            ):
                continue
            with pytest.raises(OpenDPException, match="output length"):
                method(values)
    ordinary = PrivacyProfile(delta=lambda _: [])
    with pytest.raises(OpenDPException, match="output length"):
        ordinary.delta(0.0)
    original = ValueError("vector callback failed")

    def fail(_):
        raise original

    profile = PrivacyProfile(log_delta=fail)
    for method, values in [(profile.delta, [0.0, 1.0]), (profile.epsilon, [0.1, 0.2])]:
        with pytest.raises(ValueError) as error:
            method(values)
        assert error.value is original

    # The new constructor does not adapt scalar callbacks.
    def scalar_only(value: float) -> float:
        return float(value)

    scalar_callback = PrivacyProfile(delta=scalar_only)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        scalar_callback.delta(1.0)


def test_legacy_privacy_profile_scalar_callback_with_vector_queries():
    import math
    from opendp.measures import new_privacy_profile

    calls = []

    def scalar_callback(value):
        assert isinstance(value, float)
        calls.append(value)
        return math.exp(-value)

    with pytest.warns(DeprecationWarning, match="vector-valued"):
        profile = new_privacy_profile(scalar_callback)
    assert profile.delta([0.0, 1.0, 1.0]) == pytest.approx(
        [1.0, math.exp(-1.0), math.exp(-1.0)]
    )
    assert calls == [0.0, 1.0, 1.0]
    assert profile.epsilon(0.5) == pytest.approx(math.log(2.0))


def test_privacy_guarantee_profile_representation_is_distinct():
    from opendp.mod import PrivacyGuarantee, PrivacyProfile

    profile = PrivacyProfile(log_delta=lambda eps: [-value for value in eps])
    guarantee = PrivacyGuarantee(profile=profile)

    assert isinstance(profile, PrivacyProfile)
    assert isinstance(guarantee, PrivacyGuarantee)
    assert not isinstance(profile, PrivacyGuarantee)
    assert guarantee.epsilon(delta=1e-7) == profile.epsilon(delta=1e-7)

    with pytest.raises(TypeError, match="profile must be a PrivacyProfile"):
        PrivacyGuarantee(profile=lambda _eps: 0.0)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        PrivacyGuarantee(profile)  # type: ignore[arg-type, misc]
    with pytest.raises(TypeError, match="expected at least one"):
        PrivacyGuarantee()
    assert guarantee.delta(1.0) == pytest.approx(profile.delta(1.0))


@pytest.mark.parametrize("delta", [0.0, 5e-324, 0.1, 1.0])
def test_privacy_guarantee_approx_dp_certificate(delta):
    import math
    from opendp.mod import PrivacyGuarantee

    guarantee = PrivacyGuarantee(approx_dp=(2.0, delta))
    assert guarantee.delta(2.0) == delta
    assert guarantee.delta(math.inf) == delta
    assert guarantee.delta(math.nextafter(2.0, 0.0)) == 1.0
    assert guarantee.epsilon(delta) == (0.0 if delta == 1.0 else 2.0)
    assert guarantee.epsilon(1.0) == 0.0
    if delta > 0.0:
        assert guarantee.epsilon(math.nextafter(delta, 0.0)) == math.inf


def test_privacy_guarantee_conjunctive_representations():
    import math
    from opendp.mod import PrivacyGuarantee, PrivacyProfile
    from opendp.measures import _privacy_guarantee_with_approx_dp

    profile = PrivacyProfile(
        log_delta=lambda eps: [0.0 if e < 2.0 else -math.inf for e in eps]
    )
    combined = PrivacyGuarantee(approx_dp=(1.0, 0.1), profile=profile)
    reverse = _privacy_guarantee_with_approx_dp(
        PrivacyGuarantee(profile=profile), 1.0, 0.1
    )
    for guarantee in [combined, reverse]:
        assert guarantee.delta(1.0) == 0.1
        assert guarantee.delta(2.0) == 0.0
        assert guarantee.epsilon(0.1) == 1.0
        assert guarantee.epsilon(0.0) == 2.0
    assert profile.delta(1.0) == 1.0
    assert profile.epsilon(0.1) == 2.0


def test_privacy_guarantee_builders_do_not_require_hbc(monkeypatch):
    from opendp.mod import PrivacyGuarantee, PrivacyProfile

    profile = PrivacyProfile(log_delta=lambda eps: [-value for value in eps])
    monkeypatch.setattr("opendp.mod.GLOBAL_FEATURES", {"contrib"})
    for guarantee in [
        PrivacyGuarantee(approx_dp=(1.0, 0.1)),
        PrivacyGuarantee(profile=profile),
        PrivacyGuarantee(approx_dp=(1.0, 0.1), profile=profile),
    ]:
        assert guarantee.delta(1.0) >= 0.0
        assert guarantee.epsilon(0.1) >= 0.0


@pytest.mark.parametrize("approx_dp", [None, (1.0, 0.1)])
@pytest.mark.parametrize(
    "query, budget", [("delta", 1.0), ("epsilon", 0.1), ("beta", 0.5), ("alpha", 0.5)]
)
def test_privacy_guarantee_propagates_representation_errors(approx_dp, query, budget):
    from opendp.mod import PrivacyGuarantee, PrivacyProfile

    original = ValueError("profile failure")

    def fail(_):
        raise original

    guarantee = PrivacyGuarantee(
        approx_dp=approx_dp, profile=PrivacyProfile(log_delta=fail)
    )
    with pytest.raises(ValueError) as error:
        getattr(guarantee, query)(budget)
    assert error.value is original


def test_privacy_guarantee_construction_validation():
    import math
    from opendp.mod import PrivacyGuarantee, OpenDPException

    for invalid in [1.0, (1.0,), (1.0, 0.0, 0.0)]:
        with pytest.raises(TypeError, match="tuple"):
            PrivacyGuarantee(approx_dp=invalid)  # type: ignore[arg-type]
    for epsilon, delta in [
        (-1.0, 0.0),
        (math.inf, 0.0),
        (math.nan, 0.0),
        (1.0, -0.1),
        (1.0, 1.1),
        (1.0, math.nan),
    ]:
        with pytest.raises(OpenDPException):
            PrivacyGuarantee(approx_dp=(epsilon, delta))
    guarantee = PrivacyGuarantee(approx_dp=(1.0, 0.0))
    with pytest.raises(TypeError, match="_ptr cannot be combined"):
        PrivacyGuarantee(approx_dp=(1.0, 0.0), _ptr=guarantee.guarantee)
    with pytest.raises(TypeError):
        PrivacyGuarantee((1.0, 0.0))  # type: ignore[misc]
    with pytest.raises(OpenDPException):
        guarantee.delta(-1.0)
    with pytest.raises(OpenDPException):
        guarantee.epsilon(1.1)


def test_privacy_guarantee_and_profile_pointer_roundtrips():
    import gc
    import weakref
    from opendp._convert import py_to_c, c_to_py
    from opendp._lib import AnyObjectPtr
    from opendp.mod import PrivacyGuarantee, PrivacyProfile

    assert PrivacyGuarantee.__bases__ == (object,)
    assert PrivacyProfile.__bases__ == (object,)
    callback = lambda eps: [-value for value in eps]
    reference = weakref.ref(callback)
    profile = PrivacyProfile(log_delta=callback)
    roundtrip = c_to_py(py_to_c(profile, AnyObjectPtr, "PrivacyProfile"))
    assert roundtrip.curve is profile.curve
    assert roundtrip.log_delta(10000.0) == -10000.0
    guarantee = PrivacyGuarantee(approx_dp=(1.0, 0.1), profile=roundtrip)
    del callback, profile, roundtrip
    gc.collect()
    assert reference() is not None  # the native profile still owns the callback
    roundtrip = c_to_py(py_to_c(guarantee, AnyObjectPtr, "PrivacyGuarantee"))
    assert roundtrip.guarantee is guarantee.guarantee
    del guarantee
    gc.collect()
    assert roundtrip.delta(1.0) == 0.1
    assert roundtrip.epsilon(0.01) == pytest.approx(4.605170185988092)
    del roundtrip
    gc.collect()
    assert reference() is None


def test_privacy_guarantee_fdp_reporting():
    import math
    from opendp.mod import PrivacyGuarantee, PrivacyProfile

    profile = PrivacyProfile(log_delta=lambda eps: [-math.inf] * len(eps))
    for guarantee in [
        PrivacyGuarantee(approx_dp=(0.0, 0.0)),
        PrivacyGuarantee(profile=profile),
        PrivacyGuarantee(approx_dp=(0.0, 0.0), profile=profile),
    ]:
        assert guarantee.beta(alpha=0.3) == pytest.approx(0.7)
        assert guarantee.alpha(beta=0.7) == pytest.approx(0.3)
        assert guarantee.alpha(beta=0.0) < 1.0
        assert guarantee.alpha(beta=1.0) == 0.0
        assert guarantee.beta([0.0, 0.25, 0.25, 1.0]) == pytest.approx(
            [1.0, 0.75, 0.75, 0.0]
        )
        assert guarantee.alpha([0.25, 0.5, 0.5, 1.0]) == pytest.approx(
            [0.75, 0.5, 0.5, 0.0]
        )
        assert guarantee.beta([]) == []
        assert guarantee.alpha([]) == []


def test_fdp_reporting_validates_vectors_before_source_queries():
    from opendp.mod import PrivacyGuarantee, PrivacyProfile, OpenDPException

    def unexpected(_):
        raise AssertionError("invalid query must be rejected before source evaluation")

    guarantee = PrivacyGuarantee(profile=PrivacyProfile(log_delta=unexpected))
    for query in [guarantee.beta, guarantee.alpha]:
        for invalid in [[0.5, 0.1], [float("nan")], [-0.1], [1.1]]:
            with pytest.raises(OpenDPException):
                query(invalid)


@pytest.mark.parametrize("output", [[], [0.0, 0.0], [float("nan")], [0.1]])
def test_fdp_reporting_propagates_invalid_source_results(output):
    from opendp.mod import PrivacyGuarantee, PrivacyProfile, OpenDPException

    guarantee = PrivacyGuarantee(
        approx_dp=(0.0, 0.0),
        profile=PrivacyProfile(log_delta=lambda _: output),
    )
    for query in [guarantee.beta, guarantee.alpha]:
        with pytest.raises(OpenDPException):
            query(0.5)


def test_privacy_guarantee_renyi_representation():
    from opendp.mod import PrivacyGuarantee

    with pytest.raises(TypeError, match="renyiDP_delta requires renyiDP"):
        PrivacyGuarantee(approx_dp=(1.0, 0.1), renyiDP_delta=0.2)

    batches = []

    def curve(orders):
        batches.append(orders)
        assert isinstance(orders, list)
        assert all(order > 1 for order in orders)
        return [0.5 * order for order in orders]

    guarantee = PrivacyGuarantee(renyiDP=curve)
    assert guarantee.epsilon(delta=0.1) > 0.0
    assert guarantee.delta(epsilon=1.0) < 1.0

    approximate = PrivacyGuarantee(
        renyiDP=lambda orders: [0.0] * len(orders),
        renyiDP_delta=0.2,
    )
    assert approximate.delta(epsilon=float("inf")) == 0.2
    assert batches and all(len(batch) == 1 for batch in batches)


@pytest.mark.parametrize("output", [[], [0.0, 0.0], [float("nan")], [-1.0]])
def test_privacy_guarantee_invalid_renyi_output(output):
    import math

    guarantee = dp.PrivacyGuarantee(renyiDP=lambda _: output)
    with pytest.raises(dp.OpenDPException) as error:
        guarantee.delta(epsilon=1.0)
    assert error.value.variant == (
        "NumericError" if any(math.isnan(x) for x in output) else "FailedMap"
    )


def test_privacy_guarantee_renyi_callback_error():
    original = dp.OpenDPException("NumericError", "RDP failed")

    def curve(_):
        raise original

    guarantee = dp.PrivacyGuarantee(approx_dp=(1.0, 0.1), renyiDP=curve)
    with pytest.raises(dp.OpenDPException) as error:
        guarantee.delta(epsilon=1.0)
    assert error.value is original


def test_privacy_guarantee_zcdp_representation():
    from opendp.mod import PrivacyGuarantee

    with pytest.raises(TypeError, match="zCDP_delta requires zCDP"):
        PrivacyGuarantee(approx_dp=(1.0, 0.1), zCDP_delta=0.2)

    guarantee = PrivacyGuarantee(zCDP=0.5)
    assert guarantee.epsilon(delta=0.1) > 0.0
    assert guarantee.delta(epsilon=1.0) < 1.0

    approximate = PrivacyGuarantee(zCDP=0.0, zCDP_delta=0.2)
    assert approximate.delta(epsilon=float("inf")) == 0.2


def test_privacy_guarantee_gdp_representation():
    from opendp.mod import PrivacyGuarantee

    guarantee = PrivacyGuarantee(gaussianDP=1.0)
    assert guarantee.beta(alpha=0.0) == 1.0
    assert guarantee.beta(alpha=1.0) == 0.0
    assert 0.0 <= guarantee.beta(alpha=0.5) <= 1.0
    assert 0.0 <= guarantee.alpha(beta=0.5) <= 1.0
    assert 0.0 <= guarantee.delta(epsilon=0.5) <= 1.0


def test_member():
    from opendp.domains import atom_domain, vector_domain
    from opendp.metrics import symmetric_distance

    int_10_domain = atom_domain(T=int, bounds=(0, 10))
    assert int_10_domain.member(0)
    assert not int_10_domain.member(100)
    with pytest.warns(UserWarning, match=r"inferred type is f64, expected i32"):
        assert not int_10_domain.member(0.0)
    with pytest.warns(UserWarning, match=r"inferred type is f64, expected i32"):
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

    mechanism = make_laplace(
        atom_domain(T=float, nan=False), absolute_distance(T=float), 1.0
    )
    assert not mechanism.input_domain.member(float("NaN"))


def test_new_domain():
    from opendp.domains import atom_domain, vector_domain

    domain = atom_domain(T=dp.i32)
    assert domain.member(3)
    assert str(domain) == "AtomDomain(T=i32)"

    domain = atom_domain(T=dp.f64, nan=False)
    assert not domain.member(float("nan"))
    assert str(domain) == "AtomDomain(T=f64)"

    domain = atom_domain((1, 2))
    assert domain.member(2)
    assert not domain.member(3)
    assert str(domain) == "AtomDomain(bounds=[1, 2], T=i32)"

    domain = vector_domain(atom_domain(T=dp.i32))
    assert domain.member([2])
    assert str(domain) == "VectorDomain(AtomDomain(T=i32))"

    domain = vector_domain(atom_domain((2, 3)))
    assert domain.member([2])
    assert not domain.member([2, 4])
    assert str(domain) == "VectorDomain(AtomDomain(bounds=[2, 3], T=i32))"

    domain = vector_domain(atom_domain(T=dp.i32), 10)
    assert domain.member([1] * 10)
    assert str(domain) == "VectorDomain(AtomDomain(T=i32), size=10)"

    domain = vector_domain(atom_domain((2.0, 7.0)), 10)
    assert domain.member([3.0] * 10)
    assert not domain.member([1.0] * 10)
    assert (
        str(domain)
        == "VectorDomain(AtomDomain(bounds=[2.0, 7.0], nan=true, T=f64), size=10)"
    )

    null_domain = atom_domain(nan=True, T=float)
    assert str(null_domain) == "AtomDomain(nan=true, T=f64)"
    assert null_domain.member(float("nan"))

    not_null_domain = atom_domain(nan=False, T=float)
    assert str(not_null_domain) == "AtomDomain(T=f64)"
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

    gc.collect()  # if refcount is incorrect, then accessing .descriptor will trigger a use-after-free

    # can retrieve the descriptor for use in further analysis
    assert domain.descriptor == {1, 2, 3, 4}
    # or retrieve the descriptor with type-checking
    assert domain.cast(set) == {1, 2, 3, 4}

    with pytest.raises(ValueError, match="domain descriptor must be a int"):
        assert domain.cast(int)

    # nest inside a vector domain
    vec_domain = dp.vector_domain(domain)
    january_1 = datetime.fromisoformat("2024-01-01")
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


def test_extrinsic_callback_result_roundtrip():
    qbl = dp.new_queryable(lambda query: query + 1, int, int)
    assert qbl(2) == 3


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


@pytest.mark.parametrize(
    "new_distance, new_divergence",
    [
        (dp.user_distance, dp.user_divergence),
        (_extrinsic_distance, _extrinsic_divergence),
    ],
)
def test_custom_distance(new_distance, new_divergence):
    from datetime import datetime, timedelta

    # create custom transformation
    trans = dp.t.make_user_transformation(
        dp.vector_domain(
            dp.user_domain("DatetimeDomain()", lambda x: isinstance(x, datetime))
        ),
        new_distance("sum of millisecond distances"),
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        lambda arg: sum(datetime.timestamp(x) for x in arg),
        lambda d_in: d_in.total_seconds() * 1000,
    )

    january_1 = datetime.fromisoformat("2024-01-01")
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
