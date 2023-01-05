from opendp.partial import enable_partial

enable_partial()

from opendp.mod import Measurement, enable_features
from opendp.transformations import *
from opendp.measurements import *

enable_features("floating-point", "contrib")


def test_unit():
    partial_base_laplace = make_base_laplace()
    meas = partial_base_laplace(1.0)  # gives a base_laplace Measurement with scale 1.
    assert isinstance(meas, Measurement)


def test_chain():
    size = 10
    bounds = (0.0, 10.0)
    partial_meas = make_sized_bounded_mean(size, bounds) >> make_base_laplace()

    dp_mean_meas = partial_meas.fix(d_in=2, d_out=1.0)
    assert dp_mean_meas.param > 1.0


from integration.test_mean import *


def test_getattr():
    meas = (
        make_split_dataframe(separator=",", col_names=col_names)
        .select_column(key=index, TOA=str)
        .cast(TIA=str, TOA=float)
        .impute_constant(impute_constant)
        .clamp(bounds)
        .bounded_resize(n, bounds, impute_constant)
        .sized_bounded_mean(n, bounds)
        .base_laplace()
        .fix(1, 1.)
    )

    res = meas(data)
    assert type(res) == float
