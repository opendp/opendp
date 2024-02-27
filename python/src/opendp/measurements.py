# Auto-generated. Do not edit!
'''
The ``measurements`` module provides functions that apply calibrated noise to data to ensure differential privacy.
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "make_alp_queryable",
    "make_base_discrete_gaussian",
    "make_base_discrete_laplace",
    "make_base_discrete_laplace_cks20",
    "make_base_discrete_laplace_linear",
    "make_base_gaussian",
    "make_base_geometric",
    "make_base_laplace",
    "make_base_laplace_threshold",
    "make_gaussian",
    "make_laplace",
    "make_randomized_response",
    "make_randomized_response_bool",
    "make_report_noisy_max_gumbel",
    "make_user_measurement",
    "then_alp_queryable",
    "then_base_discrete_gaussian",
    "then_base_discrete_laplace",
    "then_base_discrete_laplace_cks20",
    "then_base_discrete_laplace_linear",
    "then_base_gaussian",
    "then_base_geometric",
    "then_base_laplace",
    "then_base_laplace_threshold",
    "then_gaussian",
    "then_laplace",
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



def make_base_discrete_gaussian(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
) -> Measurement:
    r"""Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |

    [make_base_discrete_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_gaussian.html)

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
    :param MO: Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
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
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_base_discrete_gaussian
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_MO), Measurement))

    return output

def then_base_discrete_gaussian(
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
):  
    r"""partial constructor of make_base_discrete_gaussian

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_discrete_gaussian`

    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param MO: Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        MO=MO))



def make_base_discrete_laplace(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    This uses `make_base_discrete_laplace_cks20` if scale is greater than 10, otherwise it uses `make_base_discrete_laplace_linear`.

    [make_base_discrete_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace.html)

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
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_base_discrete_laplace
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_QO), Measurement))

    return output

def then_base_discrete_laplace(
    scale,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_base_discrete_laplace

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_discrete_laplace`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



def make_base_discrete_laplace_cks20(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
    using an efficient algorithm on rational bignums.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    [make_base_discrete_laplace_cks20 in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_cks20.html)

    **Citations:**

    * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`

    :param input_domain: 
    :type input_domain: Domain
    :param input_metric: 
    :type input_metric: Metric
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_base_discrete_laplace_cks20
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_QO), Measurement))

    return output

def then_base_discrete_laplace_cks20(
    scale,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_base_discrete_laplace_cks20

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_discrete_laplace_cks20`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace_cks20(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



def make_base_discrete_laplace_linear(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    bounds: Optional[Any] = None,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
    using a linear-time algorithm on finite data types.

    This algorithm can be executed in constant time if bounds are passed.
    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    [make_base_discrete_laplace_linear in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_linear.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
    :param bounds: Set bounds on the count to make the algorithm run in constant-time.
    :type bounds: Any
    :param QO: Data type of the scale and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
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
    lib_function = lib.opendp_measurements__make_base_discrete_laplace_linear
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_bounds, c_QO), Measurement))

    return output

def then_base_discrete_laplace_linear(
    scale,
    bounds: Optional[Any] = None,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_base_discrete_laplace_linear

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_discrete_laplace_linear`

    :param scale: Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
    :param bounds: Set bounds on the count to make the algorithm run in constant-time.
    :type bounds: Any
    :param QO: Data type of the scale and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace_linear(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds,
        QO=QO))



def make_base_gaussian(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    k: int = -1074,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<T>"
) -> Measurement:
    r"""Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(T)`       |

    This function takes a noise granularity in terms of 2^k.
    Larger granularities are more computationally efficient, but have a looser privacy map.
    If k is not set, k defaults to the smallest granularity.

    [make_base_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_gaussian.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MO`

    :param input_domain: Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized. Valid values are `AbsoluteDistance<T>` or `L2Distance<T>`.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["T"])
    T = get_atom_or_infer(get_carrier_type(input_domain), scale) # type: ignore
    MO = MO.substitute(T=T) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_base_gaussian
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k, c_MO), Measurement))

    return output

def then_base_gaussian(
    scale,
    k: int = -1074,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<T>"
):  
    r"""partial constructor of make_base_gaussian

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_gaussian`

    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k,
        MO=MO))



def make_base_geometric(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    bounds: Optional[Any] = None,
    QO: Optional[RuntimeTypeDescriptor] = None
) -> Measurement:
    r"""An alias for `make_base_discrete_laplace_linear`.
    If you don't need timing side-channel protections via `bounds`,
    `make_base_discrete_laplace` is more efficient.

    [make_base_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_geometric.html)

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
    :type bounds: Any
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
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
    lib_function = lib.opendp_measurements__make_base_geometric
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_bounds, c_QO), Measurement))

    return output

def then_base_geometric(
    scale,
    bounds: Optional[Any] = None,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_base_geometric

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_geometric`

    :param scale: 
    :param bounds: 
    :type bounds: Any
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_geometric(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds,
        QO=QO))



def make_base_laplace(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    k: int = -1074
) -> Measurement:
    r"""Make a Measurement that adds noise from the Laplace(`scale`) distribution to a scalar value.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    This function takes a noise granularity in terms of 2^k.
    Larger granularities are more computationally efficient, but have a looser privacy map.
    If k is not set, k defaults to the smallest granularity.

    [make_base_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_laplace.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<D::Atom>`

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = get_atom_or_infer(get_carrier_type(input_domain), scale) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    c_k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)

    # Call library function.
    lib_function = lib.opendp_measurements__make_base_laplace
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_k), Measurement))

    return output

def then_base_laplace(
    scale,
    k: int = -1074
):  
    r"""partial constructor of make_base_laplace

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_laplace`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k))



def make_base_laplace_threshold(
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

    [make_base_laplace_threshold in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_laplace_threshold.html)

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
    lib_function = lib.opendp_measurements__make_base_laplace_threshold
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_threshold, c_k), Measurement))

    return output

def then_base_laplace_threshold(
    scale,
    threshold,
    k: int = -1074
):  
    r"""partial constructor of make_base_laplace_threshold

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_base_laplace_threshold`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param threshold: Exclude counts that are less than this minimum value.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    """
    return PartialConstructor(lambda input_domain, input_metric: make_base_laplace_threshold(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        threshold=threshold,
        k=k))



def make_gaussian(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
) -> Measurement:
    r"""Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.

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
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["QO"])
    QO = get_atom_or_infer(MO, scale) # type: ignore
    MO = MO.substitute(QO=QO) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=get_atom(MO))
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_gaussian
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_MO), Measurement))

    return output

def then_gaussian(
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
):  
    r"""partial constructor of make_gaussian

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_gaussian`

    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        MO=MO))



def make_laplace(
    input_domain: Domain,
    input_metric: Metric,
    scale,
    QO: RuntimeTypeDescriptor = "float"
) -> Measurement:
    r"""Make a Measurement that adds noise from the laplace(`scale`) distribution to the input.

    Valid inputs for `input_domain` and `input_metric` are:

    | `input_domain`                  | input type   | `input_metric`         |
    | ------------------------------- | ------------ | ---------------------- |
    | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
    | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |

    This uses `make_base_laplace` if `T` is float, otherwise it uses `make_base_discrete_laplace`.

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
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    QO = RuntimeType.parse(type_name=QO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=get_atom(QO))
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_laplace
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_QO), Measurement))

    return output

def then_laplace(
    scale,
    QO: RuntimeTypeDescriptor = "float"
):  
    r"""partial constructor of make_laplace

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_laplace`

    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    """
    return PartialConstructor(lambda input_domain, input_metric: make_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



def make_randomized_response(
    categories: Any,
    prob,
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
    :type categories: Any
    :param prob: Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
    :param T: Data type of a category.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of probability and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(categories))
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=prob)

    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[T]))
    c_prob = py_to_c(prob, c_type=ctypes.c_void_p, type_name=QO)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_measurements__make_randomized_response
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_categories, c_prob, c_T, c_QO), Measurement))

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
    scale: Any,
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
    :type scale: Any
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
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
    scale: Any,
    optimize: str,
    QO: Optional[RuntimeTypeDescriptor] = None
):  
    r"""partial constructor of make_report_noisy_max_gumbel

    .. seealso:: 
      Delays application of `input_domain` and `input_metric` in :py:func:`opendp.measurements.make_report_noisy_max_gumbel`

    :param scale: Higher scales are more private.
    :type scale: Any
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
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

