import pytest
import opendp.prelude as dp


def test_binary_search_fail():
    with pytest.raises(ZeroDivisionError) as e:
        dp.binary_search(lambda _: bool(1 / 0), T=float)
    if hasattr(e.value, "add_note"):
        assert e.value.__notes__[0] == "Predicate in binary search always raises an exception. This exception is raised when the predicate is evaluated at 0.0." # type: ignore[attr-defined]

def test_binary_search_overflow():

    input_domain = dp.vector_domain(dp.atom_domain(T=float))
    input_metric = dp.symmetric_distance()

    d_in = 1
    d_out = 1.01
    bounded_sum = (
        dp.t.make_clamp(input_domain, input_metric, bounds=(0.0, 1.0)) >>
        dp.t.then_sum()
    )
    dp.binary_search_param(
        lambda s: bounded_sum >> dp.m.then_laplace(scale=s),
        d_in=d_in,
        d_out=d_out
    )

def test_stuck():
    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()

    epsilon = 1.3
    sens = 500_000.0 * 500_000.0
    bounded_sum = (
        input_space >>
        dp.t.then_clamp(bounds=(0.0, sens)) >>
        dp.t.then_sum()
    )
    real_v = sens / epsilon
    discovered_scale = dp.binary_search_param(
        lambda s: bounded_sum >> dp.m.then_laplace(scale=s),
        d_in=1,
        bounds=(0.0, real_v * 2.0),
        d_out=epsilon)
    print(discovered_scale)
    
def test_binary_search():
    assert dp.binary_search(lambda x: x <= -5, T=int) == -5
    assert dp.binary_search(lambda x: x <= 5, T=int) == 5
    assert dp.binary_search(lambda x: x >= -5, T=int) == -5
    assert dp.binary_search(lambda x: x >= 5, T=int) == 5


def test_type_inference():
    def chainer(b):
        return dp.t.make_sum(
            dp.vector_domain(dp.atom_domain(bounds=(-b, b)), size=1000), 
            dp.symmetric_distance())
    assert dp.binary_search_param(chainer, 2, 100) == pytest.approx(50)

    def mean_chainer_n(n):
        return dp.t.make_mean(
            dp.vector_domain(dp.atom_domain(bounds=(-20., 20.)), size=n), 
            dp.symmetric_distance())
    assert dp.binary_search_param(mean_chainer_n, 2, 1.) == 41

    def mean_chainer_b(b):
        return dp.t.make_mean(
            dp.vector_domain(dp.atom_domain(bounds=(-b, b)), size=1000), 
            dp.symmetric_distance())
    assert 499.999 < dp.binary_search_param(mean_chainer_b, 2, 1.) < 500.

