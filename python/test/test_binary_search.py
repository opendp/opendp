import math
import sys

import pytest
import opendp.prelude as dp
from opendp.mod import OpenDPException, _call_rust_search


def test_binary_search_fail():
    with pytest.raises(ZeroDivisionError):
        dp.binary_search(lambda _: bool(1 / 0), T=float)


@pytest.mark.parametrize("epsilon", [1.01, 1e-300])
def test_binary_search_overflow(epsilon):
    # Legacy float sum arithmetic, including infinite privacy loss at scale zero.
    space = (
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
    )
    bounded_sum = space >> dp.t.then_clamp((0.0, 1.0)) >> dp.t.then_sum()
    calls = []

    def make_chain(scale):
        calls.append(scale)
        return bounded_sum >> dp.m.then_laplace(scale)

    scale = dp.binary_search_param(make_chain, 1, epsilon, T=float)
    trace = calls[:]
    assert len(trace) < 300
    calls.clear()
    measurement = dp.binary_search_chain(make_chain, 1, epsilon, T=float)
    assert calls == trace + [scale]
    assert measurement.check(1, epsilon) and math.isfinite(measurement.map(1))
    assert not make_chain(math.nextafter(scale, 0.0)).check(1, epsilon)
    assert math.isinf(make_chain(0.0).map(1))
    assert not make_chain(0.0).check(1, epsilon)


def test_legacy_overflow_is_not_recovered():
    calls = []

    def make_chain(bound):
        calls.append(bound)
        return dp.t.make_sized_bounded_float_ordered_sum(1000, (-bound, bound))

    with pytest.raises(OpenDPException) as exc_info:
        dp.binary_search_param(
            make_chain, 2, sys.float_info.max, bounds=(0.0, None), T=float
        )
    assert exc_info.value.variant == "Overflow"
    assert max(calls) > 2.0**49
    assert len(calls) < 2400


@pytest.mark.parametrize("nan", [False, True])
def test_legacy_nonfinite_scale_is_not_a_passing_parameter(nan):
    space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)

    def make_chain(parameter):
        scale = parameter * 1e290  # silently overflows only in the new bands
        return dp.m.make_laplace(*space, scale=scale - scale if nan else scale)

    with pytest.raises(OpenDPException) as exc_info:
        dp.binary_search_param(make_chain, 1.0, 1e-310, bounds=(0.0, None), T=float)
    assert exc_info.value.variant == "MakeTransformation"


def test_stuck():
    input_space = (
        dp.vector_domain(dp.atom_domain(T=float, nan=False)),
        dp.symmetric_distance(),
    )

    epsilon = 1.3
    sens = 500_000.0 * 500_000.0
    bounded_sum = input_space >> dp.t.then_clamp(bounds=(0.0, sens)) >> dp.t.then_sum()
    real_v = sens / epsilon
    discovered_scale = dp.binary_search_param(
        lambda s: bounded_sum >> dp.m.then_laplace(scale=s),
        d_in=1,
        bounds=(0.0, real_v * 2.0),
        d_out=epsilon,
    )
    assert discovered_scale == 192307694098.6973


def test_binary_search():
    assert dp.binary_search(lambda x: x <= -5, T=int) == -5
    assert dp.binary_search(lambda x: x <= 5, T=int) == 5
    assert dp.binary_search(lambda x: x >= -5, T=int) == -5
    assert dp.binary_search(lambda x: x >= 5, T=int) == 5


@pytest.mark.parametrize("bounds", [(0, None), (None, 10)])
def test_binary_search_one_sided(bounds):
    assert dp.binary_search(lambda x: x <= 5, bounds=bounds) == 5


@pytest.mark.parametrize("bounds", [(0,), (0, 10, 20)])
def test_binary_search_rejects_malformed_bounds(bounds):
    with pytest.raises(ValueError, match="bounds must contain exactly two elements"):
        dp.binary_search(lambda x: x <= 5, bounds=bounds)


@pytest.mark.parametrize("value", [float("nan"), float("inf"), -float("inf")])
@pytest.mark.parametrize("side", [0, 1])
def test_nonfinite_bounds(value, side):
    bounds = (value, 1.0) if side == 0 else (0.0, value)
    calls = []

    def predicate(x):
        calls.append(x)
        return True

    with pytest.raises(ValueError, match="bounds must be finite"):
        dp.binary_search(predicate, bounds=bounds, T=float)
    assert not calls


@pytest.mark.parametrize(
    "bounds, message",
    [
        ((10, None), "decision boundary is below the lower bound"),
        ((None, 0), "decision boundary is above the upper bound"),
    ],
)
def test_binary_search_one_sided_errors(bounds, message):
    with pytest.raises(ValueError, match=message):
        dp.binary_search(lambda x: x <= 5, bounds=bounds)


def test_mixed_type_bounds():
    with pytest.raises(TypeError, match="bounds must share the same type"):
        dp.binary_search(lambda x: x <= -5, bounds=(-10, 20.0))


def test_binary_search_inferred_int():
    def predicate(v):
        if not isinstance(v, int):
            raise TypeError("Expected int")
        return v <= 5

    assert dp.binary_search(predicate) == 5


def test_binary_search_inferred_int_fail():
    def predicate(v):
        if not isinstance(v, int):
            raise TypeError("Expected int")
        raise ValueError("I always fail for some other reason")

    # int is inferred, but the ValueError should pop up
    with pytest.raises(ValueError):
        dp.binary_search(predicate)


def test_binary_search_failed_infer():
    def predicate(_):
        raise TypeError("always raised for some other reason")

    # type inference should fail, and we get the error from type inference
    with pytest.raises(TypeError):
        dp.binary_search(predicate)


def test_binary_search_forwards_rust_type_errors():
    def search(_):
        raise OpenDPException("Type", "expected int")

    with pytest.raises(TypeError, match="expected int"):
        _call_rust_search(lambda _: True, search)


def test_binary_search_forwards_rust_search_errors():
    def search(_):
        raise OpenDPException("Search", "predicate is constant")

    with pytest.raises(ValueError, match="predicate is constant"):
        _call_rust_search(lambda _: True, search)


def test_binary_search_forwards_untranslated_rust_errors():
    terminal = OpenDPException("FFI", "unrelated Rust failure")

    def predicate(value):
        if value == 0:
            raise ValueError("recovered callback failure")
        return True

    def search(wrapped):
        with pytest.raises(ValueError):
            wrapped(0)
        wrapped(1)
        raise terminal

    with pytest.raises(OpenDPException) as exc_info:
        _call_rust_search(predicate, search)
    assert exc_info.value is terminal


@pytest.mark.parametrize(
    "variant",
    ["NumericError", "Search", None],
)
@pytest.mark.parametrize("bounds", [None, (0, 10)])
def test_binary_search_propagates_non_range_errors(variant, bounds):
    original = (
        OpenDPException(variant, "callback failure")
        if variant
        else ValueError("callback failure")
    )
    calls = []

    def predicate(value):
        calls.append(value)
        if value == 0:
            raise original
        raise AssertionError("evaluated after a terminal callback failure")

    with pytest.raises(type(original)) as exc_info:
        dp.binary_search(predicate, bounds=bounds, T=int)
    assert exc_info.value is original
    if bounds is not None:
        assert calls == [0]
    else:
        assert 1 < len(calls) < 40


def test_exponential_bounds_search():
    assert dp.exponential_bounds_search(lambda x: x > 0, int) == (0, 1)
    assert dp.exponential_bounds_search(lambda x: x > 0, float) == (0.0, 0.5)


@pytest.mark.parametrize("numpy_bool", [False, True])
def test_exponential_bounds_search_propagates_late_callback_error(numpy_bool):
    boolean = pytest.importorskip("numpy").bool_ if numpy_bool else bool
    original = ValueError("boom happened")

    def predicate(value):
        if value == 16:
            raise original
        return boolean(False)

    with pytest.raises(ValueError, match="boom happened") as exc_info:
        dp.exponential_bounds_search(predicate, int)
    assert exc_info.value is original


def test_overflow_recovery():
    def predicate(value):
        if value > 10:
            raise OpenDPException("Overflow", "intermediate overflow")
        return value >= 5

    result = dp.binary_search(predicate, bounds=(0, 20))
    assert result == 5 and predicate(result)


def test_initial_overflow_recovery_and_later_errors():
    def predicate(value):
        if value <= 0:
            raise OpenDPException("Overflow", "initial point unavailable")
        return value >= 5

    assert dp.binary_search(predicate, T=int) == 5

    def constant(value):
        predicate(value)
        return True

    with pytest.raises(ValueError, match="unable to infer bounds"):
        dp.binary_search(constant, T=int)

    terminal = RuntimeError("later terminal callback failure")

    def failing(value):
        predicate(value)
        if value >= 16:
            raise terminal
        return False

    with pytest.raises(RuntimeError) as exc_info:
        dp.binary_search(failing, T=int)
    assert exc_info.value is terminal


def test_recovered_range_error_does_not_mask_terminal_callback_error():
    terminal = OpenDPException("NumericError", "terminal backend failure")

    def predicate(value):
        if value > 10:
            raise OpenDPException("Overflow", "recoverable probe overflow")
        if value == 8:
            raise terminal
        return value >= 5

    with pytest.raises(OpenDPException) as exc_info:
        dp.binary_search(predicate, T=int)
    assert exc_info.value is terminal


def test_callback_exception_identity_with_duplicate_messages():
    original, later = ValueError("same"), ValueError("same")

    def predicate(value):
        raise (original if value == 0 else later)

    with pytest.raises(ValueError) as exc_info:
        dp.binary_search(predicate, T=int)
    assert exc_info.value is original

    assert original is not later


def test_type_inference():
    def chainer(b):
        return dp.t.make_sum(
            dp.vector_domain(dp.atom_domain(bounds=(-b, b)), size=1000),
            dp.symmetric_distance(),
        )

    assert dp.binary_search_param(chainer, 2, 100) == pytest.approx(50)

    def mean_chainer_n(n):
        return dp.t.make_mean(
            dp.vector_domain(dp.atom_domain(bounds=(-20.0, 20.0)), size=n),
            dp.symmetric_distance(),
        )

    assert dp.binary_search_param(mean_chainer_n, 2, 1.0, T=int) == 41

    def mean_chainer_b(b):
        return dp.t.make_mean(
            dp.vector_domain(dp.atom_domain(bounds=(-b, b)), size=1000),
            dp.symmetric_distance(),
        )

    assert 499.999 < dp.binary_search_param(mean_chainer_b, 2, 1.0) < 500.0


@pytest.mark.parametrize("sign", [-1, 1])
@pytest.mark.parametrize("polarity", [False, True])
@pytest.mark.parametrize("one_sided", [False, True])
def test_initial_invalid_segment(sign, polarity, one_sided):
    def predicate(value):
        value *= sign
        if not 3 <= value <= 16:
            raise ValueError("outside the evaluable interval")
        return (value >= 5) == polarity

    bounds = ((0, None) if sign > 0 else (None, 0)) if one_sided else None
    assert dp.binary_search(predicate, bounds=bounds, T=int) == sign * (
        5 if polarity else 4
    )


@pytest.mark.parametrize("sign", [-1, 1])
def test_invalid_one_sided_endpoint_respects_bound(sign):
    def predicate(value):
        value *= sign
        assert value >= 10
        if value == 10:
            raise ValueError("unavailable initial endpoint")
        return value >= 15

    bounds = (10, None) if sign > 0 else (None, -10)
    assert dp.binary_search(predicate, bounds=bounds, T=int) == sign * 15


def test_narrow_evaluable_interval_can_be_missed():
    original = ValueError("original")

    def predicate(value):
        if value == 7:
            return True
        if value == 0:
            raise original
        raise RuntimeError("other discovery error")

    with pytest.raises(ValueError) as error:
        dp.binary_search(predicate, T=int)
    assert error.value is original


def test_initial_recovery_does_not_mask_later_same_message():
    original, terminal = ValueError("same"), ValueError("same")

    def predicate(value):
        if value <= 0:
            raise original
        if value >= 16:
            raise terminal
        return False

    with pytest.raises(ValueError) as error:
        dp.binary_search(predicate, T=int)
    assert error.value is terminal


def test_tolerated_search_error_does_not_mask_absent_boundary():
    swallowed = OpenDPException("Search", "initial region unavailable")
    maximum = 2**31 - 1

    def predicate(value):
        if value == maximum:
            return False
        raise swallowed

    with pytest.raises(ValueError, match="unable to infer bounds"):
        dp.binary_search(predicate, T=int)
