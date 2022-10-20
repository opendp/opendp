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
    "make_randomized_response",
    "make_randomized_response_bool"
]


def make_base_discrete_gaussian(
    scale,
    D: RuntimeTypeDescriptor = "AllDomain<int>",
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<QO>",
    QI: RuntimeTypeDescriptor = "int"
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
    
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`        |
    | ---------------------------- | ------------ | ----------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<QI>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L2Distance<QI>`        |
    
    [make_base_discrete_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_gaussian.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MO`
    
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`.
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MO: Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :param QI: Input distance. The type of sensitivities. Can be any integer or float.
    :type QI: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    MO = RuntimeType.parse(type_name=MO, generics=["QO"])
    QI = RuntimeType.parse(type_name=QI)
    QO = get_atom_or_infer(MO, scale)
    MO = MO.substitute(QO=QO)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    QI = py_to_c(QI, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_discrete_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, D, MO, QI), Measurement))


def make_base_discrete_laplace(
    scale,
    D: RuntimeTypeDescriptor = "AllDomain<int>",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.
    
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`       |
    | ---------------------------- | ------------ | ---------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
    
    This uses `make_base_discrete_laplace_cks20` if scale is greater than 10, otherwise it uses `make_base_discrete_laplace_linear`.
    
    [make_base_discrete_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace.html)
    
    **Citations:**
    
    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`
    
    :param scale: Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of the output distance and scale. `f32` or `f64`.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_discrete_laplace
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, D, QO), Measurement))


def make_base_discrete_laplace_cks20(
    scale,
    D: RuntimeTypeDescriptor = "AllDomain<int>",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input, 
    using an efficient algorithm on rational bignums.
    
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`       |
    | ---------------------------- | ------------ | ---------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
    
    [make_base_discrete_laplace_cks20 in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_cks20.html)
    
    **Citations:**
    
    * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`
    
    :param scale: Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of the output distance and scale.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_discrete_laplace_cks20
    function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, D, QO), Measurement))


def make_base_discrete_laplace_linear(
    scale,
    bounds: Any = None,
    D: RuntimeTypeDescriptor = "AllDomain<int>",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input, 
    using a linear-time algorithm on finite data types.
    
    This algorithm can be executed in constant time if bounds are passed.
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`       |
    | ---------------------------- | ------------ | ---------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
    
    [make_base_discrete_laplace_linear in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_linear.html)
    
    **Citations:**
    
    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`
    
    :param scale: Noise scale parameter for the distribution. `scale` == sqrt(2) * standard_deviation.
    :param bounds: Set bounds on the count to make the algorithm run in constant-time.
    :type bounds: Any
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param QO: Data type of the scale and output distance.
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_atom(D)
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])])
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=OptionT)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_discrete_laplace_linear
    function.argtypes = [ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, bounds, D, QO), Measurement))


def make_base_gaussian(
    scale,
    k: int = -1074,
    D: RuntimeTypeDescriptor = "AllDomain<T>",
    MO: RuntimeTypeDescriptor = "ZeroConcentratedDivergence<T>"
) -> Measurement:
    """Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
    
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`       |
    | ---------------------------- | ------------ | ---------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L2Distance<T>`        |
    
    This function takes a noise granularity in terms of 2^k. 
    Larger granularities are more computationally efficient, but have a looser privacy map. 
    If k is not set, k defaults to the smallest granularity.
    
    [make_base_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_gaussian.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MO`
    
    :param scale: Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`.
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MO: Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    MO = RuntimeType.parse(type_name=MO, generics=["T"])
    T = get_atom_or_infer(D, scale)
    D = D.substitute(T=T)
    MO = MO.substitute(T=T)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_gaussian
    function.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, k, D, MO), Measurement))


def make_base_geometric(
    scale,
    bounds: Any = None,
    D: RuntimeTypeDescriptor = "AllDomain<int>",
    QO: RuntimeTypeDescriptor = None
) -> Measurement:
    """Deprecated. 
    Use `make_base_discrete_laplace` instead (more efficient). 
    `make_base_discrete_laplace_linear` has a similar interface with the optional constant-time bounds.
    
    [make_base_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_geometric.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<QO>`
    
    :param scale: 
    :param bounds: 
    :type bounds: Any
    :param D: 
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param QO: 
    :type QO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    QO = RuntimeType.parse_or_infer(type_name=QO, public_example=scale)
    T = get_atom(D)
    OptionT = RuntimeType(origin='Option', args=[RuntimeType(origin='Tuple', args=[T, T])])
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=QO)
    bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=OptionT)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    QO = py_to_c(QO, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_geometric
    function.argtypes = [ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, bounds, D, QO), Measurement))


def make_base_laplace(
    scale,
    k: int = -1074,
    D: RuntimeTypeDescriptor = "AllDomain<T>"
) -> Measurement:
    """Make a Measurement that adds noise from the laplace(`scale`) distribution to a scalar value.
    
    Set `D` to change the input data type and input metric:
    
    | `D`                          | input type   | `D::InputMetric`       |
    | ---------------------------- | ------------ | ---------------------- |
    | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
    | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
    
    
    This function takes a noise granularity in terms of 2^k. 
    Larger granularities are more computationally efficient, but have a looser privacy map. 
    If k is not set, k defaults to the smallest granularity.
    
    [make_base_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_laplace.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `D::InputMetric`
    * Output Measure: `MaxDivergence<D::Atom>`
    
    :param scale: Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
    :param k: The noise granularity in terms of 2^k.
    :type k: int
    :param D: Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    T = get_atom_or_infer(D, scale)
    D = D.substitute(T=T)
    
    # Convert arguments to c types.
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=T)
    k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)
    D = py_to_c(D, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_laplace
    function.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, k, D), Measurement))


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
    
    * Input Domain:   `MapDomain<AllDomain<TK>, AllDomain<TV>>`
    * Output Domain:  `MapDomain<AllDomain<TK>, AllDomain<TV>>`
    * Input Metric:   `L1Distance<TV>`
    * Output Measure: `SmoothedMaxDivergence<TV>`
    
    :param scale: Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
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
    scale = py_to_c(scale, c_type=ctypes.c_void_p, type_name=TV)
    threshold = py_to_c(threshold, c_type=ctypes.c_void_p, type_name=TV)
    k = py_to_c(k, c_type=ctypes.c_uint32, type_name=i32)
    TK = py_to_c(TK, c_type=ctypes.c_char_p)
    TV = py_to_c(TV, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_base_ptr
    function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(scale, threshold, k, TK, TV), Measurement))


def make_randomized_response(
    categories: Any,
    prob,
    constant_time: bool = False,
    T: RuntimeTypeDescriptor = None,
    Q: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that implements randomized response on a categorical value.
    
    [make_randomized_response in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AllDomain<T>`
    * Output Domain:  `AllDomain<T>`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<Q>`
    
    :param categories: Set of valid outcomes
    :type categories: Any
    :param prob: Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param T: Data type of a category.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :param Q: Data type of probability and output distance.
    :type Q: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(categories))
    Q = RuntimeType.parse_or_infer(type_name=Q, public_example=prob)
    
    # Convert arguments to c types.
    categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[T]))
    prob = py_to_c(prob, c_type=ctypes.c_void_p, type_name=Q)
    constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)
    T = py_to_c(T, c_type=ctypes.c_char_p)
    Q = py_to_c(Q, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_randomized_response
    function.argtypes = [AnyObjectPtr, ctypes.c_void_p, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(categories, prob, constant_time, T, Q), Measurement))


def make_randomized_response_bool(
    prob,
    constant_time: bool = False,
    Q: RuntimeTypeDescriptor = None
) -> Measurement:
    """Make a Measurement that implements randomized response on a boolean value.
    
    [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response_bool.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AllDomain<bool>`
    * Output Domain:  `AllDomain<bool>`
    * Input Metric:   `DiscreteDistance`
    * Output Measure: `MaxDivergence<Q>`
    
    :param prob: Probability of returning the correct answer. Must be in `[0.5, 1)`
    :param constant_time: Set to true to enable constant time. Slower.
    :type constant_time: bool
    :param Q: Data type of probability and output distance.
    :type Q: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measurement
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    Q = RuntimeType.parse_or_infer(type_name=Q, public_example=prob)
    
    # Convert arguments to c types.
    prob = py_to_c(prob, c_type=ctypes.c_void_p, type_name=Q)
    constant_time = py_to_c(constant_time, c_type=ctypes.c_bool, type_name=bool)
    Q = py_to_c(Q, c_type=ctypes.c_char_p)
    
    # Call library function.
    function = lib.opendp_measurements__make_randomized_response_bool
    function.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_char_p]
    function.restype = FfiResult
    
    return c_to_py(unwrap(function(prob, constant_time, Q), Measurement))
