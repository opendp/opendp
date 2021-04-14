import opendp
import pytest
from opendp import OdpException

odp = opendp.OpenDP()


def test_unknown_dispatch():
    with pytest.raises(OdpException):
        odp.meas.make_base_laplace(b"<u32>", opendp.f64_p(1.0))


def test_invalid_arg_length():
    with pytest.raises(OdpException):
        odp.meas.make_base_laplace(b"<u32, f64>", opendp.f64_p(1.0))


def test_invalid_data():
    base_laplace = odp.meas.make_base_laplace(b"<f64>", opendp.f64_p(1.0))
    with pytest.raises(OdpException):
        odp.measurement_invoke(base_laplace, 2)


def test_invalid_constructor():
    with pytest.raises(OdpException):
        odp.meas.make_base_laplace(b"<f64>", opendp.f64_p(-1.0))


def test_relation():
    base_laplace = odp.meas.make_base_gaussian(b"<f64>", opendp.f64_p(1.0))
    with pytest.raises(OdpException):
        odp.measurement_check(base_laplace, 2, (-.01, 1e-7))
