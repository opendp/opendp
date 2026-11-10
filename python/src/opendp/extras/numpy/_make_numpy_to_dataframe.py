"""Stable conversion of a homogeneous NumPy matrix to an eager Polars DataFrame."""

from opendp._internal import _make_transformation
from opendp.context import register
from opendp.domains import atom_domain, dataframe_domain, series_domain
from opendp.extras.numpy._make_dataframe_to_numpy import _NUMPY_DTYPES, _row_distance


def make_numpy_to_dataframe(input_domain, input_metric, *, names=None):
    """Convert a numeric NumPy matrix using public feature names and dtypes.

    The feature count must be public. Numerical bounds are dropped, but the
    element dtype and non-NaN guarantee are preserved.
    """
    import numpy as np
    import polars as pl
    from opendp.extras.numpy import NPArray2Domain

    descriptor = input_domain.cast(NPArray2Domain)
    width = descriptor.num_columns
    if width is None or width == 0:
        raise ValueError("NumPy to DataFrame conversion requires a positive public width")

    if names is None:
        names = tuple(f"x{i}" for i in range(width))
    elif isinstance(names, str):
        raise TypeError("names must be a sequence of public column names")
    else:
        names = tuple(names)

    if (
        len(names) != width
        or any(not isinstance(name, str) or not name for name in names)
        or len(set(names)) != len(names)
    ):
        raise ValueError("names must be unique, nonempty strings matching the width")

    T = str(descriptor.T)
    if T not in _NUMPY_DTYPES:
        raise TypeError(f"Unsupported NumPy dtype: {T}")
    dtype = np.dtype(_NUMPY_DTYPES[T])
    element = atom_domain(
        T=descriptor.T,
        **({"nan": descriptor.nan} if T in {"f32", "f64"} else {}),
    )
    output_domain = dataframe_domain([
        series_domain(name, element) for name in names
    ])

    def convert(array):
        if array.dtype != dtype or array.ndim != 2 or array.shape[1] != width:
            raise TypeError("NumPy array does not match the public domain")
        return pl.from_numpy(array, schema=names, orient="row")

    output_metric, stability = _row_distance(input_metric)
    return _make_transformation(
        input_domain, input_metric, output_domain, output_metric,
        convert, stability,
    )


register(make_numpy_to_dataframe, name="to_dataframe")
