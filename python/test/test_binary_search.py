from opendp.mod import binary_search_param, enable_features, binary_search
from opendp.trans import make_bounded_sum, make_clamp
from opendp.meas import make_base_laplace

enable_features('floating-point', 'contrib')


def test_binary_search_overflow():
    d_in = 1
    d_out = 1.01
    bounded_sum = (
        make_clamp(bounds=(0.0, 1.0)) >>
        make_bounded_sum(bounds=(0.0, 1.0))
    )
    binary_search_param(
        lambda s: bounded_sum >> make_base_laplace(scale=s),
        d_in=d_in,
        d_out=d_out
    )

def test_stuck():
    epsilon = 1.3
    sens = 500_000.0 * 500_000.0
    bounded_sum = (
        make_clamp(bounds=(0.0, sens)) >>
        make_bounded_sum(bounds=(0.0, sens))
    )
    real_v = sens / epsilon
    discovered_scale = binary_search_param(
        lambda s: bounded_sum >> make_base_laplace(scale=s),
        d_in=1,
        bounds=(0.0, real_v * 2.0),
        d_out=(epsilon))
    print(discovered_scale)

def test_binary_search():
    assert binary_search(lambda x: x <= -5) == -5
    assert binary_search(lambda x: x <= 5) == 5
    assert binary_search(lambda x: x >= -5) == -5
    assert binary_search(lambda x: x >= 5) == 5
