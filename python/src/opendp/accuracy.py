# Auto-generated. Do not edit!
'''
The ``accuracy`` module provides functions for converting between accuracy and scale parameters.
For more context, see :ref:`accuracy in the User Guide <accuracy-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401

__all__ = [
    "accuracy_to_discrete_gaussian_scale",
    "accuracy_to_discrete_laplacian_scale",
    "accuracy_to_gaussian_scale",
    "accuracy_to_laplacian_scale",
    "discrete_gaussian_scale_to_accuracy",
    "discrete_laplacian_scale_to_accuracy",
    "gaussian_scale_to_accuracy",
    "laplacian_scale_to_accuracy",
    "summarize_polars_measurement"
]


def accuracy_to_discrete_gaussian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a desired `accuracy` (tolerance) into a discrete gaussian noise scale at a statistical significance level `alpha`.

    [accuracy_to_discrete_gaussian_scale in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.accuracy_to_discrete_gaussian_scale.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/accuracy_to_discrete_gaussian_scale.pdf)

    .. end-markdown

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``accuracy`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_discrete_gaussian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'accuracy_to_discrete_gaussian_scale',
            '__module__': 'accuracy',
            '__kwargs__': {
                'accuracy': accuracy, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def accuracy_to_discrete_laplacian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a desired `accuracy` (tolerance) into a discrete Laplacian noise scale at a statistical significance level `alpha`.

    [accuracy_to_discrete_laplacian_scale in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.accuracy_to_discrete_laplacian_scale.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/accuracy_to_discrete_laplacian_scale.pdf)

    .. end-markdown

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``accuracy`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :return: Discrete laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_discrete_laplacian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'accuracy_to_discrete_laplacian_scale',
            '__module__': 'accuracy',
            '__kwargs__': {
                'accuracy': accuracy, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def accuracy_to_gaussian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a desired `accuracy` (tolerance) into a gaussian noise scale at a statistical significance level `alpha`.

    [accuracy_to_gaussian_scale in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.accuracy_to_gaussian_scale.html)

    .. end-markdown

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``accuracy`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_gaussian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'accuracy_to_gaussian_scale',
            '__module__': 'accuracy',
            '__kwargs__': {
                'accuracy': accuracy, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def accuracy_to_laplacian_scale(
    accuracy,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a desired `accuracy` (tolerance) into a Laplacian noise scale at a statistical significance level `alpha`.

    [accuracy_to_laplacian_scale in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.accuracy_to_laplacian_scale.html)

    .. end-markdown

    :param accuracy: Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``accuracy`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :return: Laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=accuracy)

    # Convert arguments to c types.
    c_accuracy = py_to_c(accuracy, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__accuracy_to_laplacian_scale
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_accuracy, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'accuracy_to_laplacian_scale',
            '__module__': 'accuracy',
            '__kwargs__': {
                'accuracy': accuracy, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def discrete_gaussian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a discrete gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [discrete_gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.discrete_gaussian_scale_to_accuracy.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/discrete_gaussian_scale_to_accuracy.pdf)

    .. end-markdown

    :param scale: Gaussian noise scale.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``scale`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__discrete_gaussian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'discrete_gaussian_scale_to_accuracy',
            '__module__': 'accuracy',
            '__kwargs__': {
                'scale': scale, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def discrete_laplacian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    $\alpha = P[Y \ge accuracy]$, where $Y = | X - z |$, and $X \sim \mathcal{L}_{Z}(0, scale)$.
    That is, $X$ is a discrete Laplace random variable and $Y$ is the distribution of the errors.

    This function returns a float accuracy.
    You can take the floor without affecting the coverage probability.

    [discrete_laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.discrete_laplacian_scale_to_accuracy.html)

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/accuracy/discrete_laplacian_scale_to_accuracy.pdf)

    .. end-markdown

    :param scale: Discrete Laplacian noise scale.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``scale`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__discrete_laplacian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'discrete_laplacian_scale_to_accuracy',
            '__module__': 'accuracy',
            '__kwargs__': {
                'scale': scale, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def gaussian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.gaussian_scale_to_accuracy.html)

    .. end-markdown

    :param scale: Gaussian noise scale.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``scale`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__gaussian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'gaussian_scale_to_accuracy',
            '__module__': 'accuracy',
            '__kwargs__': {
                'scale': scale, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def laplacian_scale_to_accuracy(
    scale,
    alpha,
    T: Optional[RuntimeTypeDescriptor] = None
):
    r"""Convert a Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

    [laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/accuracy/fn.laplacian_scale_to_accuracy.html)

    .. end-markdown

    :param scale: Laplacian noise scale.
    :param alpha: Statistical significance, level-``alpha``, or (1. - ``alpha``)100% confidence. Must be within (0, 1].
    :param T: Data type of ``scale`` and ``alpha``
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=scale)

    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_accuracy__laplacian_scale_to_accuracy
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_scale, c_alpha, c_T), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'laplacian_scale_to_accuracy',
            '__module__': 'accuracy',
            '__kwargs__': {
                'scale': scale, 'alpha': alpha, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def summarize_polars_measurement(
    measurement: Measurement,
    alpha = None
):
    r"""Summarize the statistics to be released from a measurement that returns a OnceFrame.

    If a threshold is configured for censoring small/sensitive groups,
    a threshold column will be included,
    containing the cutoff for the respective count query being thresholded.


    Required features: `contrib`

    .. end-markdown

    :param measurement: computation from which you want to read noise scale parameters from
    :type measurement: Measurement
    :param alpha: optional statistical significance to use to compute accuracy estimates
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

        First, create a measurement with the Polars API:

        >>> import opendp.prelude as dp
        >>> import polars as pl
        >>> dp.enable_features("contrib")
        ... 
        >>> lf = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
        >>> lf_domain = dp.lazyframe_domain([
        ...     dp.series_domain("A", dp.atom_domain(T="i32")), 
        ...     dp.series_domain("B", dp.atom_domain(T=str))
        ... ])
        >>> lf_domain = dp.with_margin(lf_domain, dp.polars.Margin(by=[], max_length=1000))
        >>> meas = dp.m.make_private_lazyframe(
        ...     lf_domain,
        ...     dp.symmetric_distance(),
        ...     dp.max_divergence(),
        ...     lf.select([dp.len(), pl.col("A").dp.sum((0, 1))]),
        ...     global_scale=1.0
        ... )

        This function extracts utility information about each aggregate in the resulting data frame:

        >>> dp.summarize_polars_measurement(meas)
        shape: (2, 4)
        ┌────────┬──────────────┬─────────────────┬───────┐
        │ column ┆ aggregate    ┆ distribution    ┆ scale │
        │ ---    ┆ ---          ┆ ---             ┆ ---   │
        │ str    ┆ str          ┆ str             ┆ f64   │
        ╞════════╪══════════════╪═════════════════╪═══════╡
        │ len    ┆ Frame Length ┆ Integer Laplace ┆ 1.0   │
        │ A      ┆ Sum          ┆ Integer Laplace ┆ 1.0   │
        └────────┴──────────────┴─────────────────┴───────┘

        If you pass an alpha argument, then you also get accuracy estimates:

        >>> dp.summarize_polars_measurement(meas, alpha=.05)
        shape: (2, 5)
        ┌────────┬──────────────┬─────────────────┬───────┬──────────┐
        │ column ┆ aggregate    ┆ distribution    ┆ scale ┆ accuracy │
        │ ---    ┆ ---          ┆ ---             ┆ ---   ┆ ---      │
        │ str    ┆ str          ┆ str             ┆ f64   ┆ f64      │
        ╞════════╪══════════════╪═════════════════╪═══════╪══════════╡
        │ len    ┆ Frame Length ┆ Integer Laplace ┆ 1.0   ┆ 3.375618 │
        │ A      ┆ Sum          ┆ Integer Laplace ┆ 1.0   ┆ 3.375618 │
        └────────┴──────────────┴─────────────────┴───────┴──────────┘


    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name="AnyMeasurement")
    c_alpha = py_to_c(alpha, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=["f64"]))

    # Call library function.
    lib_function = lib.opendp_accuracy__summarize_polars_measurement
    lib_function.argtypes = [Measurement, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_alpha), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'summarize_polars_measurement',
            '__module__': 'accuracy',
            '__kwargs__': {
                'measurement': measurement, 'alpha': alpha
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output
