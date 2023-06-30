# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "make_base_discrete_exponential",
    "make_base_discrete_gaussian",
    "make_base_discrete_laplace",
    "make_base_discrete_laplace_cks20",
    "make_base_discrete_laplace_linear",
    "make_base_gaussian",
    "make_base_geometric",
    "make_base_laplace",
    "make_base_laplace_threshold",
    "make_discrete_exponential_expr",
    "make_gaussian",
    "make_laplace",
    "make_private_agg",
    "make_private_mean_expr",
    "make_private_quantile",
    "make_randomized_response",
    "make_randomized_response_bool",
    "make_user_measurement",
    "then_base_discrete_exponential",
    "then_base_discrete_gaussian",
    "then_base_discrete_laplace",
    "then_base_discrete_laplace_cks20",
    "then_base_discrete_laplace_linear",
    "then_base_gaussian",
    "then_base_geometric",
    "then_base_laplace",
    "then_base_laplace_threshold",
    "then_discrete_exponential_expr",
    "then_gaussian",
    "then_laplace",
    "then_private_agg",
    "then_private_mean_expr",
    "then_private_quantile",
    "then_user_measurement"
]


@versioned
def make_base_discrete_exponential(
    input_domain,
    input_metric,
    temperature: Any,
    optimize: str,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
    
    [make_base_discrete_exponential in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_exponential.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Type:    `usize`
    * Input Metric:   `LInfDiffDistance<TIA>`
    * Output Measure: `MaxDivergence<QO>`
    
    **Proof Definition:**
    
    [(Proof Document)](https://docs.opendp.org/en/latest/proofs/rust/src/measurements/discrete_exponential/make_base_discrete_exponential.pdf)
    
    :param input_domain: Domain of the input vector. Must be a non-nullable VectorDomain.
    :param input_metric: Metric on the input domain. Must be LInfDiffDistance
    :param temperature: Higher temperatures are more private.
    :type temperature: Any
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "floating-point")
    
    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=temperature)
    
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_temperature = py_to_c(temperature, c_type=AnyObjectPtr, type_name=QO)
    c_optimize = py_to_c(optimize, c_type=ctypes.c_char_p, type_name=String)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measurements__make_base_discrete_exponential
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_temperature, c_optimize, c_QO), Measurement))
    
    return output

def then_base_discrete_exponential(
    temperature: Any,
    optimize: str,
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_exponential(
        input_domain=input_domain,
        input_metric=input_metric,
        temperature=temperature,
        optimize=optimize,
        QO=QO))



@versioned
def make_base_discrete_gaussian(
    input_domain,
    input_metric,
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
    
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param MO: Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["QO"])
    QO = get_atom_or_infer(MO, scale)
    MO = MO.substitute(QO=QO)
    
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
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        MO=MO))



@versioned
def make_base_discrete_laplace(
    input_domain,
    input_metric,
    scale,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.
    
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



@versioned
def make_base_discrete_laplace_cks20(
    input_domain,
    input_metric,
    scale,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
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
    :param input_metric: 
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace_cks20(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



@versioned
def make_base_discrete_laplace_linear(
    input_domain,
    input_metric,
    scale,
    bounds: Any = None,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
    :param bounds: Set bounds on the count to make the algorithm run in constant-time.
    :type bounds: Any
    :param QO: Data type of the scale and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_atom(get_carrier_type(input_domain))
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])])
    
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
    bounds: Any = None,
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_base_discrete_laplace_linear(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds,
        QO=QO))



@versioned
def make_base_gaussian(
    input_domain,
    input_metric,
    scale,
    k: int = -1074,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<T>"
) -> Measurement:
    """Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
    
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
    :param input_metric: Metric of the data type to be privatized. Valid values are `AbsoluteDistance<T>` or `L2Distance<T>`.
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["T"])
    T = get_atom_or_infer(get_carrier_type(input_domain), scale)
    MO = MO.substitute(T=T)
    
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
    return PartialConstructor(lambda input_domain, input_metric: make_base_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k,
        MO=MO))



@versioned
def make_base_geometric(
    input_domain,
    input_metric,
    scale,
    bounds: Any = None,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """An alias for `make_base_discrete_laplace_linear`.
    If you don't need timing side-channel protections via `bounds`,
    `make_base_discrete_laplace` is more efficient.
    
    [make_base_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_geometric.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Type:    `D::Carrier`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`
    
    :param input_domain: 
    :param input_metric: 
    :param scale: 
    :param bounds: 
    :type bounds: Any
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_atom(get_carrier_type(input_domain))
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])])
    
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
    bounds: Any = None,
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_base_geometric(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        bounds=bounds,
        QO=QO))



@versioned
def make_base_laplace(
    input_domain,
    input_metric,
    scale,
    k: int = -1074
) -> Measurement:
    """Make a Measurement that adds noise from the Laplace(`scale`) distribution to a scalar value.
    
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = get_atom_or_infer(get_carrier_type(input_domain), scale)
    
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
    return PartialConstructor(lambda input_domain, input_metric: make_base_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        k=k))



@versioned
def make_base_laplace_threshold(
    input_domain,
    input_metric,
    scale,
    threshold,
    k: int = -1074
) -> Measurement:
    """Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
    
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
    :param input_metric: Metric for the input domain.
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param threshold: Exclude counts that are less than this minimum value.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "floating-point")
    
    # Standardize type arguments.
    TV = get_distance_type(input_metric)
    
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
    return PartialConstructor(lambda input_domain, input_metric: make_base_laplace_threshold(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        threshold=threshold,
        k=k))



@versioned
def make_discrete_exponential_expr(
    input_domain,
    input_metric,
    temperature: Any,
    optimize: str,
    MI: RuntimeTypeDescriptor,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Makes a Measurement to implement the discrete exponential mechanism with Polars.
    Takes a series of scores and privately selects the index of the highest score.
    
    [make_discrete_exponential_expr in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_discrete_exponential_expr.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `ExprDomain<MI::LazyDomain>`
    * Output Type:    `Expr`
    * Input Metric:   `MI`
    * Output Measure: `MaxDivergence<QO>`
    
    :param input_domain: ExprDomain
    :param input_metric: The metric space under which neighboring LazyFrames are compared
    :param temperature: Higher temperatures are more private.
    :type temperature: Any
    :param optimize: Indicate whether to privately return the "Max" or "Min"
    :type optimize: str
    :param MI: Input Metric.
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=temperature)
    
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_temperature = py_to_c(temperature, c_type=AnyObjectPtr, type_name=QO)
    c_optimize = py_to_c(optimize, c_type=ctypes.c_char_p, type_name=Optimize)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measurements__make_discrete_exponential_expr
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_temperature, c_optimize, c_MI, c_QO), Measurement))
    
    return output

def then_discrete_exponential_expr(
    temperature: Any,
    optimize: str,
    MI: RuntimeTypeDescriptor,
    QO: RuntimeTypeDescriptor = None
):
    return PartialConstructor(lambda input_domain, input_metric: make_discrete_exponential_expr(
        input_domain=input_domain,
        input_metric=input_metric,
        temperature=temperature,
        optimize=optimize,
        MI=MI,
        QO=QO))



@versioned
def make_gaussian(
    input_domain,
    input_metric,
    scale,
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>"
) -> Measurement:
    """Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
    
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["QO"])
    QO = get_atom_or_infer(MO, scale)
    MO = MO.substitute(QO=QO)
    
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
    return PartialConstructor(lambda input_domain, input_metric: make_gaussian(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        MO=MO))



@versioned
def make_laplace(
    input_domain,
    input_metric,
    scale,
    QO: RuntimeTypeDescriptor = "float"
) -> Measurement:
    """Make a Measurement that adds noise from the laplace(`scale`) distribution to the input.
    
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
    :param input_metric: Metric of the data type to be privatized.
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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
    return PartialConstructor(lambda input_domain, input_metric: make_laplace(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



@versioned
def make_private_agg(
    input_domain,
    input_metric,
    measurement: Measurement
) -> Measurement:
    """Make a Transformation that returns a Measurement in agg context.
    
    Valid inputs for `input_domain` and `input_metric` are:
    
    | `input_domain`                  | `input_metric`                             |
    | ------------------------------- | ------------------------------------------ |
    | `LazyGroupByDomain`             | `SymmetricDistance`                        |
    | `LazyGroupByDomain`             | `InsertDeleteDistance`                     |
    | `LazyGroupByDomain`             | `ChangeOneDistance` if Margins provided    |
    | `LazyGroupByDomain`             | `HammingDistance` if Margins provided      |
    | `LazyGroupByDomain`             | `Lp<p, AbsoluteDistance>`                  |
    
    [make_private_agg in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_private_agg.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `LazyGroupByDomain`
    * Output Type:    `LazyFrame`
    * Input Metric:   `T::InputMetric`
    * Output Measure: `T::OutputMeasure`
    
    :param input_domain: LazyGroupByDomain.
    :param input_metric: The metric space under which neighboring LazyGroupBy frames are compared.
    :param measurement: 
    :type measurement: Measurement
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_measurement = py_to_c(measurement, c_type=Measurement, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measurements__make_private_agg
    lib_function.argtypes = [Domain, Metric, Measurement]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_measurement), Measurement))
    
    return output

def then_private_agg(
    measurement: Measurement
):
    return PartialConstructor(lambda input_domain, input_metric: make_private_agg(
        input_domain=input_domain,
        input_metric=input_metric,
        measurement=measurement))



@versioned
def make_private_mean_expr(
    input_domain,
    input_metric,
    scale,
    QO: RuntimeTypeDescriptor = "float"
) -> Measurement:
    """Polars operator to compute the private mean of a column in a LazyFrame or LazyGroupBy
    
    [make_private_mean_expr in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_private_mean_expr.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `ExprDomain<MI::LazyDomain>`
    * Output Type:    `Expr`
    * Input Metric:   `MI`
    * Output Measure: `MaxDivergence<QO>`
    
    :param input_domain: ExprDomain
    :param input_metric: The metric under which neighboring LazyFrames are compared
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param QO: Output data type of the scale and epsilon
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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
    lib_function = lib.opendp_measurements__make_private_mean_expr
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_scale, c_QO), Measurement))
    
    return output

def then_private_mean_expr(
    scale,
    QO: RuntimeTypeDescriptor = "float"
):
    return PartialConstructor(lambda input_domain, input_metric: make_private_mean_expr(
        input_domain=input_domain,
        input_metric=input_metric,
        scale=scale,
        QO=QO))



@versioned
def make_private_quantile(
    input_domain,
    input_metric,
    candidates: Any,
    temperature,
    alpha: Any,
    QO: RuntimeTypeDescriptor = "float",
    A: RuntimeTypeDescriptor = "float"
) -> Measurement:
    """Makes a Measurement to compute DP quantiles with Polars.
    Based on a list of candidates, first compute scores, then use exponential mechanism
    to get the maximum index of the quantile.
    
    [make_private_quantile in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_private_quantile.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `ExprDomain<MI::LazyDomain>`
    * Output Type:    `Expr`
    * Input Metric:   `MI`
    * Output Measure: `MaxDivergence<QO>`
    
    :param input_domain: ExprDomain
    :param input_metric: The metric space under which neighboring LazyFrames are compared
    :param candidates: Potential quantile to score
    :type candidates: Any
    :param temperature: Higher temperatures are more private.
    :param alpha: Quantile value in [0, 1]. Choose 0.5 for median
    :type alpha: Any
    :param QO: Output Distance Type.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :param A: Alpha type. Can be a (numer, denom) tuple, or float.
    :type A: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=temperature)
    A = RuntimeType.parse_or_infer(type_name=A, public_example=alpha)
    TIA = get_active_column_type(input_domain)
    
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_candidates = py_to_c(candidates, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_temperature = py_to_c(temperature, c_type=ctypes.c_void_p, type_name=QO)
    c_alpha = py_to_c(alpha, c_type=AnyObjectPtr, type_name=A)
    c_QO = py_to_c(QO, c_type=ctypes.c_char_p)
    c_A = py_to_c(A, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measurements__make_private_quantile
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_candidates, c_temperature, c_alpha, c_QO, c_A), Measurement))
    
    return output

def then_private_quantile(
    candidates: Any,
    temperature,
    alpha: Any,
    QO: RuntimeTypeDescriptor = "float",
    A: RuntimeTypeDescriptor = "float"
):
    return PartialConstructor(lambda input_domain, input_metric: make_private_quantile(
        input_domain=input_domain,
        input_metric=input_metric,
        candidates=candidates,
        temperature=temperature,
        alpha=alpha,
        QO=QO,
        A=A))



@versioned
def make_randomized_response(
    categories: Any,
    prob,
    constant_time: bool = False,
    T: RuntimeTypeDescriptor = None,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that implements randomized response on a categorical value.
    
    [make_randomized_response in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AtomDomain<T>`
    * Output Type:    `T`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<QO>`
    
    :param categories: Set of valid outcomes
    :type categories: Any
    :param prob: Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param T: Data type of a category.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of probability and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
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


@versioned
def make_randomized_response_bool(
    prob,
    constant_time: bool = False,
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that implements randomized response on a boolean value.
    
    [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response_bool.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AtomDomain<bool>`
    * Output Type:    `bool`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<QO>`
    
    :param prob: Probability of returning the correct answer. Must be in `[0.5, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param QO: Data type of probability and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
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


@versioned
def make_user_measurement(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor
) -> Measurement:
    """Construct a Measurement from user-defined callbacks.
    
    [make_user_measurement in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_user_measurement.html)
    
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
    :raises UnknownTypeError: if a type argument fails to parse
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
    output._depends_on(c_function, c_privacy_map)
    return output

def then_user_measurement(
    output_measure: Measure,
    function,
    privacy_map,
    TO: RuntimeTypeDescriptor
):
    return PartialConstructor(lambda input_domain, input_metric: make_user_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=function,
        privacy_map=privacy_map,
        TO=TO))

