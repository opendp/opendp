# Auto-generated. Do not edit!
'''
The ``measurements`` module provides functions that apply calibrated noise to data to ensure differential privacy.
For more context, see :ref:`measurements in the User Guide <measurements-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.m``.
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
import polars
__all__ = [
    "make_alp_queryable",
    "make_gaussian",
    "make_geometric",
    "make_laplace",
    "make_laplace_threshold",
    "make_private_expr",
    "make_private_lazyframe",
    "make_randomized_response",
    "make_randomized_response_bool",
    "make_report_noisy_max_gumbel",
    "make_user_measurement",
    "then_alp_queryable",
    "then_gaussian",
    "then_geometric",
    "then_laplace",
    "then_laplace_threshold",
    "then_private_expr",
    "then_private_lazyframe",
    "then_report_noisy_max_gumbel",
    "then_user_measurement"
]


def make_alp_queryable(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    total_limit,
    value_limit = None,
    size_factor = 50,
    alpha = 4,
    CO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Measurement to release a queryable containing a DP projection of bounded sparse data.

    The size of the projection is O(total * size_factor * scale / alpha).
    The evaluation time of post-processing is O(beta * scale / alpha).

    `size_factor` is an optional multiplier (defaults to 50) for setting the size of the projection.
    There is a memory/utility trade-off.
    The value should be sufficiently large to limit hash collisions.

    [make_alp_queryable in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_alp_queryable.html)

    **Citations:**

    * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4

    **Supporting Elements:**

    * Input Domain:   `MapDomain<AtomDomain<K>, AtomDomain<CI>>`
    * Output Type:    `Queryable<K, CO>`
    * Input Metric:   `L1Distance<CI>`
    * Output Measure: `MaxDivergence<CO>`

    :param input_domain: 
    :type input_domain: Domain
    :param input_metric: 
    :type input_metric: Metric
    :param scale: Privacy loss parameter. This is equal to epsilon/sensitivity.
    :param total_limit: Either the true value or an upper bound estimate of the sum of all values in the input.
    :param value_limit: Upper bound on individual values (referred to as β). Entries above β are clamped.
    :param size_factor: Optional multiplier (default of 50) for setting the size of the projection.
    :param alpha: Optional parameter (default of 4) for scaling and determining p in randomized response step.
    :param CO: 
    :type CO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    CO = RuntimeType.parse_or_infer(type_name=CO, public_example=scale)
    CI = get_value_type(get_carrier_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=CO)
    c_total_limit = py_to_c(total_limit, c_type=ctypes.c_void_p, type_name=CI)
    c_value_limit = py_to_c(value_limit, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[CI]))
    c_size_factor = py_to_c(size_factor, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_CO = py_to_c(CO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_alp_queryable
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_total_limit, c_value_limit, c_size_factor, c_alpha, c_CO), Measurement))

    return output

def then_alp_queryable(
    scale,
    total_limit,
    value_limit = None,
    size_factor = 50,
    alpha = 4,
    CO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_alp_queryable

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_alp_queryable`

    :param scale: Privacy loss parameter. This is equal to epsilon/sensitivity.
    :param total_limit: Either the true value or an upper bound estimate of the sum of all values in the input.
    :param value_limit: Upper bound on individual values (referred to as β). Entries above β are clamped.
    :param size_factor: Optional multiplier (default of 50) for setting the size of the projection.
    :param alpha: Optional parameter (default of 4) for scaling and determining p in randomized response step.
    :param CO: 
    :type CO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_alp_queryable(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        total_limit=total_limit,
        value_limit=value_limit,
        size_factor=size_factor,
        alpha=alpha,
        CO=CO))



def make_gaussian(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    k = None,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
) -> Measurement:
    r"""Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`          |
    | ------------------------------- | ------------ | ----------------------- |
    | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |

    [make_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_gaussian.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MO`

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features('contrib')
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> gaussian = dp.m.make_gaussian(*input_space, scale=1.0)
    >>> print('100?', gaussian(100.0))
    100? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["QO"])
    QO = get_atom_or_infer(MO, scale) # type: ignore
    MO = MO.substitute(QO=QO) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    c_k = py_to_c(k, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[i32]))
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_gaussian
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k, c_MO), Measurement))

    return output

def then_gaussian(
    scale,
    k = None,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
):  
    r"""partial constructor of make_gaussian

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_gaussian`

    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`

    :example:

    >>> dp.enable_features('contrib')
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> gaussian = input_space >> dp.m.then_gaussian(scale=1.0)
    >>> print('100?', gaussian(100.0))
    100? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k,
        MO=MO))



def make_geometric(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    bounds = None,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Equivalent to `make_laplace` but restricted to an integer support.
    Can specify `bounds` to run the algorithm in near constant-time.

    [make_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_geometric.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`

    :param input_domain: 
    :type input_domain: Domain
    :param input_metric: 
    :type input_metric: Metric
    :param scale: 
    :param bounds: 
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    >>> geometric = dp.m.make_geometric(*input_space, scale=1.0)
    >>> print('100?', geometric(100))
    100? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_atom(get_carrier_type(input_domain)) # type: ignore
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])]) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=OptionT)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_geometric
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_bounds, c_QO), Measurement))

    return output

def then_geometric(
    scale,
    bounds = None,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_geometric

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_geometric`

    :param scale: 
    :param bounds: 
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    >>> geometric = input_space >> dp.m.then_geometric(scale=1.0)
    >>> print('100?', geometric(100))
    100? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_geometric(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds,
        QO=QO))



def make_laplace(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    k = None,
    QO: RuntimeTypeDescriptor = "float"
) -> Measurement:
    r"""Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    Internally, all sampling is done using the discrete Laplace distribution.

    [make_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_laplace.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param k: The noise granularity in terms of 2^k, only valid for domains over floats.
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> laplace = dp.m.make_laplace(*input_space, scale=1.0)
    >>> print('100?', laplace(100.0))
    100? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse(type_name=QO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=get_atom(QO))
    c_k = py_to_c(k, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[i32]))
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_laplace
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k, c_QO), Measurement))

    return output

def then_laplace(
    scale,
    k = None,
    QO: RuntimeTypeDescriptor = "float"
):  
    r"""partial constructor of make_laplace

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_laplace`

    :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param k: The noise granularity in terms of 2^k, only valid for domains over floats.
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`

    :example:

    >>> dp.enable_features('contrib')
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> laplace = input_space >> dp.m.then_laplace(scale=1.0)
    >>> print('100?', laplace(100.0))
    100? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k,
        QO=QO))



def make_laplace_threshold(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    threshold,
    k: int = -1074
) -> Measurement:
    r"""Make a Measurement that uses propose-test-release to privatize a hashmap of counts.

    This function takes a noise granularity in terms of 2^k.
    Larger granularities are more computationally efficient, but have a looser privacy map.
    If k is not set, k defaults to the smallest granularity.

    [make_laplace_threshold in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_laplace_threshold.html)

    **Supporting Elements:**

    * Input Domain:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
    * Output Type:    `HashMap<TK, TV>`
    * Input Metric:   `L1Distance<TV>`
    * Output Measure: `FixedSmoothedMaxDivergence<TV>`

    :param input_domain: Domain of the input.
    :type input_domain: Domain
    :param input_metric: Metric for the input domain.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param threshold: Exclude counts that are less than this minimum value.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "floating-point")

    # Standardize type arguments.
    TV = get_distance_type(input_metric) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=TV)
    c_threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=TV)
    c_k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)

    # Call library function.
    lib_function = lib.opendp_measurements__make_laplace_threshold
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_threshold, c_k), Measurement))

    return output

def then_laplace_threshold(
    scale,
    threshold,
    k: int = -1074
):  
    r"""partial constructor of make_laplace_threshold

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_laplace_threshold`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param threshold: Exclude counts that are less than this minimum value.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    """
    return PartialConstructor(lambda input_domain, input_metric: make_laplace_threshold(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        threshold=threshold,
        k=k))



def make_private_expr(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    expr,
    global_scale = None
) -> Measurement:
    r"""Create a differentially private measurement from an [`Expr`].

    [make_private_expr in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_private_expr.html)

    **Supporting Elements:**

    * Input Domain:   `ExprDomain`
    * Output Type:    `Expr`
    * Input Metric:   `MI`
    * Output Measure: `MO`

    :param input_domain: The domain of the input data.
    :type input_domain: Domain
    :param input_metric: How to measure distances between neighboring input data sets.
    :type input_metric: Metric
    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param expr: The [`Expr`] to be privatized.
    :param global_scale: A tune-able parameter that affects the privacy-utility tradeoff.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=None)
    c_expr = py_to_c(expr, c_type=AnyObjectPtr, type_name=Expr)
    c_global_scale = py_to_c(global_scale, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[f64]))

    # Call library function.
    lib_function = lib.opendp_measurements__make_private_expr
    lib_function.argtypes = [Domain, Metric, Measure, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_expr, c_global_scale), Measurement))

    return output

def then_private_expr(
    output_measure: Measure,
    expr,
    global_scale = None
):  
    r"""partial constructor of make_private_expr

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_private_expr`

    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param expr: The [`Expr`] to be privatized.
    :param global_scale: A tune-able parameter that affects the privacy-utility tradeoff.
    """
    return PartialConstructor(lambda input_domain, input_metric: make_private_expr(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        expr=expr,
        global_scale=global_scale))



def make_private_lazyframe(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    lazyframe: polars.LazyFrame,
    global_scale = None
) -> Measurement:
    r"""Create a differentially private measurement from a [`LazyFrame`].

    Any data inside the [`LazyFrame`] is ignored,
    but it is still recommended to start with an empty [`DataFrame`] and build up the computation using the [`LazyFrame`] API.

    [make_private_lazyframe in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_private_lazyframe.html)

    **Supporting Elements:**

    * Input Domain:   `LazyFrameDomain`
    * Output Type:    `LazyFrame`
    * Input Metric:   `MI`
    * Output Measure: `MO`

    :param input_domain: The domain of the input data.
    :type input_domain: Domain
    :param input_metric: How to measure distances between neighboring input data sets.
    :type input_metric: Metric
    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param lazyframe: A description of the computations to be run, in the form of a [`LazyFrame`].
    :param global_scale: A tune-able parameter that affects the privacy-utility tradeoff.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=None)
    c_lazyframe = py_to_c(lazyframe, c_type=AnyObjectPtr, type_name=LazyFrame)
    c_global_scale = py_to_c(global_scale, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[f64]))

    # Call library function.
    lib_function = lib.opendp_measurements__make_private_lazyframe
    lib_function.argtypes = [Domain, Metric, Measure, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_lazyframe, c_global_scale), Measurement))

    return output

def then_private_lazyframe(
    output_measure: Measure,
    lazyframe,
    global_scale = None
):  
    r"""partial constructor of make_private_lazyframe

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_private_lazyframe`

    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param lazyframe: A description of the computations to be run, in the form of a [`LazyFrame`].
    :param global_scale: A tune-able parameter that affects the privacy-utility tradeoff.
    """
    return PartialConstructor(lambda input_domain, input_metric: make_private_lazyframe(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        lazyframe=lazyframe,
        global_scale=global_scale))



def make_randomized_response(
    categories,
    prob,
    constant_time: bool = False,
    T: Optional[RuntimeTypeDescriptor] = None,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that implements randomized response on a categorical value.

    [make_randomized_response in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<T>`
    * Output Type:    `T`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<QO>`

    :param categories: Set of valid outcomes
    :param prob: Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param T: Data type of a category.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of probability and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> random_string = dp.m.make_randomized_response(['a', 'b', 'c'], 0.99)
    >>> print('a?', random_string('a'))
    a? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(categories))
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=prob)

    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[T]))
    c_prob = py_to_c(prob, c_type=ctypes.c_void_p, type_name=QO)
    c_constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_void_p, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories, c_prob, c_constant_time, c_T, c_QO), Measurement))

    return output


def make_randomized_response_bool(
    prob,
    constant_time: bool = False,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that implements randomized response on a boolean value.

    [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response_bool.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<bool>`
    * Output Type:    `bool`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<QO>`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/randomized_response/make_randomized_response_bool.pdf)

    :param prob: Probability of returning the correct answer. Must be in `[0.5, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param QO: Data type of probability and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> random_bool = dp.m.make_randomized_response_bool(0.99)
    >>> print('True?', random_bool(True))
    True? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=prob)

    # Convert arguments to c types.
    c_prob = py_to_c(prob, c_type=ctypes.c_void_p, type_name=QO)
    c_constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response_bool
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_prob, c_constant_time, c_QO), Measurement))

    return output


def make_report_noisy_max_gumbel(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    optimize: str,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that takes a vector of scores and privately selects the index of the highest score.

    [make_report_noisy_max_gumbel in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_report_noisy_max_gumbel.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Type:    `usize`
    * Input Metric:   `LInfDistance<TIA>`
    * Output Measure: `MaxDivergence<QO>`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/gumbel_max/make_report_noisy_max_gumbel.pdf)

    :param input_domain: Domain of the input vector. Must be a non-nullable VectorDomain.
    :type input_domain: Domain
    :param input_metric: Metric on the input domain. Must be LInfDistance
    :type input_metric: Metric
    :param scale: Higher scales are more private.
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
    >>> select_index = dp.m.make_report_noisy_max_gumbel(*input_space, scale=1.0, optimize='Max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=AnyObjectPtr, type_name=QO)
    c_optimize = py_to_c(optimize, c_type=ctypes.c_char_p, type_name=String)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_report_noisy_max_gumbel
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_optimize, c_QO), Measurement))

    return output

def then_report_noisy_max_gumbel(
    scale,
    optimize: str,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_report_noisy_max_gumbel

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_report_noisy_max_gumbel`

    :param scale: Higher scales are more private.
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
    >>> select_index = input_space >> dp.m.then_report_noisy_max_gumbel(scale=1.0, optimize='Max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_report_noisy_max_gumbel(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        optimize=optimize,
        QO=QO))



def make_user_measurement(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor = "ExtrinsicObject"
) -> Measurement:
    r"""Construct a Measurement from user-defined callbacks.

    **Supporting Elements:**

    * Input Domain:   `AnyDomain`
    * Output Type:    `AnyObject`
    * Input Metric:   `AnyMetric`
    * Output Measure: `AnyMeasure`

    :param input_domain: A domain describing the set of valid inputs for the function.
    :type input_domain: Domain
    :param input_metric: The metric from which distances between adjacent inputs are measured.
    :type input_metric: Metric
    :param output_measure: The measure from which distances between adjacent output distributions are measured.
    :type output_measure: Measure
    :param function: A function mapping data from `input_domain` to a release of type `TO`.
    :param privacy_map: A function mapping distances from `input_metric` to `output_measure`.
    :param TO: The data type of outputs from the function.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> def const_function(_arg):
    ...     return 42
    >>> def privacy_map(_d_in):
    ...     return 0.
    >>> space = dp.atom_domain(T=int), dp.absolute_distance(int)
    >>> user_measurement = dp.m.make_user_measurement(
    ...     *space,
    ...     output_measure=dp.max_divergence(float),
    ...     function=const_function,
    ...     privacy_map=privacy_map
    ... )
    >>> print('42?', user_measurement(0))
    42? 42



    """
    assert_features("contrib", "honest-but-curious")

    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=AnyMeasure)
    c_function = py_to_c(function, c_type=CallbackFn, type_name=pass_through(TO))
    c_privacy_map = py_to_c(privacy_map, c_type=CallbackFn, type_name=measure_distance_type(output_measure))
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_user_measurement
    lib_function.argtypes = [Domain, Metric, Measure, CallbackFn, CallbackFn, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_function, c_privacy_map, c_TO), Measurement))
    output._depends_on(input_domain, input_metric, output_measure, c_function, c_privacy_map)
    return output

def then_user_measurement(
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor = "ExtrinsicObject"
):  
    r"""partial constructor of make_user_measurement

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_user_measurement`

    :param output_measure: The measure from which distances between adjacent output distributions are measured.
    :type output_measure: Measure
    :param function: A function mapping data from `input_domain` to a release of type `TO`.
    :param privacy_map: A function mapping distances from `input_metric` to `output_measure`.
    :param TO: The data type of outputs from the function.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_user_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=function,
        privacy_map=privacy_map,
        TO=TO))

