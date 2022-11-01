# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "choose_branching_factor",
    "make_b_ary_tree",
    "make_bounded_float_checked_sum",
    "make_bounded_float_ordered_sum",
    "make_bounded_int_monotonic_sum",
    "make_bounded_int_ordered_sum",
    "make_bounded_int_split_sum",
    "make_bounded_sum",
    "make_cast",
    "make_cast_default",
    "make_cast_inherent",
    "make_cdf",
    "make_clamp",
    "make_consistent_b_ary_tree",
    "make_count",
    "make_count_by",
    "make_count_by_categories",
    "make_count_distinct",
    "make_create_dataframe",
    "make_df_cast_default",
    "make_df_is_equal",
    "make_drop_null",
    "make_find",
    "make_find_bin",
    "make_identity",
    "make_impute_constant",
    "make_impute_uniform_float",
    "make_index",
    "make_is_equal",
    "make_is_null",
    "make_lipschitz_float_mul",
    "make_metric_bounded",
    "make_metric_unbounded",
    "make_ordered_random",
    "make_quantiles_from_counts",
    "make_resize",
    "make_select_column",
    "make_sized_bounded_float_checked_sum",
    "make_sized_bounded_float_ordered_sum",
    "make_sized_bounded_int_checked_sum",
    "make_sized_bounded_int_monotonic_sum",
    "make_sized_bounded_int_ordered_sum",
    "make_sized_bounded_int_split_sum",
    "make_sized_bounded_mean",
    "make_sized_bounded_sum",
    "make_sized_bounded_sum_of_squared_deviations",
    "make_sized_bounded_variance",
    "make_split_dataframe",
    "make_split_lines",
    "make_split_records",
    "make_subset_by",
    "make_unordered"
]


def choose_branching_factor(
    size_guess: int
) -> int:
    """Returns an approximation to the ideal `branching_factor` for a dataset of a given size,
    that minimizes error in cdf and quantile estimates based on b-ary trees.
    
    [choose_branching_factor in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.choose_branching_factor.html)
    
    **Citations:**
    
    * [QYL13 Understanding Hierarchical Methods for Differentially Private Histograms](http://www.vldb.org/pvldb/vol6/p1954-qardaji.pdf)
    
    :param size_guess: A guess at the size of your dataset.
    :type size_guess: int
    :rtype: int
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_size_guess = py_to_c(size_guess, c_type=ctypes.c_size_t, type_name=usize)
    
    # Call library function.
    lib_function = lib.opendp_transformations__choose_branching_factor
    lib_function.argtypes = [ctypes.c_size_t]
    lib_function.restype = ctypes.c_size_t
    
    output = c_to_py(lib_function(c_size_guess))
    
    return output


def make_b_ary_tree(
    leaf_count: int,
    branching_factor: int,
    M: RuntimeTypeDescriptor,
    TA: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Expand a vector of counts into a b-ary tree of counts,
    where each branch is the sum of its `b` immediate children.
    
    [make_b_ary_tree in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_b_ary_tree.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TA>>`
    * Input Metric:   `M`
    * Output Metric:  `M`
    
    :param leaf_count: The number of leaf nodes in the b-ary tree.
    :type leaf_count: int
    :param branching_factor: The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
    :type branching_factor: int
    :param M: Metric. Must be `L1Distance<Q>` or `L2Distance<Q>`
    :type M: :py:ref:`RuntimeTypeDescriptor`
    :param TA: Atomic Type of the input data.
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    M = RuntimeType.parse(type_name=M)
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    c_leaf_count = py_to_c(leaf_count, c_type=ctypes.c_size_t, type_name=usize)
    c_branching_factor = py_to_c(branching_factor, c_type=ctypes.c_size_t, type_name=usize)
    c_M = py_to_c(M, c_type=ctypes.c_char_p)
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_b_ary_tree
    lib_function.argtypes = [ctypes.c_size_t, ctypes.c_size_t, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_leaf_count, c_branching_factor, c_M, c_TA), Transformation))
    
    return output


def make_bounded_float_checked_sum(
    size_limit: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data with known dataset size.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.
    
    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |
    
    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.
    
    For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.
    
    [make_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_float_checked_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size_limit: Upper bound on number of records to keep in the input data.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param S: Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size_limit = py_to_c(size_limit, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_float_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size_limit, c_bounds, c_S), Transformation))
    
    return output


def make_bounded_float_ordered_sum(
    size_limit: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with known ordering.
    
    Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
    You may need to use `make_ordered_random` to impose an ordering on the data.
    The utility loss from overestimating the `size_limit` is small.
    
    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |
    
    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.
    
    For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.
    
    [make_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_float_ordered_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `InsertDeleteDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size_limit: Upper bound on the number of records in input data. Used to bound sensitivity.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param S: Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size_limit = py_to_c(size_limit, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_float_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size_limit, c_bounds, c_S), Transformation))
    
    return output


def make_bounded_int_monotonic_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints,
    where all values share the same sign.
    
    [make_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_int_monotonic_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_int_monotonic_sum
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_T), Transformation))
    
    return output


def make_bounded_int_ordered_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints.
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    [make_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_int_ordered_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `InsertDeleteDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_int_ordered_sum
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_T), Transformation))
    
    return output


def make_bounded_int_split_sum(
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints.
    Adds the saturating sum of the positives to the saturating sum of the negatives.
    
    [make_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_int_split_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_int_split_sum
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_T), Transformation))
    
    return output


def make_bounded_sum(
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data.
    Use `make_clamp` to bound data.
    
    [make_bounded_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_bounded_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
<<<<<<< HEAD
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
=======
    * Input Domain:   `VectorDomain<BoundedDomain<T>>`
    * Output Domain:  `AllDomain<T>`
>>>>>>> initial compatible pairings implementation
    * Input Metric:   `MI`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :param T: Atomic Input Type and Output Type.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_sum
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_MI, c_T), Transformation))
    
    return output


def make_cast(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
    For each element, failure to parse results in `None`, else `Some(out)`.
    
    Can be chained with `make_impute_constant` or `make_drop_null` to handle nullity.
    
    [make_cast in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_cast.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<OptionDomain<AtomDomain<TOA>>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_cast
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TIA, c_TOA), Transformation))
    
    return output


def make_cast_default(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
    Any element that fails to cast is filled with default.
    
    
    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | `0.`             |
    | int    | `0`              |
    | string | `""`             |
    | bool   | `false`          |
    
    [make_cast_default in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_cast_default.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_cast_default
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TIA, c_TOA), Transformation))
    
    return output


def make_cast_inherent(
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation that casts a vector of data from type `TIA` to a type that can represent nullity `TOA`.
    If cast fails, fill with `TOA`'s null value.
    
    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | NaN              |
    
    [make_cast_inherent in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_cast_inherent.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_cast_inherent
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TIA, c_TOA), Transformation))
    
    return output


def make_cdf(
    TA: RuntimeTypeDescriptor = "float"
) -> Function:
    """Postprocess a noisy array of float summary counts into a cumulative distribution.
    
    [make_cdf in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_cdf.html)
    
    **Supporting Elements:**
    
    * Input Type:     `Vec<TA>`
    * Output Type:    `Vec<TA>`
    
    :param TA: Atomic Type. One of `f32` or `f64`
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse(type_name=TA)
    
    # Convert arguments to c types.
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_cdf
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TA), Function))
    
    return output


def make_clamp(
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.
    
    If datum is less than lower, let datum be lower.
    If datum is greater than upper, let datum be upper.
    
    [make_clamp in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_clamp.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param TA: Atomic Type
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_clamp
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_TA), Transformation))
    
    return output


def make_consistent_b_ary_tree(
    branching_factor: int,
    TIA: RuntimeTypeDescriptor = "int",
    TOA: RuntimeTypeDescriptor = "float"
) -> Function:
    """Postprocessor that makes a noisy b-ary tree internally consistent, and returns the leaf layer.
    
    The input argument of the function is a balanced `b`-ary tree implicitly stored in breadth-first order
    Tree is assumed to be complete, as in, all leaves on the last layer are on the left.
    Non-existent leaves are assumed to be zero.
    
    The output remains consistent even when leaf nodes are missing.
    This is due to an adjustment to the original algorithm to apportion corrections to children relative to their variance.
    
    [make_consistent_b_ary_tree in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_consistent_b_ary_tree.html)
    
    **Citations:**
    
    * [HRMS09 Boosting the Accuracy of Differentially Private Histograms Through Consistency, section 4.1](https://arxiv.org/pdf/0904.0942.pdf)
    
    **Supporting Elements:**
    
    * Input Type:     `Vec<TIA>`
    * Output Type:    `Vec<TOA>`
    
    :param branching_factor: the maximum number of children
    :type branching_factor: int
    :param TIA: Atomic type of the input data. Should be an integer type.
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic type of the output data. Should be a float type.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_branching_factor = py_to_c(branching_factor, c_type=ctypes.c_size_t, type_name=usize)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_consistent_b_ary_tree
    lib_function.argtypes = [ctypes.c_size_t, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_branching_factor, c_TIA, c_TOA), Function))
    
    return output


def make_count(
    TIA: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes a count of the number of records in data.
    
    [make_count in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_count.html)
    
    **Citations:**
    
    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `AtomDomain<TO>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<TO>`
    
    **Proof Definition:**
    
    [(Proof Document)](https://docs.opendp.org/en/latest/proofs/rust/src/transformations/count/make_count.pdf)
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form `Vec<TIA>`.
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_count
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TIA, c_TO), Transformation))
    
    return output


def make_count_by(
    MO: SensitivityMetric,
    TK: RuntimeTypeDescriptor,
    TV: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes the count of each unique value in data.
    This assumes that the category set is unknown.
    
    [make_count_by in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_count_by.html)
    
    **Citations:**
    
    * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TK>>`
    * Output Domain:  `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `MO`
    
    :param MO: Output Metric.
    :type MO: SensitivityMetric
    :param TK: Type of Key. Categorical/hashable input data type. Input data must be `Vec<TK>`.
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :param TV: Type of Value. Express counts in terms of this integral type.
    :type TV: :py:ref:`RuntimeTypeDescriptor`
    :return: The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TK = RuntimeType.parse(type_name=TK)
    TV = RuntimeType.parse(type_name=TV)
    
    # Convert arguments to c types.
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)
    c_TK = py_to_c(TK, c_type=ctypes.c_char_p)
    c_TV = py_to_c(TV, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_count_by
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_MO, c_TK, c_TV), Transformation))
    
    return output


def make_count_by_categories(
    categories: Any,
    null_category: bool = True,
    MO: SensitivityMetric = "L1Distance<int>",
    TIA: RuntimeTypeDescriptor = None,
    TOA: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes the number of times each category appears in the data.
    This assumes that the category set is known.
    
    [make_count_by_categories in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_count_by_categories.html)
    
    **Citations:**
    
    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `MO`
    
    :param categories: The set of categories to compute counts for.
    :type categories: Any
    :param null_category: Include a count of the number of elements that were not in the category set at the end of the vector.
    :type null_category: bool
    :param MO: Output Metric.
    :type MO: SensitivityMetric
    :param TIA: Atomic Input Type that is categorical/hashable. Input data must be `Vec<TIA>`
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type that is numeric.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :return: The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=get_first(categories))
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_null_category = py_to_c(null_category, c_type=ctypes.c_bool, type_name=bool)
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_count_by_categories
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_categories, c_null_category, c_MO, c_TIA, c_TOA), Transformation))
    
    return output


def make_count_distinct(
    TIA: RuntimeTypeDescriptor,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    """Make a Transformation that computes a count of the number of unique, distinct records in data.
    
    [make_count_distinct in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_count_distinct.html)
    
    **Citations:**
    
    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `AtomDomain<TO>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<TO>`
    
    :param TIA: Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA)
    TO = RuntimeType.parse(type_name=TO)
    
    # Convert arguments to c types.
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_count_distinct
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_TIA, c_TO), Transformation))
    
    return output


def make_create_dataframe(
    col_names: Any,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that constructs a dataframe from a `Vec<Vec<String>>` (a vector of records).
    
    [make_create_dataframe in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_create_dataframe.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<VectorDomain<AtomDomain<String>>>`
    * Output Domain:  `DataFrameDomain<K>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param col_names: Column names for each record entry.
    :type col_names: Any
    :param K: categorical/hashable data type of column names
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=get_first(col_names))
    
    # Convert arguments to c types.
    c_col_names = py_to_c(col_names, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[K]))
    c_K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_create_dataframe
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_col_names, c_K), Transformation))
    
    return output


def make_df_cast_default(
    column_name: Any,
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor,
    TK: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that casts the elements in a column in a dataframe from type `TIA` to type `TOA`.
    If cast fails, fill with default.
    
    
    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | `0.`             |
    | int    | `0`              |
    | string | `""`             |
    | bool   | `false`          |
    
    [make_df_cast_default in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_df_cast_default.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `DataFrameDomain<TK>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param column_name: column name to be transformed
    :type column_name: Any
    :param TK: Type of the column name
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TK = RuntimeType.parse_or_infer(type_name=TK, public_example=column_name)
    TIA = RuntimeType.parse(type_name=TIA)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_column_name = py_to_c(column_name, c_type=AnyObjectPtr, type_name=TK)
    c_TK = py_to_c(TK, c_type=ctypes.c_char_p)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_df_cast_default
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_column_name, c_TK, c_TIA, c_TOA), Transformation))
    
    return output


def make_df_is_equal(
    column_name: Any,
    value: Any,
    TK: RuntimeTypeDescriptor = None,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.
    
    [make_df_is_equal in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_df_is_equal.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `DataFrameDomain<TK>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param column_name: Column name to be transformed
    :type column_name: Any
    :param value: Value to check for equality
    :type value: Any
    :param TK: Type of the column name
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TK = RuntimeType.parse_or_infer(type_name=TK, public_example=column_name)
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=value)
    
    # Convert arguments to c types.
    c_column_name = py_to_c(column_name, c_type=AnyObjectPtr, type_name=TK)
    c_value = py_to_c(value, c_type=AnyObjectPtr, type_name=TIA)
    c_TK = py_to_c(TK, c_type=ctypes.c_char_p)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_df_is_equal
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_column_name, c_value, c_TK, c_TIA), Transformation))
    
    return output


def make_drop_null(
    atom_domain,
    DA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that drops null values.
    
    
    | `DA`                                | `DA::Imputed` |
    | ----------------------------------- | ------------- |
    | `OptionDomain<AtomDomain<TA>>`      | `TA`          |
    | `AtomDomain<TA>`                    | `TA`          |
    
    [make_drop_null in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_drop_null.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<DA>`
    * Output Domain:  `VectorDomain<AtomDomain<DA::Imputed>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param atom_domain: 
    :param DA: atomic domain of input data that contains nulls.
    :type DA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DA = RuntimeType.parse_or_infer(type_name=DA, public_example=atom_domain)
    
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=DA)
    c_DA = py_to_c(DA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_drop_null
    lib_function.argtypes = [Domain, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_atom_domain, c_DA), Transformation))
    
    return output


def make_find(
    categories: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Find the index of a data value in a set of categories.
    
    For each value in the input vector, finds the index of the value in `categories`.
    If an index is found, returns `Some(index)`, else `None`.
    Chain with `make_impute_constant` or `make_drop_null` to handle nullity.
    
    [make_find in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_find.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<OptionDomain<AtomDomain<usize>>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param categories: The set of categories to find indexes from.
    :type categories: Any
    :param TIA: Atomic Input Type that is categorical/hashable
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=get_first(categories))
    
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_find
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_categories, c_TIA), Transformation))
    
    return output


def make_find_bin(
    edges: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a transformation that finds the bin index in a monotonically increasing vector of edges.
    
    For each value in the input vector, finds the index of the bin the value falls into.
    `edges` splits the entire range of `TIA` into bins.
    The first bin at index zero ranges from negative infinity to the first edge, non-inclusive.
    The last bin at index `edges.len()` ranges from the last bin, inclusive, to positive infinity.
    
    To be valid, `edges` must be unique and ordered.
    `edges` are left inclusive, right exclusive.
    
    [make_find_bin in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_find_bin.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<AtomDomain<usize>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param edges: The set of edges to split bins by.
    :type edges: Any
    :param TIA: Atomic Input Type that is numeric
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=get_first(edges))
    
    # Convert arguments to c types.
    c_edges = py_to_c(edges, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_find_bin
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_edges, c_TIA), Transformation))
    
    return output


def make_identity(
    D: RuntimeTypeDescriptor,
    M: RuntimeTypeDescriptor
) -> Transformation:
    """Make a Transformation representing the identity function.
    
    [make_identity in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_identity.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `M`
    * Output Metric:  `M`
    
    :param D: Domain of the identity function. Must be `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param M: Metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AtomDomain
    :type M: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D)
    M = RuntimeType.parse(type_name=M)
    
    # Convert arguments to c types.
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_identity
    lib_function.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_D, c_M), Transformation))
    
    return output


def make_impute_constant(
    atom_domain,
    constant: Any,
    DIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that replaces null/None data with `constant`.
    
    By default, the input type is `Vec<Option<TA>>`, as emitted by make_cast.
    Set `DA` to `AtomDomain<TA>` for imputing on types
    that have an inherent representation of nullity, like floats.
    
    | Atom Input Domain `DIA`             |  Input Type       | `DIA::Imputed` |
    | ----------------------------------- | ----------------- | -------------- |
    | `OptionDomain<AtomDomain<TA>>`      | `Vec<Option<TA>>` | `TA`           |
    | `AtomDomain<TA>`                    | `Vec<TA>`         | `TA`           |
    
    [make_impute_constant in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_impute_constant.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<DIA>`
    * Output Domain:  `VectorDomain<AtomDomain<DIA::Imputed>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param atom_domain: 
    :param constant: Value to replace nulls with.
    :type constant: Any
    :param DIA: Atomic Input Domain of data being imputed.
    :type DIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DIA = RuntimeType.parse_or_infer(type_name=DIA, public_example=atom_domain)
    
    # Convert arguments to c types.
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=DIA)
    c_constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=get_atom(get_carrier_type(atom_domain)))
    c_DIA = py_to_c(DIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_impute_constant
    lib_function.argtypes = [Domain, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_atom_domain, c_constant, c_DIA), Transformation))
    
    return output


def make_impute_uniform_float(
    bounds: Tuple[Any, Any],
    TA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that replaces NaN values in `Vec<TA>` with uniformly distributed floats within `bounds`.
    
    [make_impute_uniform_float in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_impute_uniform_float.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `VectorDomain<AtomDomain<TA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param TA: Atomic Type of data being imputed. One of `f32` or `f64`
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_impute_uniform_float
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bounds, c_TA), Transformation))
    
    return output


def make_index(
    categories: Any,
    null: Any,
    TOA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a transformation that treats each element as an index into a vector of categories.
    
    [make_index in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_index.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<usize>>`
    * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param categories: The set of categories to index into.
    :type categories: Any
    :param null: Category to return if the index is out-of-range of the category set.
    :type null: Any
    :param TOA: Atomic Output Type. Output data will be `Vec<TOA>`.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TOA = RuntimeType.parse_or_infer(type_name=TOA, public_example=get_first(categories))
    
    # Convert arguments to c types.
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TOA]))
    c_null = py_to_c(null, c_type=AnyObjectPtr, type_name=TOA)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_index
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_categories, c_null, c_TOA), Transformation))
    
    return output


def make_is_equal(
    value: Any,
    TIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that checks if each element is equal to `value`.
    
    [make_is_equal in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_is_equal.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `VectorDomain<AtomDomain<bool>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param value: value to check against
    :type value: Any
    :param TIA: Atomic Input Type. Type of elements in the input vector
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=value)
    
    # Convert arguments to c types.
    c_value = py_to_c(value, c_type=AnyObjectPtr, type_name=TIA)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_is_equal
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_value, c_TIA), Transformation))
    
    return output


def make_is_null(
    input_atom_domain,
    DIA: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that checks if each element in a vector is null.
    
    [make_is_null in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_is_null.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<DIA>`
    * Output Domain:  `VectorDomain<AtomDomain<bool>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param input_atom_domain: 
    :param DIA: Atomic Input Domain. Can be any domain for which the carrier type has a notion of nullity.
    :type DIA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DIA = RuntimeType.parse_or_infer(type_name=DIA, public_example=input_atom_domain)
    
    # Convert arguments to c types.
    c_input_atom_domain = py_to_c(input_atom_domain, c_type=Domain, type_name=DIA)
    c_DIA = py_to_c(DIA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_is_null
    lib_function.argtypes = [Domain, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_input_atom_domain, c_DIA), Transformation))
    
    return output


def make_lipschitz_float_mul(
    constant,
    bounds: Tuple[Any, Any],
    D: RuntimeTypeDescriptor = "AtomDomain<T>",
    M: RuntimeTypeDescriptor = "AbsoluteDistance<T>"
) -> Transformation:
    """Make a transformation that multiplies an aggregate by a constant.
    
    The bounds clamp the input, in order to bound the increase in sensitivity from float rounding.
    
    [make_lipschitz_float_mul in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_lipschitz_float_mul.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `M`
    * Output Metric:  `M`
    
    :param constant: The constant to multiply aggregates by.
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param D: Domain of the function. Must be `AtomDomain<T>` or `VectorDomain<AtomDomain<T>>`
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param M: Metric. Must be `AbsoluteDistance<T>`, `L1Distance<T>` or `L2Distance<T>`
    :type M: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse(type_name=D, generics=["T"])
    M = RuntimeType.parse(type_name=M, generics=["T"])
    T = get_atom_or_infer(D, constant)
    D = D.substitute(T=T)
    M = M.substitute(T=T)
    
    # Convert arguments to c types.
    c_constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=T)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_M = py_to_c(M, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_lipschitz_float_mul
    lib_function.argtypes = [ctypes.c_void_p, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_constant, c_bounds, c_D, c_M), Transformation))
    
    return output


def make_metric_bounded(
    domain,
    D: RuntimeTypeDescriptor = None,
    MI: RuntimeTypeDescriptor = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that converts the unbounded dataset metric `MI`
    to the respective bounded dataset metric with a no-op.
    
    The constructor enforces that the input domain has known size,
    because it must have known size to be valid under a bounded dataset metric.
    
    | `MI`                 | `MI::BoundedMetric` |
    | -------------------- | ------------------- |
    | SymmetricDistance    | ChangeOneDistance   |
    | InsertDeleteDistance | HammingDistance     |
    
    [make_metric_bounded in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_metric_bounded.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `MI`
    * Output Metric:  `MI::BoundedMetric`
    
    :param domain: 
    :param D: Domain
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MI: Input Metric
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse_or_infer(type_name=D, public_example=domain)
    MI = RuntimeType.parse(type_name=MI)
    
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=D)
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_metric_bounded
    lib_function.argtypes = [Domain, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_domain, c_D, c_MI), Transformation))
    
    return output


def make_metric_unbounded(
    domain,
    D: RuntimeTypeDescriptor = None,
    MI: RuntimeTypeDescriptor = "ChangeOneDistance"
) -> Transformation:
    """Make a Transformation that converts the bounded dataset metric `MI`
    to the respective unbounded dataset metric with a no-op.
    
    | `MI`              | `MI::UnboundedMetric` |
    | ----------------- | --------------------- |
    | ChangeOneDistance | SymmetricDistance     |
    | HammingDistance   | InsertDeleteDistance  |
    
    [make_metric_unbounded in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_metric_unbounded.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `MI`
    * Output Metric:  `MI::UnboundedMetric`
    
    :param domain: 
    :param D: Domain. The function is a no-op so input and output domains are the same.
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MI: Input Metric.
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse_or_infer(type_name=D, public_example=domain)
    MI = RuntimeType.parse(type_name=MI)
    
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=D)
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_metric_unbounded
    lib_function.argtypes = [Domain, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_domain, c_D, c_MI), Transformation))
    
    return output


def make_ordered_random(
    domain,
    D: RuntimeTypeDescriptor = None,
    MI: RuntimeTypeDescriptor = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that converts the unordered dataset metric `SymmetricDistance`
    to the respective ordered dataset metric `InsertDeleteDistance` by assigning a random permutation.
    
    | `MI`              | `MI::OrderedMetric`  |
    | ----------------- | -------------------- |
    | SymmetricDistance | InsertDeleteDistance |
    | ChangeOneDistance | HammingDistance      |
    
    [make_ordered_random in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_ordered_random.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `MI`
    * Output Metric:  `MI::OrderedMetric`
    
    :param domain: 
    :param D: Domain
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MI: Input Metric
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse_or_infer(type_name=D, public_example=domain)
    MI = RuntimeType.parse(type_name=MI)
    
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=D)
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_ordered_random
    lib_function.argtypes = [Domain, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_domain, c_D, c_MI), Transformation))
    
    return output


def make_quantiles_from_counts(
    bin_edges: Any,
    alphas: Any,
    interpolation: str = "linear",
    TA: RuntimeTypeDescriptor = None,
    F: RuntimeTypeDescriptor = "float"
) -> Function:
    """Postprocess a noisy array of summary counts into quantiles.
    
    [make_quantiles_from_counts in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_quantiles_from_counts.html)
    
    **Supporting Elements:**
    
    * Input Type:     `Vec<TA>`
    * Output Type:    `Vec<TA>`
    
    :param bin_edges: The edges that the input data was binned into before counting.
    :type bin_edges: Any
    :param alphas: Return all specified `alpha`-quantiles.
    :type alphas: Any
    :param interpolation: Must be one of `linear` or `nearest`
    :type interpolation: str
    :param TA: Atomic Type of the bin edges and data.
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :param F: Float type of the alpha argument. One of `f32` or `f64`
    :type F: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Function
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bin_edges))
    F = RuntimeType.parse_or_infer(type_name=F, public_example=get_first(alphas))
    
    # Convert arguments to c types.
    c_bin_edges = py_to_c(bin_edges, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TA]))
    c_alphas = py_to_c(alphas, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[F]))
    c_interpolation = py_to_c(interpolation, c_type=ctypes.c_char_p, type_name=String)
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    c_F = py_to_c(F, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_quantiles_from_counts
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_bin_edges, c_alphas, c_interpolation, c_TA, c_F), Function))
    
    return output


def make_resize(
    size: int,
    atom_domain: Domain,
    constant: Any,
    DA: RuntimeTypeDescriptor = None,
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    MO: RuntimeTypeDescriptor = "SymmetricDistance"
) -> Transformation:
    """Make a Transformation that either truncates or imputes records
    with `constant` to match a provided `size`.
    
    [make_resize in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_resize.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<DA>`
    * Output Domain:  `VectorDomain<DA>`
    * Input Metric:   `MI`
    * Output Metric:  `MO`
    
    :param size: Number of records in output data.
    :type size: int
    :param atom_domain: Domain of elements.
    :type atom_domain: Domain
    :param constant: Value to impute with.
    :type constant: Any
    :param DA: Atomic Domain.
    :type DA: :py:ref:`RuntimeTypeDescriptor`
    :param MI: Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :param MO: Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :return: A vector of the same type `TA`, but with the provided `size`.
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    DA = RuntimeType.parse_or_infer(type_name=DA, public_example=atom_domain)
    MI = RuntimeType.parse(type_name=MI)
    MO = RuntimeType.parse(type_name=MO)
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_atom_domain = py_to_c(atom_domain, c_type=Domain, type_name=DA)
    c_constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=get_atom(DA))
    c_DA = py_to_c(DA, c_type=ctypes.c_char_p)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_resize
    lib_function.argtypes = [ctypes.c_size_t, Domain, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_atom_domain, c_constant, c_DA, c_MI, c_MO), Transformation))
    
    return output


def make_select_column(
    key: Any,
    TOA: RuntimeTypeDescriptor,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that retrieves the column `key` from a dataframe as `Vec<TOA>`.
    
    [make_select_column in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_select_column.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `DataFrameDomain<K>`
    * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param key: categorical/hashable data type of the key/column name
    :type key: Any
    :param K: data type of key
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to downcast vector to
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key)
    TOA = RuntimeType.parse(type_name=TOA)
    
    # Convert arguments to c types.
    c_key = py_to_c(key, c_type=AnyObjectPtr, type_name=K)
    c_K = py_to_c(K, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_select_column
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_key, c_K, c_TOA), Transformation))
    
    return output


def make_sized_bounded_float_checked_sum(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with known dataset size.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    
    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |
    
    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.
    
    For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.
    
    [make_sized_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_float_checked_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param S: Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_float_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_S), Transformation))
    
    return output


def make_sized_bounded_float_ordered_sum(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of bounded floats with known ordering and dataset size.
    
    Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |
    
    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.
    
    For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.
    
    [make_sized_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_float_ordered_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `InsertDeleteDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param S: Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_float_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_S), Transformation))
    
    return output


def make_sized_bounded_int_checked_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints.
    The effective range is reduced, as (bounds * size) must not overflow.
    
    [make_sized_bounded_int_checked_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_int_checked_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    
    return output


def make_sized_bounded_int_monotonic_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints,
    where all values share the same sign.
    
    [make_sized_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_int_monotonic_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_monotonic_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    
    return output


def make_sized_bounded_int_ordered_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints with known dataset size.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    You may need to use `make_ordered_random` to impose an ordering on the data.
    
    [make_sized_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_int_ordered_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `InsertDeleteDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    
    return output


def make_sized_bounded_int_split_sum(
    size: int,
    bounds: Tuple[Any, Any],
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded ints with known dataset size.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    Adds the saturating sum of the positives to the saturating sum of the negatives.
    
    [make_sized_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_int_split_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_split_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    
    return output


def make_sized_bounded_mean(
    size: int,
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the mean of bounded data.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.
    
    [make_sized_bounded_mean in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_mean.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
    * Input Metric:   `MI`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :param T: Atomic Input Type and Output Type.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_mean
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_MI, c_T), Transformation))
    
    return output


def make_sized_bounded_sum(
    size: int,
    bounds: Tuple[Any, Any],
    MI: RuntimeTypeDescriptor = "SymmetricDistance",
    T: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that computes the sum of bounded data with known dataset size.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.
    
    [make_sized_bounded_sum in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_sum.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
<<<<<<< HEAD
    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `AtomDomain<T>`
=======
    * Input Domain:   `SizedDomain<VectorDomain<BoundedDomain<T>>>`
    * Output Domain:  `AllDomain<T>`
>>>>>>> initial compatible pairings implementation
    * Input Metric:   `MI`
    * Output Metric:  `AbsoluteDistance<T>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param MI: Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :param T: Atomic Input Type and Output Type.
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    MI = RuntimeType.parse(type_name=MI)
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_MI, c_T), Transformation))
    
    return output


def make_sized_bounded_sum_of_squared_deviations(
    size: int,
    bounds: Tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the sum of squared deviations of bounded data.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.
    
    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |
    
    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.
    
    For example, to construct a transformation that computes the SSD of `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.
    
    [make_sized_bounded_sum_of_squared_deviations in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_sum_of_squared_deviations.html)
    
    **Citations:**
    
    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param S: Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_sum_of_squared_deviations
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_S), Transformation))
    
    return output


def make_sized_bounded_variance(
    size: int,
    bounds: Tuple[Any, Any],
    ddof: int = 1,
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    """Make a Transformation that computes the variance of bounded data.
    
    This uses a restricted-sensitivity proof that takes advantage of known dataset size.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.
    
    [make_sized_bounded_variance in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_sized_bounded_variance.html)
    
    **Citations:**
    
    * [DHK15 Differential Privacy for Social Science Inference](http://hona.kr/papers/files/DOrazioHonakerKingPrivacy.pdf)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `AtomDomain<S::Item>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `AbsoluteDistance<S::Item>`
    
    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: Tuple[Any, Any]
    :param ddof: Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
    :type ddof: int
    :param S: Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds))
    S = S.substitute(T=T)
    
    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name=usize)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_ddof = py_to_c(ddof, c_type=ctypes.c_size_t, type_name=usize)
    c_S = py_to_c(S, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_variance
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_size_t, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_ddof, c_S), Transformation))
    
    return output


def make_split_dataframe(
    separator: str,
    col_names: Any,
    K: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that splits each record in a String into a `Vec<Vec<String>>`,
    and loads the resulting table into a dataframe keyed by `col_names`.
    
    [make_split_dataframe in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_split_dataframe.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AtomDomain<String>`
    * Output Domain:  `DataFrameDomain<K>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :param col_names: Column names for each record entry.
    :type col_names: Any
    :param K: categorical/hashable data type of column names
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=get_first(col_names))
    
    # Convert arguments to c types.
    c_separator = py_to_c(separator, c_type=ctypes.c_char_p, type_name=None)
    c_col_names = py_to_c(col_names, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[K]))
    c_K = py_to_c(K, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_split_dataframe
    lib_function.argtypes = [ctypes.c_char_p, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_separator, c_col_names, c_K), Transformation))
    
    return output


def make_split_lines(
    
) -> Transformation:
    """Make a Transformation that takes a string and splits it into a `Vec<String>` of its lines.
    
    [make_split_lines in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_split_lines.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `AtomDomain<String>`
    * Output Domain:  `VectorDomain<AtomDomain<String>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_transformations__make_split_lines
    lib_function.argtypes = []
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(), Transformation))
    
    return output


def make_split_records(
    separator: str
) -> Transformation:
    """Make a Transformation that splits each record in a `Vec<String>` into a `Vec<Vec<String>>`.
    
    [make_split_records in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_split_records.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `VectorDomain<AtomDomain<String>>`
    * Output Domain:  `VectorDomain<VectorDomain<AtomDomain<String>>>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_separator = py_to_c(separator, c_type=ctypes.c_char_p, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_split_records
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_separator), Transformation))
    
    return output


def make_subset_by(
    indicator_column: Any,
    keep_columns: Any,
    TK: RuntimeTypeDescriptor = None
) -> Transformation:
    """Make a Transformation that subsets a dataframe by a boolean column.
    
    [make_subset_by in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_subset_by.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `DataFrameDomain<TK>`
    * Input Metric:   `SymmetricDistance`
    * Output Metric:  `SymmetricDistance`
    
    :param indicator_column: name of the boolean column that indicates inclusion in the subset
    :type indicator_column: Any
    :param keep_columns: list of column names to apply subset to
    :type keep_columns: Any
    :param TK: Type of the column name
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    TK = RuntimeType.parse_or_infer(type_name=TK, public_example=indicator_column)
    
    # Convert arguments to c types.
    c_indicator_column = py_to_c(indicator_column, c_type=AnyObjectPtr, type_name=TK)
    c_keep_columns = py_to_c(keep_columns, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TK]))
    c_TK = py_to_c(TK, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_subset_by
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_indicator_column, c_keep_columns, c_TK), Transformation))
    
    return output


def make_unordered(
    domain,
    D: RuntimeTypeDescriptor = None,
    MI: RuntimeTypeDescriptor = "InsertDeleteDistance"
) -> Transformation:
    """Make a Transformation that converts the ordered dataset metric `MI`
    to the respective ordered dataset metric with a no-op.
    
    | `MI`                 | `MI::UnorderedMetric` |
    | -------------------- | --------------------- |
    | InsertDeleteDistance | SymmetricDistance     |
    | HammingDistance      | ChangeOneDistance     |
    
    [make_unordered in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_unordered.html)
    
    **Supporting Elements:**
    
    * Input Domain:   `D`
    * Output Domain:  `D`
    * Input Metric:   `MI`
    * Output Metric:  `MI::UnorderedMetric`
    
    :param domain: 
    :param D: Domain
    :type D: :py:ref:`RuntimeTypeDescriptor`
    :param MI: Input Metric
    :type MI: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Transformation
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeError: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")
    
    # Standardize type arguments.
    D = RuntimeType.parse_or_infer(type_name=D, public_example=domain)
    MI = RuntimeType.parse(type_name=MI)
    
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=D)
    c_D = py_to_c(D, c_type=ctypes.c_char_p)
    c_MI = py_to_c(MI, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_transformations__make_unordered
    lib_function.argtypes = [Domain, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_domain, c_D, c_MI), Transformation))
    
    return output
