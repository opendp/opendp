# Auto-generated. Do not edit!
'''
The ``combinators`` module provides functions for combining transformations and measurements.
For more context, see :ref:`combinators in the User Guide <combinators-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.c``.
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "make_approximate",
    "make_basic_composition",
    "make_chain_mt",
    "make_chain_pm",
    "make_chain_tt",
    "make_fix_delta",
    "make_population_amplification",
    "make_pureDP_to_zCDP",
    "make_select_private_candidate",
    "make_sequential_composition",
    "make_zCDP_to_approxDP",
    "then_sequential_composition"
]


def make_approximate(
    measurement: Measurement
) -> Measurement:
    r"""Constructs a new output measurement where the output measure
    is δ-approximate, where δ=0.


    Required features: `contrib`

    [make_approximate in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_approximate.html)

    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)

    # Call library function.
    lib_function = lib.opendp_combinators__make_approximate
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))

    return output


def make_basic_composition(
    measurements
) -> Measurement:
    r"""Construct the DP composition \[`measurement0`, `measurement1`, ...\].
    Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`

    All metrics and domains must be equivalent.

    **Composition Properties**

    * sequential: all measurements are applied to the same dataset
    * basic: the composition is the linear sum of the privacy usage of each query
    * noninteractive: all mechanisms specified up-front (but each can be interactive)
    * compositor: all privacy parameters specified up-front (via the map)


    Required features: `contrib`

    [make_basic_composition in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_basic_composition.html)

    :param measurements: A vector of Measurements to compose.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurements = py_to_c(measurements, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[AnyMeasurementPtr]))

    # Call library function.
    lib_function = lib.opendp_combinators__make_basic_composition
    lib_function.argtypes = [AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurements), Measurement))
    output._depends_on(get_dependencies_iterable(measurements))
    return output


def make_chain_mt(
    measurement1: Measurement,
    transformation0: Transformation
) -> Measurement:
    r"""Construct the functional composition (`measurement1` ○ `transformation0`).
    Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.


    Required features: `contrib`

    [make_chain_mt in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_chain_mt.html)

    :param measurement1: outer mechanism
    :type measurement1: Measurement
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement1 = py_to_c(measurement1, c_type=Measurement, type_name=None)
    c_transformation0 = py_to_c(transformation0, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_mt
    lib_function.argtypes = [Measurement, Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement1, c_transformation0), Measurement))
    output._depends_on(get_dependencies(measurement1), get_dependencies(transformation0))
    return output


def make_chain_pm(
    postprocess1: Function,
    measurement0: Measurement
) -> Measurement:
    r"""Construct the functional composition (`postprocess1` ○ `measurement0`).
    Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
    Used to represent non-interactive postprocessing.


    Required features: `contrib`

    [make_chain_pm in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_chain_pm.html)

    :param postprocess1: outer postprocessor
    :type postprocess1: Function
    :param measurement0: inner measurement/mechanism
    :type measurement0: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_postprocess1 = py_to_c(postprocess1, c_type=Function, type_name=None)
    c_measurement0 = py_to_c(measurement0, c_type=Measurement, type_name=None)

    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_pm
    lib_function.argtypes = [Function, Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_postprocess1, c_measurement0), Measurement))
    output._depends_on(get_dependencies(postprocess1), get_dependencies(measurement0))
    return output


def make_chain_tt(
    transformation1: Transformation,
    transformation0: Transformation
) -> Transformation:
    r"""Construct the functional composition (`transformation1` ○ `transformation0`).
    Returns a Transformation that when invoked, computes `transformation1(transformation0(x))`.


    Required features: `contrib`

    [make_chain_tt in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_chain_tt.html)

    :param transformation1: outer transformation
    :type transformation1: Transformation
    :param transformation0: inner transformation
    :type transformation0: Transformation
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_transformation1 = py_to_c(transformation1, c_type=Transformation, type_name=None)
    c_transformation0 = py_to_c(transformation0, c_type=Transformation, type_name=None)

    # Call library function.
    lib_function = lib.opendp_combinators__make_chain_tt
    lib_function.argtypes = [Transformation, Transformation]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_transformation1, c_transformation0), Transformation))
    output._depends_on(get_dependencies(transformation1), get_dependencies(transformation0))
    return output


def make_fix_delta(
    measurement: Measurement,
    delta: float
) -> Measurement:
    r"""Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.


    Required features: `contrib`

    [make_fix_delta in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_fix_delta.html)

    :param measurement: a measurement with a privacy curve to be fixed
    :type measurement: Measurement
    :param delta: parameter to fix the privacy curve with
    :type delta: float
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    c_delta = py_to_c(delta, c_type=ctypes.c_double, type_name=f64)

    # Call library function.
    lib_function = lib.opendp_combinators__make_fix_delta
    lib_function.argtypes = [Measurement, ctypes.c_double]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_delta), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output


def make_population_amplification(
    measurement: Measurement,
    population_size: int
) -> Measurement:
    r"""Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
    This measurement does not perform any sampling.
    It is useful when you have a dataset on-hand that is a simple random sample from a larger population.

    The DIA, DO, MI and MO between the input measurement and amplified output measurement all match.


    Required features: `contrib`, `honest-but-curious`

    [make_population_amplification in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_population_amplification.html)

    **Why honest-but-curious?:**

    The privacy guarantees are only valid if the input dataset is a simple sample from a population with `population_size` records.

    :param measurement: the computation to amplify
    :type measurement: Measurement
    :param population_size: the size of the population from which the input dataset is a simple sample
    :type population_size: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    c_population_size = py_to_c(population_size, c_type=ctypes.c_size_t, type_name=usize)

    # Call library function.
    lib_function = lib.opendp_combinators__make_population_amplification
    lib_function.argtypes = [Measurement, ctypes.c_size_t]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_population_size), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output


def make_pureDP_to_zCDP(
    measurement: Measurement
) -> Measurement:
    r"""Constructs a new output measurement where the output measure
    is casted from `MaxDivergence` to `ZeroConcentratedDivergence`.


    Required features: `contrib`

    [make_pureDP_to_zCDP in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_pureDP_to_zCDP.html)

    **Citations:**

    - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)

    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)

    # Call library function.
    lib_function = lib.opendp_combinators__make_pureDP_to_zCDP
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))

    return output


def make_select_private_candidate(
    measurement: Measurement,
    stop_probability: float,
    threshold: float
) -> Measurement:
    r"""Select a private candidate whose score is above a threshold.

    Given `measurement` that satisfies ε-DP, returns new measurement M' that satisfies 2ε-DP.
    M' releases the first invocation of `measurement` whose score is above `threshold`.

    Each time a score is below `threshold`
    the algorithm may terminate with probability `stop_probability` and return nothing.

    `measurement` should make releases in the form of (score, candidate).
    If you are writing a custom scorer measurement in Python,
    specify the output type as `TO=(float, "ExtrinsicObject")`.
    This ensures that the float value is accessible to the algorithm.
    The candidate, left as arbitrary Python data, is held behind the ExtrinsicObject.

    Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).


    Required features: `contrib`

    [make_select_private_candidate in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_select_private_candidate.html)

    **Supporting Elements:**

    * Input Domain:   `DI`
    * Output Type:    `Option<(f64, TO)>`
    * Input Metric:   `MI`
    * Output Measure: `MaxDivergence`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/combinators/select_private_candidate/make_select_private_candidate.pdf)

    :param measurement: A measurement that releases a 2-tuple of (score, candidate)
    :type measurement: Measurement
    :param stop_probability: The probability of stopping early at any iteration.
    :type stop_probability: float
    :param threshold: The threshold score. Return immediately if the score is above this threshold.
    :type threshold: float
    :return: A measurement that returns a release from `measurement` whose score is greater than `threshold`, or none.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    >>> threshold = 23
    >>> space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    ...
    >>> # For demonstration purposes-- construct a measurement that releases
    >>> # a tuple with a differentially private score and value.
    >>> # The tuple released must satisfy the privacy guarantee from the map.
    >>> m_mock = space >> dp.m.then_user_measurement(
    ...     dp.max_divergence(),
    ...     lambda x: (np.random.laplace(loc=x), "arbitrary candidate"),
    ...     lambda d_in: d_in,
    ...     TO="(f64, ExtrinsicObject)"
    ... )
    ...
    >>> m_private_selection = dp.c.make_select_private_candidate(
    ...     m_mock, threshold=threshold, stop_probability=0
    ... )
    ...
    >>> score, candidate = m_private_selection(20)
    ...
    >>> assert score >= threshold
    >>> assert m_private_selection.map(1) == 2 * m_mock.map(1)
    >>> assert isinstance(candidate, str)


    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)
    c_stop_probability = py_to_c(stop_probability, c_type=ctypes.c_double, type_name=f64)
    c_threshold = py_to_c(threshold, c_type=ctypes.c_double, type_name=f64)

    # Call library function.
    lib_function = lib.opendp_combinators__make_select_private_candidate
    lib_function.argtypes = [Measurement, ctypes.c_double, ctypes.c_double]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement, c_stop_probability, c_threshold), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output


def make_sequential_composition(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_mids
) -> Measurement:
    r"""Construct a Measurement that when invoked,
    returns a queryable that interactively composes measurements.

    **Composition Properties**

    * sequential: all measurements are applied to the same dataset
    * basic: the composition is the linear sum of the privacy usage of each query
    * interactive: mechanisms can be specified based on answers to previous queries
    * compositor: all privacy parameters specified up-front

    If the privacy measure supports concurrency,
    this compositor allows you to spawn multiple interactive mechanisms
    and interleave your queries amongst them.


    Required features: `contrib`

    [make_sequential_composition in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_sequential_composition.html)

    **Supporting Elements:**

    * Input Domain:   `DI`
    * Output Type:    `Queryable<Measurement<DI, TO, MI, MO>, TO>`
    * Input Metric:   `MI`
    * Output Measure: `MO`

    :param input_domain: indicates the space of valid input datasets
    :type input_domain: Domain
    :param input_metric: how distances are measured between members of the input domain
    :type input_metric: Metric
    :param output_measure: how privacy is measured
    :type output_measure: Measure
    :param d_in: maximum distance between adjacent input datasets
    :param d_mids: maximum privacy expenditure of each query
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = get_distance_type(output_measure) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=None)
    c_d_in = py_to_c(d_in, c_type=AnyObjectPtr, type_name=get_distance_type(input_metric))
    c_d_mids = py_to_c(d_mids, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[QO]))

    # Call library function.
    lib_function = lib.opendp_combinators__make_sequential_composition
    lib_function.argtypes = [Domain, Metric, Measure, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_d_in, c_d_mids), Measurement))

    return output

def then_sequential_composition(
    output_measure: Measure,
    d_in,
    d_mids
):  
    r"""partial constructor of make_sequential_composition

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.combinators.make_sequential_composition`

    :param output_measure: how privacy is measured
    :type output_measure: Measure
    :param d_in: maximum distance between adjacent input datasets
    :param d_mids: maximum privacy expenditure of each query
    """
    return PartialConstructor(lambda input_domain, input_metric: make_sequential_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=d_in,
        d_mids=d_mids))



def make_zCDP_to_approxDP(
    measurement: Measurement
) -> Measurement:
    r"""Constructs a new output measurement where the output measure
    is casted from `ZeroConcentratedDivergence` to `SmoothedMaxDivergence`.


    Required features: `contrib`

    [make_zCDP_to_approxDP in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/combinators/fn.make_zCDP_to_approxDP.html)

    :param measurement: a measurement with a privacy measure to be casted
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=AnyMeasurement)

    # Call library function.
    lib_function = lib.opendp_combinators__make_zCDP_to_approxDP
    lib_function.argtypes = [Measurement]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measurement), Measurement))
    output._depends_on(get_dependencies(measurement))
    return output
