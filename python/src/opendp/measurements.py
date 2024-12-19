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
    "debias_randomized_response_bitvec",
    "make_alp_queryable",
    "make_gaussian",
    "make_geometric",
    "make_laplace",
    "make_laplace_threshold",
    "make_private_expr",
    "make_private_lazyframe",
    "make_randomized_response",
    "make_randomized_response_bitvec",
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
    "then_randomized_response_bitvec",
    "then_report_noisy_max_gumbel",
    "then_user_measurement"
]


def debias_randomized_response_bitvec(
    answers,
    f: float
):
    r"""Convert a vector of randomized response bitvec responses to a frequency estimate


    Required features: `contrib`

    [debias_randomized_response_bitvec in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.debias_randomized_response_bitvec.html)

    :param answers: A vector of BitVectors with consistent size
    :param f: The per bit flipping probability used to encode `answers`

    Computes the sum of the answers into a $k$-length vector $Y$ and returns
    ```math
    Y\frac{Y-\frac{f}{2}}{1-f}
    ```
    :type f: float
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_answers = py_to_c(answers, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[BitVector]))
    c_f = py_to_c(f, c_type=ctypes.c_double, type_name=f64)

    # Call library function.
    lib_function = lib.opendp_measurements__debias_randomized_response_bitvec
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_double]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_answers, c_f), AnyObjectPtr))

    return output


def make_alp_queryable(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    total_limit,
    value_limit = None,
    size_factor = 50,
    alpha = 4
) -> Measurement:
    r"""Measurement to release a queryable containing a DP projection of bounded sparse data.

    The size of the projection is O(total * size_factor * scale / alpha).
    The evaluation time of post-processing is O(beta * scale / alpha).

    `size_factor` is an optional multiplier (defaults to 50) for setting the size of the projection.
    There is a memory/utility trade-off.
    The value should be sufficiently large to limit hash collisions.


    Required features: `contrib`

    [make_alp_queryable in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_alp_queryable.html)

    **Citations:**

    * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4

    **Supporting Elements:**

    * Input Domain:   `MapDomain<AtomDomain<K>, AtomDomain<CI>>`
    * Output Type:    `Queryable<K, f64>`
    * Input Metric:   `L1Distance<CI>`
    * Output Measure: `MaxDivergence`

    :param input_domain: 
    :type input_domain: Domain
    :param input_metric: 
    :type input_metric: Metric
    :param scale: Privacy loss parameter. This is equal to epsilon/sensitivity.
    :type scale: float
    :param total_limit: Either the true value or an upper bound estimate of the sum of all values in the input.
    :param value_limit: Upper bound on individual values (referred to as β). Entries above β are clamped.
    :param size_factor: Optional multiplier (default of 50) for setting the size of the projection.
    :param alpha: Optional parameter (default of 4) for scaling and determining p in randomized response step.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    CI = get_value_type(get_carrier_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_total_limit = py_to_c(total_limit, c_type=ctypes.c_void_p, type_name=CI)
    c_value_limit = py_to_c(value_limit, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[CI]))
    c_size_factor = py_to_c(size_factor, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))
    c_alpha = py_to_c(alpha, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[u32]))

    # Call library function.
    lib_function = lib.opendp_measurements__make_alp_queryable
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_total_limit, c_value_limit, c_size_factor, c_alpha), Measurement))

    return output

def then_alp_queryable(
    scale: float,
    total_limit,
    value_limit = None,
    size_factor = 50,
    alpha = 4
):  
    r"""partial constructor of make_alp_queryable

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_alp_queryable`

    :param scale: Privacy loss parameter. This is equal to epsilon/sensitivity.
    :type scale: float
    :param total_limit: Either the true value or an upper bound estimate of the sum of all values in the input.
    :param value_limit: Upper bound on individual values (referred to as β). Entries above β are clamped.
    :param size_factor: Optional multiplier (default of 50) for setting the size of the projection.
    :param alpha: Optional parameter (default of 4) for scaling and determining p in randomized response step.
    """
    return PartialConstructor(lambda input_domain, input_metric: make_alp_queryable(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        total_limit=total_limit,
        value_limit=value_limit,
        size_factor=size_factor,
        alpha=alpha))



def make_gaussian(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    k = None,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence"
) -> Measurement:
    r"""Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`          |
    | ------------------------------- | ------------ | ----------------------- |
    | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |


    Required features: `contrib`

    [make_gaussian in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_gaussian.html)

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
    :type scale: float
    :param k: The noise granularity in terms of 2^k.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
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

    Or, more readably, define the space and then chain:

    >>> gaussian = input_space >> dp.m.then_gaussian(scale=1.0)
    >>> print('100?', gaussian(100.0))
    100? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_k = py_to_c(k, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[i32]))
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_gaussian
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k, c_MO), Measurement))

    return output

def then_gaussian(
    scale: float,
    k = None,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence"
):  
    r"""partial constructor of make_gaussian

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_gaussian`

    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :type scale: float
    :param k: The noise granularity in terms of 2^k.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`

    :example:

    >>> dp.enable_features('contrib')
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> gaussian = dp.m.make_gaussian(*input_space, scale=1.0)
    >>> print('100?', gaussian(100.0))
    100? ...

    Or, more readably, define the space and then chain:

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
    scale: float,
    bounds = None
) -> Measurement:
    r"""Equivalent to `make_laplace` but restricted to an integer support.
    Can specify `bounds` to run the algorithm in near constant-time.


    Required features: `contrib`

    [make_geometric in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_geometric.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence`

    :param input_domain: 
    :type input_domain: Domain
    :param input_metric: 
    :type input_metric: Metric
    :param scale: 
    :type scale: float
    :param bounds: 
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

    Or, more readably, define the space and then chain:

    >>> geometric = input_space >> dp.m.then_geometric(scale=1.0)
    >>> print('100?', geometric(100))
    100? ...

    """
    assert_features("contrib")

    # Standardize type arguments.
    T = get_atom(get_carrier_type(input_domain)) # type: ignore
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])]) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=OptionT)

    # Call library function.
    lib_function = lib.opendp_measurements__make_geometric
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_bounds), Measurement))

    return output

def then_geometric(
    scale: float,
    bounds = None
):  
    r"""partial constructor of make_geometric

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_geometric`

    :param scale: 
    :type scale: float
    :param bounds: 

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    >>> geometric = dp.m.make_geometric(*input_space, scale=1.0)
    >>> print('100?', geometric(100))
    100? ...

    Or, more readably, define the space and then chain:

    >>> geometric = input_space >> dp.m.then_geometric(scale=1.0)
    >>> print('100?', geometric(100))
    100? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_geometric(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds))



def make_laplace(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    k = None
) -> Measurement:
    r"""Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    Internally, all sampling is done using the discrete Laplace distribution.


    Required features: `contrib`

    [make_laplace in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_laplace.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence`

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    :type scale: float
    :param k: The noise granularity in terms of 2^k, only valid for domains over floats.
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

    Or, more readably, define the space and then chain:

    >>> laplace = input_space >> dp.m.then_laplace(scale=1.0)
    >>> print('100?', laplace(100.0))
    100? ...

    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_k = py_to_c(k, c_type=ctypes.c_void_p, type_name=RuntimeType(origin='Option', args=[i32]))

    # Call library function.
    lib_function = lib.opendp_measurements__make_laplace
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_void_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k), Measurement))

    return output

def then_laplace(
    scale: float,
    k = None
):  
    r"""partial constructor of make_laplace

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_laplace`

    :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    :type scale: float
    :param k: The noise granularity in terms of 2^k, only valid for domains over floats.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    >>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    >>> laplace = dp.m.make_laplace(*input_space, scale=1.0)
    >>> print('100?', laplace(100.0))
    100? ...

    Or, more readably, define the space and then chain:

    >>> laplace = input_space >> dp.m.then_laplace(scale=1.0)
    >>> print('100?', laplace(100.0))
    100? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k))



def make_laplace_threshold(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    threshold,
    k: int = -1074
) -> Measurement:
    r"""Make a Measurement that uses propose-test-release to privatize a hashmap of counts.

    This function takes a noise granularity in terms of 2^k.
    Larger granularities are more computationally efficient, but have a looser privacy map.
    If k is not set, k defaults to the smallest granularity.


    Required features: `contrib`, `floating-point`

    [make_laplace_threshold in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_laplace_threshold.html)

    **Supporting Elements:**

    * Input Domain:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
    * Output Type:    `HashMap<TK, TV>`
    * Input Metric:   `L1Distance<TV>`
    * Output Measure: `Approximate<MaxDivergence>`

    :param input_domain: Domain of the input.
    :type input_domain: Domain
    :param input_metric: Metric for the input domain.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :type scale: float
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
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=TV)
    c_k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)

    # Call library function.
    lib_function = lib.opendp_measurements__make_laplace_threshold
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_void_p, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_threshold, c_k), Measurement))

    return output

def then_laplace_threshold(
    scale: float,
    threshold,
    k: int = -1074
):  
    r"""partial constructor of make_laplace_threshold

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_laplace_threshold`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :type scale: float
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


    Required features: `contrib`, `honest-but-curious`

    [make_private_expr in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_private_expr.html)

    **Why honest-but-curious?:**

    The privacy guarantee governs only at most one evaluation of the released expression.

    **Supporting Elements:**

    * Input Domain:   `WildExprDomain`
    * Output Type:    `ExprPlan`
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
    assert_features("contrib", "honest-but-curious")

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
    lazyframe,
    global_scale = None,
    threshold = None
) -> Measurement:
    r"""Create a differentially private measurement from a [`LazyFrame`].

    Any data inside the [`LazyFrame`] is ignored,
    but it is still recommended to start with an empty [`DataFrame`] and build up the computation using the [`LazyFrame`] API.


    Required features: `contrib`

    [make_private_lazyframe in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_private_lazyframe.html)

    **Supporting Elements:**

    * Input Domain:   `LazyFrameDomain`
    * Output Type:    `OnceFrame`
    * Input Metric:   `MI`
    * Output Measure: `MO`

    :param input_domain: The domain of the input data.
    :type input_domain: Domain
    :param input_metric: How to measure distances between neighboring input data sets.
    :type input_metric: Metric
    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param lazyframe: A description of the computations to be run, in the form of a [`LazyFrame`].
    :param global_scale: Optional. A tune-able parameter that affects the privacy-utility tradeoff.
    :param threshold: Optional. Minimum number of rows in each released partition.
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> import polars as pl

    We'll imagine an elementary school is taking a pet census.
    The private census data will have two columns: 

    >>> lf_domain = dp.lazyframe_domain([
    ...     dp.series_domain("grade", dp.atom_domain(T=dp.i32)),
    ...     dp.series_domain("pet_count", dp.atom_domain(T=dp.i32))])

    We also need to specify the column we'll be grouping by.

    >>> lf_domain_with_margin = dp.with_margin(
    ...     lf_domain,
    ...     by=["grade"],
    ...     public_info="keys",
    ...     max_partition_length=50)

    With that in place, we can plan the Polars computation, using the `dp` plugin. 

    >>> plan = (
    ...     pl.LazyFrame(schema={'grade': pl.Int32, 'pet_count': pl.Int32})
    ...     .group_by("grade")
    ...     .agg(pl.col("pet_count").dp.sum((0, 10), scale=1.0)))

    We now have all the pieces to make our measurement function using `make_private_lazyframe`:

    >>> dp_sum_pets_by_grade = dp.m.make_private_lazyframe(
    ...     input_domain=lf_domain_with_margin,
    ...     input_metric=dp.symmetric_distance(),
    ...     output_measure=dp.max_divergence(),
    ...     lazyframe=plan,
    ...     global_scale=1.0)

    It's only at this point that we need to introduce the private data.

    >>> df = pl.from_records(
    ...     [
    ...         [0, 0], # No kindergarteners with pets.
    ...         [0, 0],
    ...         [0, 0],
    ...         [1, 1], # Each first grader has 1 pet.
    ...         [1, 1],
    ...         [1, 1],
    ...         [2, 1], # One second grader has chickens!
    ...         [2, 1],
    ...         [2, 9]
    ...     ],
    ...     schema=['grade', 'pet_count'], orient="row")
    >>> lf = pl.LazyFrame(df)
    >>> results = dp_sum_pets_by_grade(lf).collect()
    >>> print(results.sort("grade")) # doctest: +ELLIPSIS
    shape: (3, 2)
    ┌───────┬───────────┐
    │ grade ┆ pet_count │
    │ ---   ┆ ---       │
    │ i64   ┆ i64       │
    ╞═══════╪═══════════╡
    │ 0     ┆ ...       │
    │ 1     ┆ ...       │
    │ 2     ┆ ...       │
    └───────┴───────────┘

    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_output_measure = py_to_c(output_measure, c_type=Measure, type_name=None)
    c_lazyframe = py_to_c(lazyframe, c_type=AnyObjectPtr, type_name=LazyFrame)
    c_global_scale = py_to_c(global_scale, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[f64]))
    c_threshold = py_to_c(threshold, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Option', args=[u32]))

    # Call library function.
    lib_function = lib.opendp_measurements__make_private_lazyframe
    lib_function.argtypes = [Domain, Metric, Measure, AnyObjectPtr, AnyObjectPtr, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_measure, c_lazyframe, c_global_scale, c_threshold), Measurement))

    return output

def then_private_lazyframe(
    output_measure: Measure,
    lazyframe,
    global_scale = None,
    threshold = None
):  
    r"""partial constructor of make_private_lazyframe

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_private_lazyframe`

    :param output_measure: How to measure privacy loss.
    :type output_measure: Measure
    :param lazyframe: A description of the computations to be run, in the form of a [`LazyFrame`].
    :param global_scale: Optional. A tune-able parameter that affects the privacy-utility tradeoff.
    :param threshold: Optional. Minimum number of rows in each released partition.

    :example:

    >>> dp.enable_features("contrib")
    >>> import polars as pl

    We'll imagine an elementary school is taking a pet census.
    The private census data will have two columns: 

    >>> lf_domain = dp.lazyframe_domain([
    ...     dp.series_domain("grade", dp.atom_domain(T=dp.i32)),
    ...     dp.series_domain("pet_count", dp.atom_domain(T=dp.i32))])

    We also need to specify the column we'll be grouping by.

    >>> lf_domain_with_margin = dp.with_margin(
    ...     lf_domain,
    ...     by=["grade"],
    ...     public_info="keys",
    ...     max_partition_length=50)

    With that in place, we can plan the Polars computation, using the `dp` plugin. 

    >>> plan = (
    ...     pl.LazyFrame(schema={'grade': pl.Int32, 'pet_count': pl.Int32})
    ...     .group_by("grade")
    ...     .agg(pl.col("pet_count").dp.sum((0, 10), scale=1.0)))

    We now have all the pieces to make our measurement function using `make_private_lazyframe`:

    >>> dp_sum_pets_by_grade = dp.m.make_private_lazyframe(
    ...     input_domain=lf_domain_with_margin,
    ...     input_metric=dp.symmetric_distance(),
    ...     output_measure=dp.max_divergence(),
    ...     lazyframe=plan,
    ...     global_scale=1.0)

    It's only at this point that we need to introduce the private data.

    >>> df = pl.from_records(
    ...     [
    ...         [0, 0], # No kindergarteners with pets.
    ...         [0, 0],
    ...         [0, 0],
    ...         [1, 1], # Each first grader has 1 pet.
    ...         [1, 1],
    ...         [1, 1],
    ...         [2, 1], # One second grader has chickens!
    ...         [2, 1],
    ...         [2, 9]
    ...     ],
    ...     schema=['grade', 'pet_count'], orient="row")
    >>> lf = pl.LazyFrame(df)
    >>> results = dp_sum_pets_by_grade(lf).collect()
    >>> print(results.sort("grade")) # doctest: +ELLIPSIS
    shape: (3, 2)
    ┌───────┬───────────┐
    │ grade ┆ pet_count │
    │ ---   ┆ ---       │
    │ i64   ┆ i64       │
    ╞═══════╪═══════════╡
    │ 0     ┆ ...       │
    │ 1     ┆ ...       │
    │ 2     ┆ ...       │
    └───────┴───────────┘

    """
    return PartialConstructor(lambda input_domain, input_metric: make_private_lazyframe(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        lazyframe=lazyframe,
        global_scale=global_scale,
        threshold=threshold))



def make_randomized_response(
    categories,
    prob: float,
    T: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that implements randomized response on a categorical value.


    Required features: `contrib`

    [make_randomized_response in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_randomized_response.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<T>`
    * Output Type:    `T`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response/make_randomized_response.pdf)

    :param categories: Set of valid outcomes
    :param prob: Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
    :type prob: float
    :param T: Data type of a category.
    :type T: :py:ref:`RuntimeTypeDescriptor`
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

    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[T]))
    c_prob = py_to_c(prob, c_type=ctypes.c_double, type_name=f64)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_double, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories, c_prob, c_T), Measurement))

    return output


def make_randomized_response_bitvec(
    input_domain: Domain,
    input_metric: Metric,
    f: float,
    constant_time: bool = False
) -> Measurement:
    r"""Make a Measurement that implements randomized response on a bit vector.

    This primitive can be useful for implementing RAPPOR.


    Required features: `contrib`

    [make_randomized_response_bitvec in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_randomized_response_bitvec.html)

    **Citations:**

    * [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal Response](https://arxiv.org/abs/1407.6981)

    **Supporting Elements:**

    * Input Domain:   `BitVectorDomain`
    * Output Type:    `BitVector`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response_bitvec/make_randomized_response_bitvec.pdf)

    :param input_domain: BitVectorDomain with max_weight
    :type input_domain: Domain
    :param input_metric: DiscreteDistance
    :type input_metric: Metric
    :param f: Per-bit flipping probability. Must be in $(0, 1]$.
    :type f: float
    :param constant_time: Whether to run the Bernoulli samplers in constant time, this is likely to be extremely slow.
    :type constant_time: bool
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> import numpy as np
    >>> import opendp.prelude as dp

    >>> dp.enable_features("contrib")

    >>> # Create the randomized response mechanism
    >>> m_rr = dp.m.make_randomized_response_bitvec(
    ...     dp.bitvector_domain(max_weight=4), dp.discrete_distance(), f=0.95
    ... )

    >>> # compute privacy loss
    >>> m_rr.map(1)
    0.8006676684558611

    >>> # formula is 2 * m * ln((2 - f) / f)
    >>> # where m = 4 (the weight) and f = .95 (the flipping probability)

    >>> # prepare a dataset to release, by encoding a bit vector as a numpy byte array
    >>> data = np.packbits(
    ...     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
    ... )
    >>> assert np.array_equal(data, np.array([0, 8, 12], dtype=np.uint8))

    >>> # roundtrip: numpy -> bytes -> mech -> bytes -> numpy
    >>> release = np.frombuffer(m_rr(data.tobytes()), dtype=np.uint8)

    >>> # compare the two bit vectors:
    >>> [int(bit) for bit in np.unpackbits(data)]
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
    >>> [int(bit) for bit in np.unpackbits(release)]
    [...]

    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_f = py_to_c(f, c_type=ctypes.c_double, type_name=f64)
    c_constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response_bitvec
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_bool]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_f, c_constant_time), Measurement))

    return output

def then_randomized_response_bitvec(
    f: float,
    constant_time: bool = False
):  
    r"""partial constructor of make_randomized_response_bitvec

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_randomized_response_bitvec`

    :param f: Per-bit flipping probability. Must be in $(0, 1]$.
    :type f: float
    :param constant_time: Whether to run the Bernoulli samplers in constant time, this is likely to be extremely slow.
    :type constant_time: bool

    :example:

    >>> import numpy as np
    >>> import opendp.prelude as dp

    >>> dp.enable_features("contrib")

    >>> # Create the randomized response mechanism
    >>> m_rr = dp.m.make_randomized_response_bitvec(
    ...     dp.bitvector_domain(max_weight=4), dp.discrete_distance(), f=0.95
    ... )

    >>> # compute privacy loss
    >>> m_rr.map(1)
    0.8006676684558611

    >>> # formula is 2 * m * ln((2 - f) / f)
    >>> # where m = 4 (the weight) and f = .95 (the flipping probability)

    >>> # prepare a dataset to release, by encoding a bit vector as a numpy byte array
    >>> data = np.packbits(
    ...     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
    ... )
    >>> assert np.array_equal(data, np.array([0, 8, 12], dtype=np.uint8))

    >>> # roundtrip: numpy -> bytes -> mech -> bytes -> numpy
    >>> release = np.frombuffer(m_rr(data.tobytes()), dtype=np.uint8)

    >>> # compare the two bit vectors:
    >>> [int(bit) for bit in np.unpackbits(data)]
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
    >>> [int(bit) for bit in np.unpackbits(release)]
    [...]

    """
    return PartialConstructor(lambda input_domain, input_metric: make_randomized_response_bitvec(
        input_domain=input_domain,
        input_metric=input_metric,
        f=f,
        constant_time=constant_time))



def make_randomized_response_bool(
    prob: float,
    constant_time: bool = False
) -> Measurement:
    r"""Make a Measurement that implements randomized response on a boolean value.


    Required features: `contrib`

    [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_randomized_response_bool.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<bool>`
    * Output Type:    `bool`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response/make_randomized_response_bool.pdf)

    :param prob: Probability of returning the correct answer. Must be in `[0.5, 1)`
    :type prob: float
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
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

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_prob = py_to_c(prob, c_type=ctypes.c_double, type_name=f64)
    c_constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response_bool
    lib_function.argtypes = [ctypes.c_double, ctypes.c_bool]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_prob, c_constant_time), Measurement))

    return output


def make_report_noisy_max_gumbel(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    optimize: str
) -> Measurement:
    r"""Make a Measurement that takes a vector of scores and privately selects the index of the highest score.


    Required features: `contrib`

    [make_report_noisy_max_gumbel in Rust documentation.](https://docs.rs/opendp/0.12.0-beta.20241219.1/opendp/measurements/fn.make_report_noisy_max_gumbel.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Type:    `usize`
    * Input Metric:   `LInfDistance<TIA>`
    * Output Measure: `MaxDivergence`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/gumbel_max/make_report_noisy_max_gumbel.pdf)

    :param input_domain: Domain of the input vector. Must be a non-nullable VectorDomain.
    :type input_domain: Domain
    :param input_metric: Metric on the input domain. Must be LInfDistance
    :type input_metric: Metric
    :param scale: Higher scales are more private.
    :type scale: float
    :param optimize: Indicate whether to privately return the "max" or "min"
    :type optimize: str
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
    >>> select_index = dp.m.make_report_noisy_max_gumbel(*input_space, scale=1.0, optimize='max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    Or, more readably, define the space and then chain:

    >>> select_index = input_space >> dp.m.then_report_noisy_max_gumbel(scale=1.0, optimize='max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_double, type_name=f64)
    c_optimize = py_to_c(optimize, c_type=ctypes.c_char_p, type_name=String)

    # Call library function.
    lib_function = lib.opendp_measurements__make_report_noisy_max_gumbel
    lib_function.argtypes = [Domain, Metric, ctypes.c_double, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_optimize), Measurement))

    return output

def then_report_noisy_max_gumbel(
    scale: float,
    optimize: str
):  
    r"""partial constructor of make_report_noisy_max_gumbel

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_report_noisy_max_gumbel`

    :param scale: Higher scales are more private.
    :type scale: float
    :param optimize: Indicate whether to privately return the "max" or "min"
    :type optimize: str

    :example:

    >>> dp.enable_features("contrib")
    >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
    >>> select_index = dp.m.make_report_noisy_max_gumbel(*input_space, scale=1.0, optimize='max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    Or, more readably, define the space and then chain:

    >>> select_index = input_space >> dp.m.then_report_noisy_max_gumbel(scale=1.0, optimize='max')
    >>> print('2?', select_index([1, 2, 3, 2, 1]))
    2? ...

    """
    return PartialConstructor(lambda input_domain, input_metric: make_report_noisy_max_gumbel(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        optimize=optimize))



def make_user_measurement(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor = "ExtrinsicObject"
) -> Measurement:
    r"""Construct a Measurement from user-defined callbacks.


    Required features: `contrib`, `honest-but-curious`

    **Why honest-but-curious?:**

    This constructor only returns a valid measurement if for every pair of elements $x, x'$ in `input_domain`,
    and for every pair `(d_in, d_out)`,
    where `d_in` has the associated type for `input_metric` and `d_out` has the associated type for `output_measure`,
    if $x, x'$ are `d_in`-close under `input_metric`, `privacy_map(d_in)` does not raise an exception,
    and `privacy_map(d_in) <= d_out`,
    then `function(x), function(x')` are d_out-close under `output_measure`.

    In addition, `function` must not have side-effects, and `privacy_map` must be a pure function.

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
    ...     output_measure=dp.max_divergence(),
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
    c_function = py_to_c(function, c_type=CallbackFnPtr, type_name=pass_through(TO))
    c_privacy_map = py_to_c(privacy_map, c_type=CallbackFnPtr, type_name=measure_distance_type(output_measure))
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_user_measurement
    lib_function.argtypes = [Domain, Metric, Measure, CallbackFnPtr, CallbackFnPtr, ctypes.c_char_p]
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

    :example:

    >>> dp.enable_features("contrib")
    >>> def const_function(_arg):
    ...     return 42
    >>> def privacy_map(_d_in):
    ...     return 0.
    >>> space = dp.atom_domain(T=int), dp.absolute_distance(int)
    >>> user_measurement = dp.m.make_user_measurement(
    ...     *space,
    ...     output_measure=dp.max_divergence(),
    ...     function=const_function,
    ...     privacy_map=privacy_map
    ... )
    >>> print('42?', user_measurement(0))
    42? 42



    """
    return PartialConstructor(lambda input_domain, input_metric: make_user_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=function,
        privacy_map=privacy_map,
        TO=TO))

