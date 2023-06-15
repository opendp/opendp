# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "make_base_discrete_gaussian",
    "make_base_discrete_laplace",
    "make_base_discrete_laplace_cks20",
    "make_base_discrete_laplace_linear",
    "make_base_gaussian",
    "make_base_geometric",
    "make_base_laplace",
    "make_base_ptr",
    "make_gaussian",
    "make_laplace",
    "make_randomized_response",
    "make_randomized_response_bool",
    "then_base_discrete_gaussian",
    "then_base_discrete_laplace",
    "then_base_discrete_laplace_cks20",
    "then_base_discrete_laplace_linear",
    "then_base_gaussian",
    "then_base_geometric",
    "then_base_laplace",
    "then_gaussian",
    "then_laplace"
]


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
def make_base_ptr(
    scale,
    threshold,
    TK: RuntimeTypeDescriptor,
    k: int = -1074,
    TV: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
    
    This function takes a noise granularity in terms of 2^k.
    Larger granularities are more computationally efficient, but have a looser privacy map.
    If k is not set, k defaults to the smallest granularity.
    
    [make_base_ptr in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_ptr.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
    * Output Type:    `HashMap<TK, TV>`
    * Input Metric:   `L1Distance<TV>`
    * Output Measure: `SmoothedMaxDivergence<TV>`
    
    :param scale: Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
    :param threshold: Exclude counts that are less than this minimum value.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param TK: Type of Key. Must be hashable/categorical.
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :param TV: Type of Value. Must be float.
    :type TV: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "floating-point")
    
    # Standardize type arguments.
    TK = RuntimeType.parse(type_name=TK)
    TV = RuntimeType.parse_or_infer(type_name=TV, public_example=scale)
    
    # Convert arguments to c types.
    c_scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=TV)
    c_threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=TV)
    c_k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)
    c_TK = py_to_c(TK, c_type=ctypes.c_char_p)
    c_TV = py_to_c(TV, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measurements__make_base_ptr
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_scale, c_threshold, c_k, c_TK, c_TV), Measurement))
    
    return output


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
