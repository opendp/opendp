# Auto-generated. Do not edit!
'''
The ``transformations`` module provides functions that deterministicly transform datasets.
For more context, see :ref:`transformations in the User Guide <transformations-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.t``.
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401
from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
__all__ = [
    "choose_branching_factor",
    "make_b_ary_tree",
    "make_bounded_float_checked_sum",
    "make_bounded_float_ordered_sum",
    "make_bounded_int_monotonic_sum",
    "make_bounded_int_ordered_sum",
    "make_bounded_int_split_sum",
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
    "make_mean",
    "make_metric_bounded",
    "make_metric_unbounded",
    "make_ordered_random",
    "make_quantile_score_candidates",
    "make_quantiles_from_counts",
    "make_resize",
    "make_select_column",
    "make_sized_bounded_float_checked_sum",
    "make_sized_bounded_float_ordered_sum",
    "make_sized_bounded_int_checked_sum",
    "make_sized_bounded_int_monotonic_sum",
    "make_sized_bounded_int_ordered_sum",
    "make_sized_bounded_int_split_sum",
    "make_split_dataframe",
    "make_split_lines",
    "make_split_records",
    "make_stable_expr",
    "make_stable_lazyframe",
    "make_subset_by",
    "make_sum",
    "make_sum_of_squared_deviations",
    "make_unordered",
    "make_user_transformation",
    "make_variance",
    "then_b_ary_tree",
    "then_cast",
    "then_cast_default",
    "then_cast_inherent",
    "then_clamp",
    "then_count",
    "then_count_by",
    "then_count_by_categories",
    "then_count_distinct",
    "then_df_cast_default",
    "then_df_is_equal",
    "then_drop_null",
    "then_find",
    "then_find_bin",
    "then_identity",
    "then_impute_constant",
    "then_impute_uniform_float",
    "then_index",
    "then_is_equal",
    "then_is_null",
    "then_lipschitz_float_mul",
    "then_mean",
    "then_metric_bounded",
    "then_metric_unbounded",
    "then_ordered_random",
    "then_quantile_score_candidates",
    "then_resize",
    "then_stable_expr",
    "then_stable_lazyframe",
    "then_sum",
    "then_sum_of_squared_deviations",
    "then_unordered",
    "then_variance"
]


def choose_branching_factor(
    size_guess: int
) -> int:
    r"""Returns an approximation to the ideal `branching_factor` for a dataset of a given size,
    that minimizes error in cdf and quantile estimates based on b-ary trees.


    Required features: `contrib`

    [choose_branching_factor in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.choose_branching_factor.html)

    **Citations:**

    * [QYL13 Understanding Hierarchical Methods for Differentially Private Histograms](http://www.vldb.org/pvldb/vol6/p1954-qardaji.pdf)

    .. end-markdown

    :param size_guess: A guess at the size of your dataset.
    :type size_guess: int
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_size_guess = py_to_c(size_guess, c_type=ctypes.c_uint32, type_name="u32")

    # Call library function.
    lib_function = lib.opendp_transformations__choose_branching_factor
    lib_function.argtypes = [ctypes.c_uint32]
    lib_function.restype = ctypes.c_uint32

    output = c_to_py(lib_function(c_size_guess))
    try:
        output.__opendp_dict__ = {
            '__function__': 'choose_branching_factor',
            '__module__': 'transformations',
            '__kwargs__': {
                'size_guess': size_guess
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_b_ary_tree(
    input_domain: Domain,
    input_metric: Metric,
    leaf_count: int,
    branching_factor: int
) -> Transformation:
    r"""Expand a vector of counts into a b-ary tree of counts,
    where each branch is the sum of its `b` immediate children.


    Required features: `contrib`

    [make_b_ary_tree in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_b_ary_tree.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TA>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param leaf_count: The number of leaf nodes in the b-ary tree.
    :type leaf_count: int
    :param branching_factor: The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
    :type branching_factor: int
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_leaf_count = py_to_c(leaf_count, c_type=ctypes.c_uint32, type_name="u32")
    c_branching_factor = py_to_c(branching_factor, c_type=ctypes.c_uint32, type_name="u32")

    # Call library function.
    lib_function = lib.opendp_transformations__make_b_ary_tree
    lib_function.argtypes = [Domain, Metric, ctypes.c_uint32, ctypes.c_uint32]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_leaf_count, c_branching_factor), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_b_ary_tree',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'leaf_count': leaf_count, 'branching_factor': branching_factor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_b_ary_tree(
    leaf_count: int,
    branching_factor: int
):  
    r"""Partial constructor of `make_b_ary_tree`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_b_ary_tree`

    :param leaf_count: The number of leaf nodes in the b-ary tree.
    :type leaf_count: int
    :param branching_factor: The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
    :type branching_factor: int
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_b_ary_tree(
        input_domain=input_domain,
        input_metric=input_metric,
        leaf_count=leaf_count,
        branching_factor=branching_factor))
    output.__opendp_dict__ = {
            '__function__': 'then_b_ary_tree',
            '__module__': 'transformations',
            '__kwargs__': {
                'leaf_count': leaf_count, 'branching_factor': branching_factor
            },
        }
    return output



def make_bounded_float_checked_sum(
    size_limit: int,
    bounds: tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded data with known dataset size.

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


    Required features: `contrib`

    [make_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_bounded_float_checked_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param size_limit: Upper bound on number of records to keep in the input data.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param S: Summation algorithm to use over some data type ``T`` (``T`` is shorthand for ``S::Item``)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_size_limit = py_to_c(size_limit, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_float_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size_limit, c_bounds, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_bounded_float_checked_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size_limit': size_limit, 'bounds': bounds, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_bounded_float_ordered_sum(
    size_limit: int,
    bounds: tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded floats with known ordering.

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


    Required features: `contrib`

    [make_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_bounded_float_ordered_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `InsertDeleteDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param size_limit: Upper bound on the number of records in input data. Used to bound sensitivity.
    :type size_limit: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param S: Summation algorithm to use over some data type ``T`` (``T`` is shorthand for ``S::Item``)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_size_limit = py_to_c(size_limit, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_bounded_float_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size_limit, c_bounds, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_bounded_float_ordered_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size_limit': size_limit, 'bounds': bounds, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_bounded_int_monotonic_sum(
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints,
    where all values share the same sign.


    Required features: `contrib`

    [make_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_bounded_int_monotonic_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_bounded_int_monotonic_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_bounded_int_ordered_sum(
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints.
    You may need to use `make_ordered_random` to impose an ordering on the data.


    Required features: `contrib`

    [make_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_bounded_int_ordered_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `InsertDeleteDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_bounded_int_ordered_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_bounded_int_split_sum(
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints.
    Adds the saturating sum of the positives to the saturating sum of the negatives.


    Required features: `contrib`

    [make_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_bounded_int_split_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_bounded_int_split_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_cast(
    input_domain: Domain,
    input_metric: Metric,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    r"""Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
    For each element, failure to parse results in `None`, else `Some(out)`.

    Can be chained with `make_impute_constant` or `make_drop_null` to handle nullity.


    Required features: `contrib`

    [make_cast in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_cast.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<OptionDomain<AtomDomain<TOA>>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TOA = RuntimeType.parse(type_name=TOA)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_cast
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_cast',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_cast(
    TOA: RuntimeTypeDescriptor
):  
    r"""Partial constructor of `make_cast`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_cast`

    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_cast(
        input_domain=input_domain,
        input_metric=input_metric,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_cast',
            '__module__': 'transformations',
            '__kwargs__': {
                'TOA': TOA
            },
        }
    return output



def make_cast_default(
    input_domain: Domain,
    input_metric: Metric,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    r"""Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
    Any element that fails to cast is filled with default.


    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | `0.`             |
    | int    | `0`              |
    | string | `""`             |
    | bool   | `false`          |


    Required features: `contrib`

    [make_cast_default in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_cast_default.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TOA = RuntimeType.parse(type_name=TOA, generics=["TIA", "M"])
    TIA = get_atom(get_type(input_domain)) # type: ignore
    M = get_type(input_metric) # type: ignore
    TOA = _substitute(TOA, TIA=TIA, M=M) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_cast_default
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_cast_default',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_cast_default(
    TOA: RuntimeTypeDescriptor
):  
    r"""Partial constructor of `make_cast_default`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_cast_default`

    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_cast_default(
        input_domain=input_domain,
        input_metric=input_metric,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_cast_default',
            '__module__': 'transformations',
            '__kwargs__': {
                'TOA': TOA
            },
        }
    return output



def make_cast_inherent(
    input_domain: Domain,
    input_metric: Metric,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    r"""Make a Transformation that casts a vector of data from type `TIA` to a type that can represent nullity `TOA`.
    If cast fails, fill with `TOA`'s null value.

    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | NaN              |


    Required features: `contrib`

    [make_cast_inherent in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_cast_inherent.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TOA = RuntimeType.parse(type_name=TOA)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_cast_inherent
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_cast_inherent',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_cast_inherent(
    TOA: RuntimeTypeDescriptor
):  
    r"""Partial constructor of `make_cast_inherent`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_cast_inherent`

    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_cast_inherent(
        input_domain=input_domain,
        input_metric=input_metric,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_cast_inherent',
            '__module__': 'transformations',
            '__kwargs__': {
                'TOA': TOA
            },
        }
    return output



def make_cdf(
    TA: RuntimeTypeDescriptor = "float"
) -> Function:
    r"""Postprocess a noisy array of float summary counts into a cumulative distribution.


    Required features: `contrib`

    [make_cdf in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_cdf.html)

    **Supporting Elements:**

    * Input Type:     `Vec<TA>`
    * Output Type:    `Vec<TA>`

    .. end-markdown

    :param TA: Atomic Type. One of ``f32`` or ``f64``
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_cdf',
            '__module__': 'transformations',
            '__kwargs__': {
                'TA': TA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_clamp(
    input_domain: Domain,
    input_metric: Metric,
    bounds: tuple[Any, Any]
) -> Transformation:
    r"""Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.

    If datum is less than lower, let datum be lower.
    If datum is greater than upper, let datum be upper.


    Required features: `contrib`

    [make_clamp in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_clamp.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TA>>`
    * Output Metric:  `M`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/clamp/make_clamp.pdf)

    .. end-markdown

    :param input_domain: Domain of input data.
    :type input_domain: Domain
    :param input_metric: Metric on input domain.
    :type input_metric: Metric
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TA = get_atom(get_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))

    # Call library function.
    lib_function = lib.opendp_transformations__make_clamp
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_bounds), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_clamp',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'bounds': bounds
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_clamp(
    bounds: tuple[Any, Any]
):  
    r"""Partial constructor of `make_clamp`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_clamp`

    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_clamp(
        input_domain=input_domain,
        input_metric=input_metric,
        bounds=bounds))
    output.__opendp_dict__ = {
            '__function__': 'then_clamp',
            '__module__': 'transformations',
            '__kwargs__': {
                'bounds': bounds
            },
        }
    return output



def make_consistent_b_ary_tree(
    branching_factor: int,
    TIA: RuntimeTypeDescriptor = "int",
    TOA: RuntimeTypeDescriptor = "float"
) -> Function:
    r"""Postprocessor that makes a noisy b-ary tree internally consistent, and returns the leaf layer.

    The input argument of the function is a balanced `b`-ary tree implicitly stored in breadth-first order
    Tree is assumed to be complete, as in, all leaves on the last layer are on the left.
    Non-existent leaves are assumed to be zero.

    The output remains consistent even when leaf nodes are missing.
    This is due to an adjustment to the original algorithm to apportion corrections to children relative to their variance.


    Required features: `contrib`

    [make_consistent_b_ary_tree in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_consistent_b_ary_tree.html)

    **Citations:**

    * [HRMS09 Boosting the Accuracy of Differentially Private Histograms Through Consistency, section 4.1](https://arxiv.org/pdf/0904.0942.pdf)

    **Supporting Elements:**

    * Input Type:     `Vec<TIA>`
    * Output Type:    `Vec<TOA>`

    .. end-markdown

    :param branching_factor: the maximum number of children
    :type branching_factor: int
    :param TIA: Atomic type of the input data. Should be an integer type.
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic type of the output data. Should be a float type.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA, generics=["TOA"])
    TOA = RuntimeType.parse(type_name=TOA, generics=["TIA"])
    TIA = _substitute(TIA, TOA=TOA) # type: ignore
    TOA = _substitute(TOA, TIA=TIA) # type: ignore

    # Convert arguments to c types.
    c_branching_factor = py_to_c(branching_factor, c_type=ctypes.c_uint32, type_name="u32")
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_consistent_b_ary_tree
    lib_function.argtypes = [ctypes.c_uint32, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_branching_factor, c_TIA, c_TOA), Function))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_consistent_b_ary_tree',
            '__module__': 'transformations',
            '__kwargs__': {
                'branching_factor': branching_factor, 'TIA': TIA, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_count(
    input_domain: Domain,
    input_metric: Metric,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    r"""Make a Transformation that computes a count of the number of records in data.


    Required features: `contrib`

    [make_count in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_count.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<TO>`
    * Output Metric:  `AbsoluteDistance<TO>`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/count/make_count.pdf)

    .. end-markdown

    :param input_domain: Domain of the data type to be privatized.
    :type input_domain: Domain
    :param input_metric: Metric of the data type to be privatized.
    :type input_metric: Metric
    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_count
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TO), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_count',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TO': TO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_count(
    TO: RuntimeTypeDescriptor = "int"
):  
    r"""Partial constructor of `make_count`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_count`

    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_count(
        input_domain=input_domain,
        input_metric=input_metric,
        TO=TO))
    output.__opendp_dict__ = {
            '__function__': 'then_count',
            '__module__': 'transformations',
            '__kwargs__': {
                'TO': TO
            },
        }
    return output



def make_count_by(
    input_domain: Domain,
    input_metric: Metric,
    TV: RuntimeTypeDescriptor = "int"
) -> Transformation:
    r"""Make a Transformation that computes the count of each unique value in data.
    This assumes that the category set is unknown.


    Required features: `contrib`

    [make_count_by in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_count_by.html)

    **Citations:**

    * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TK>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
    * Output Metric:  `L01InfDistance<AbsoluteDistance<TV>>`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param TV: Type of Value. Express counts in terms of this integral type.
    :type TV: :py:ref:`RuntimeTypeDescriptor`
    :return: The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TV = RuntimeType.parse(type_name=TV)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TV = py_to_c(TV, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_count_by
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TV), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_count_by',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TV': TV
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_count_by(
    TV: RuntimeTypeDescriptor = "int"
):  
    r"""Partial constructor of `make_count_by`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_count_by`

    :param TV: Type of Value. Express counts in terms of this integral type.
    :type TV: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_count_by(
        input_domain=input_domain,
        input_metric=input_metric,
        TV=TV))
    output.__opendp_dict__ = {
            '__function__': 'then_count_by',
            '__module__': 'transformations',
            '__kwargs__': {
                'TV': TV
            },
        }
    return output



def make_count_by_categories(
    input_domain: Domain,
    input_metric: Metric,
    categories,
    null_category: bool = True,
    MO: RuntimeTypeDescriptor = "L1Distance<int>",
    TOA: RuntimeTypeDescriptor = "int"
) -> Transformation:
    r"""Make a Transformation that computes the number of times each category appears in the data.
    This assumes that the category set is known.


    Required features: `contrib`

    [make_count_by_categories in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_count_by_categories.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
    * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
    * Output Metric:  `MO`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param categories: The set of categories to compute counts for.
    :param null_category: Include a count of the number of elements that were not in the category set at the end of the vector.
    :type null_category: bool
    :param MO: Output Metric.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type that is numeric.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :return: The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO, generics=["TOA", "TIA"])
    TOA = RuntimeType.parse(type_name=TOA, generics=["MO", "TIA"])
    TIA = get_atom(get_type(input_domain)) # type: ignore
    MO = _substitute(MO, TOA=TOA, TIA=TIA) # type: ignore
    TOA = _substitute(TOA, MO=MO, TIA=TIA) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_null_category = py_to_c(null_category, c_type=ctypes.c_bool, type_name="bool")
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_count_by_categories
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_categories, c_null_category, c_MO, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_count_by_categories',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'categories': categories, 'null_category': null_category, 'MO': MO, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_count_by_categories(
    categories,
    null_category: bool = True,
    MO: RuntimeTypeDescriptor = "L1Distance<int>",
    TOA: RuntimeTypeDescriptor = "int"
):  
    r"""Partial constructor of `make_count_by_categories`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_count_by_categories`

    :param categories: The set of categories to compute counts for.
    :param null_category: Include a count of the number of elements that were not in the category set at the end of the vector.
    :type null_category: bool
    :param MO: Output Metric.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type that is numeric.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_count_by_categories(
        input_domain=input_domain,
        input_metric=input_metric,
        categories=categories,
        null_category=null_category,
        MO=MO,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_count_by_categories',
            '__module__': 'transformations',
            '__kwargs__': {
                'categories': categories, 'null_category': null_category, 'MO': MO, 'TOA': TOA
            },
        }
    return output



def make_count_distinct(
    input_domain: Domain,
    input_metric: Metric,
    TO: RuntimeTypeDescriptor = "int"
) -> Transformation:
    r"""Make a Transformation that computes a count of the number of unique, distinct records in data.


    Required features: `contrib`

    [make_count_distinct in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_count_distinct.html)

    **Citations:**

    * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<TO>`
    * Output Metric:  `AbsoluteDistance<TO>`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TO = RuntimeType.parse(type_name=TO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_TO = py_to_c(TO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_count_distinct
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_TO), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_count_distinct',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'TO': TO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_count_distinct(
    TO: RuntimeTypeDescriptor = "int"
):  
    r"""Partial constructor of `make_count_distinct`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_count_distinct`

    :param TO: Output Type. Must be numeric.
    :type TO: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_count_distinct(
        input_domain=input_domain,
        input_metric=input_metric,
        TO=TO))
    output.__opendp_dict__ = {
            '__function__': 'then_count_distinct',
            '__module__': 'transformations',
            '__kwargs__': {
                'TO': TO
            },
        }
    return output



@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_create_dataframe(
    col_names,
    K: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that constructs a dataframe from a `Vec<Vec<String>>` (a vector of records).


    Required features: `contrib`

    [make_create_dataframe in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_create_dataframe.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<VectorDomain<AtomDomain<String>>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `DataFrameDomain<K>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown

    :param col_names: Column names for each record entry.
    :param K: categorical/hashable data type of column names
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_create_dataframe',
            '__module__': 'transformations',
            '__kwargs__': {
                'col_names': col_names, 'K': K
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_df_cast_default(
    input_domain: Domain,
    input_metric: Metric,
    column_name,
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
) -> Transformation:
    r"""Make a Transformation that casts the elements in a column in a dataframe from type `TIA` to type `TOA`.
    If cast fails, fill with default.


    | `TIA`  | `TIA::default()` |
    | ------ | ---------------- |
    | float  | `0.`             |
    | int    | `0`              |
    | string | `""`             |
    | bool   | `false`          |


    Required features: `contrib`

    [make_df_cast_default in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_df_cast_default.html)

    **Supporting Elements:**

    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `M`
    * Input Metric:   `DataFrameDomain<TK>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param column_name: column name to be transformed
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = RuntimeType.parse(type_name=TIA, generics=["TOA", "TK", "M"])
    TOA = RuntimeType.parse(type_name=TOA, generics=["TIA", "TK", "M"])
    TK = get_atom(get_type(input_domain)) # type: ignore
    M = get_type(input_metric) # type: ignore
    TIA = _substitute(TIA, TOA=TOA, TK=TK, M=M) # type: ignore
    TOA = _substitute(TOA, TIA=TIA, TK=TK, M=M) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_column_name = py_to_c(column_name, c_type=AnyObjectPtr, type_name=TK)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_df_cast_default
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_column_name, c_TIA, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_df_cast_default',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'column_name': column_name, 'TIA': TIA, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_df_cast_default(
    column_name,
    TIA: RuntimeTypeDescriptor,
    TOA: RuntimeTypeDescriptor
):  
    r"""Partial constructor of `make_df_cast_default`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_df_cast_default`

    :param column_name: column name to be transformed
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to cast into
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_df_cast_default(
        input_domain=input_domain,
        input_metric=input_metric,
        column_name=column_name,
        TIA=TIA,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_df_cast_default',
            '__module__': 'transformations',
            '__kwargs__': {
                'column_name': column_name, 'TIA': TIA, 'TOA': TOA
            },
        }
    return output



@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_df_is_equal(
    input_domain: Domain,
    input_metric: Metric,
    column_name,
    value,
    TIA: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.


    Required features: `contrib`

    [make_df_is_equal in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_df_is_equal.html)

    **Supporting Elements:**

    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `M`
    * Input Metric:   `DataFrameDomain<TK>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param column_name: Column name to be transformed
    :param value: Value to check for equality
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=value, generics=["TK", "M"])
    TK = get_atom(get_type(input_domain)) # type: ignore
    M = get_type(input_metric) # type: ignore
    TIA = _substitute(TIA, TK=TK, M=M) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_column_name = py_to_c(column_name, c_type=AnyObjectPtr, type_name=TK)
    c_value = py_to_c(value, c_type=AnyObjectPtr, type_name=TIA)
    c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_df_is_equal
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_column_name, c_value, c_TIA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_df_is_equal',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'column_name': column_name, 'value': value, 'TIA': TIA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_df_is_equal(
    column_name,
    value,
    TIA: Optional[RuntimeTypeDescriptor] = None
):  
    r"""Partial constructor of `make_df_is_equal`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_df_is_equal`

    :param column_name: Column name to be transformed
    :param value: Value to check for equality
    :param TIA: Atomic Input Type to cast from
    :type TIA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_df_is_equal(
        input_domain=input_domain,
        input_metric=input_metric,
        column_name=column_name,
        value=value,
        TIA=TIA))
    output.__opendp_dict__ = {
            '__function__': 'then_df_is_equal',
            '__module__': 'transformations',
            '__kwargs__': {
                'column_name': column_name, 'value': value, 'TIA': TIA
            },
        }
    return output



def make_drop_null(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that drops null values.


    | input_domain                                    |
    | ----------------------------------------------- |
    | `vector_domain(option_domain(atom_domain(TA)))` |
    | `vector_domain(atom_domain(TA))`                |


    Required features: `contrib`

    [make_drop_null in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_drop_null.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<DIA>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<DIA::Imputed>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_drop_null
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_drop_null',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_drop_null(

):  
    r"""Partial constructor of `make_drop_null`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_drop_null`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_drop_null(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_drop_null',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_find(
    input_domain: Domain,
    input_metric: Metric,
    categories
) -> Transformation:
    r"""Find the index of a data value in a set of categories.

    For each value in the input vector, finds the index of the value in `categories`.
    If an index is found, returns `Some(index)`, else `None`.
    Chain with `make_impute_constant` or `make_drop_null` to handle nullity.


    Required features: `contrib`

    [make_find in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_find.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<OptionDomain<AtomDomain<usize>>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: The domain of the input vector.
    :type input_domain: Domain
    :param input_metric: The metric of the input vector.
    :type input_metric: Metric
    :param categories: The set of categories to find indexes from.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = get_atom(get_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))

    # Call library function.
    lib_function = lib.opendp_transformations__make_find
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_categories), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_find',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'categories': categories
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_find(
    categories
):  
    r"""Partial constructor of `make_find`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_find`

    :param categories: The set of categories to find indexes from.
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_find(
        input_domain=input_domain,
        input_metric=input_metric,
        categories=categories))
    output.__opendp_dict__ = {
            '__function__': 'then_find',
            '__module__': 'transformations',
            '__kwargs__': {
                'categories': categories
            },
        }
    return output



def make_find_bin(
    input_domain: Domain,
    input_metric: Metric,
    edges
) -> Transformation:
    r"""Make a transformation that finds the bin index in a monotonically increasing vector of edges.

    For each value in the input vector, finds the index of the bin the value falls into.
    `edges` splits the entire range of `TIA` into bins.
    The first bin at index zero ranges from negative infinity to the first edge, non-inclusive.
    The last bin at index `edges.len()` ranges from the last bin, inclusive, to positive infinity.

    To be valid, `edges` must be unique and ordered.
    `edges` are left inclusive, right exclusive.


    Required features: `contrib`

    [make_find_bin in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_find_bin.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<usize>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: The domain of the input vector.
    :type input_domain: Domain
    :param input_metric: The metric of the input vector.
    :type input_metric: Metric
    :param edges: The set of edges to split bins by.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = get_atom(get_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_edges = py_to_c(edges, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))

    # Call library function.
    lib_function = lib.opendp_transformations__make_find_bin
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_edges), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_find_bin',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'edges': edges
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_find_bin(
    edges
):  
    r"""Partial constructor of `make_find_bin`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_find_bin`

    :param edges: The set of edges to split bins by.
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_find_bin(
        input_domain=input_domain,
        input_metric=input_metric,
        edges=edges))
    output.__opendp_dict__ = {
            '__function__': 'then_find_bin',
            '__module__': 'transformations',
            '__kwargs__': {
                'edges': edges
            },
        }
    return output



def make_identity(
    domain: Domain,
    metric: Metric
) -> Transformation:
    r"""Make a Transformation representing the identity function.

    WARNING: In Python, this function does not ensure that the domain and metric form a valid metric space.
    However, if the domain and metric do not form a valid metric space,
    then the resulting Transformation won't be chainable with any valid Transformation,
    so it cannot be used to introduce an invalid metric space into a chain of valid Transformations.


    Required features: `contrib`, `honest-but-curious`

    [make_identity in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_identity.html)

    **Why honest-but-curious?:**

    For the result to be a valid transformation, the `input_domain` and `input_metric` pairing must form a valid metric space.
    For instance, the symmetric distance metric and atom domain do not form a valid metric space,
    because the metric cannot be used to measure distances between any two elements of an atom domain.
    Whereas, the symmetric distance metric and vector domain,
    or absolute distance metric and atom domain on a scalar type, both form valid metric spaces.

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Domain:  `M`
    * Input Metric:   `D`
    * Output Metric:  `M`

    .. end-markdown

    :param domain: Domain of input data
    :type domain: Domain
    :param metric: Metric on input domain
    :type metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_domain = py_to_c(domain, c_type=Domain, type_name=None)
    c_metric = py_to_c(metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_identity
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_domain, c_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_identity',
            '__module__': 'transformations',
            '__kwargs__': {
                'domain': domain, 'metric': metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_identity(

):  
    r"""Partial constructor of `make_identity`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_identity`


    """
    output = _PartialConstructor(lambda domain, metric: make_identity(
        domain=domain,
        metric=metric))
    output.__opendp_dict__ = {
            '__function__': 'then_identity',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_impute_constant(
    input_domain: Domain,
    input_metric: Metric,
    constant
) -> Transformation:
    r"""Make a Transformation that replaces null/None data with `constant`.

    If chaining after a `make_cast`, the input type is `Option<Vec<TA>>`.
    If chaining after a `make_cast_inherent`, the input type is `Vec<TA>`, where `TA` may take on float NaNs.

    | input_domain                                    |  Input Data Type  |
    | ----------------------------------------------- | ----------------- |
    | `vector_domain(option_domain(atom_domain(TA)))` | `Vec<Option<TA>>` |
    | `vector_domain(atom_domain(TA))`                | `Vec<TA>`         |


    Required features: `contrib`

    [make_impute_constant in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_impute_constant.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<DIA>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<DIA::Imputed>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of the input data. See table above.
    :type input_domain: Domain
    :param input_metric: Metric of the input data. A dataset metric.
    :type input_metric: Metric
    :param constant: Value to replace nulls with.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=get_atom(get_type(input_domain)))

    # Call library function.
    lib_function = lib.opendp_transformations__make_impute_constant
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_constant), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_impute_constant',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'constant': constant
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_impute_constant(
    constant
):  
    r"""Partial constructor of `make_impute_constant`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_impute_constant`

    :param constant: Value to replace nulls with.
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_impute_constant(
        input_domain=input_domain,
        input_metric=input_metric,
        constant=constant))
    output.__opendp_dict__ = {
            '__function__': 'then_impute_constant',
            '__module__': 'transformations',
            '__kwargs__': {
                'constant': constant
            },
        }
    return output



def make_impute_uniform_float(
    input_domain: Domain,
    input_metric: Metric,
    bounds: tuple[Any, Any]
) -> Transformation:
    r"""Make a Transformation that replaces NaN values in `Vec<TA>` with uniformly distributed floats within `bounds`.


    Required features: `contrib`

    [make_impute_uniform_float in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_impute_uniform_float.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TA>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of the input.
    :type input_domain: Domain
    :param input_metric: Metric of the input.
    :type input_metric: Metric
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TA = get_atom(get_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))

    # Call library function.
    lib_function = lib.opendp_transformations__make_impute_uniform_float
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_bounds), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_impute_uniform_float',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'bounds': bounds
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_impute_uniform_float(
    bounds: tuple[Any, Any]
):  
    r"""Partial constructor of `make_impute_uniform_float`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_impute_uniform_float`

    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_impute_uniform_float(
        input_domain=input_domain,
        input_metric=input_metric,
        bounds=bounds))
    output.__opendp_dict__ = {
            '__function__': 'then_impute_uniform_float',
            '__module__': 'transformations',
            '__kwargs__': {
                'bounds': bounds
            },
        }
    return output



def make_index(
    input_domain: Domain,
    input_metric: Metric,
    categories,
    null,
    TOA: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a transformation that treats each element as an index into a vector of categories.


    Required features: `contrib`

    [make_index in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_index.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<usize>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: The domain of the input vector.
    :type input_domain: Domain
    :param input_metric: The metric of the input vector.
    :type input_metric: Metric
    :param categories: The set of categories to index into.
    :param null: Category to return if the index is out-of-range of the category set.
    :param TOA: Atomic Output Type. Output data will be ``Vec<TOA>``.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TOA = RuntimeType.parse_or_infer(type_name=TOA, public_example=get_first(categories))

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TOA]))
    c_null = py_to_c(null, c_type=AnyObjectPtr, type_name=TOA)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_index
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_categories, c_null, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_index',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'categories': categories, 'null': null, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_index(
    categories,
    null,
    TOA: Optional[RuntimeTypeDescriptor] = None
):  
    r"""Partial constructor of `make_index`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_index`

    :param categories: The set of categories to index into.
    :param null: Category to return if the index is out-of-range of the category set.
    :param TOA: Atomic Output Type. Output data will be ``Vec<TOA>``.
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_index(
        input_domain=input_domain,
        input_metric=input_metric,
        categories=categories,
        null=null,
        TOA=TOA))
    output.__opendp_dict__ = {
            '__function__': 'then_index',
            '__module__': 'transformations',
            '__kwargs__': {
                'categories': categories, 'null': null, 'TOA': TOA
            },
        }
    return output



def make_is_equal(
    input_domain: Domain,
    input_metric: Metric,
    value
) -> Transformation:
    r"""Make a Transformation that checks if each element is equal to `value`.


    Required features: `contrib`

    [make_is_equal in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_is_equal.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<bool>>`
    * Output Metric:  `M`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/manipulation/make_is_equal.pdf)

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param value: value to check against
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = get_atom(get_type(input_domain)) # type: ignore
    M = get_type(input_metric) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_value = py_to_c(value, c_type=AnyObjectPtr, type_name=TIA)

    # Call library function.
    lib_function = lib.opendp_transformations__make_is_equal
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_value), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_is_equal',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'value': value
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_is_equal(
    value
):  
    r"""Partial constructor of `make_is_equal`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_is_equal`

    :param value: value to check against
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_is_equal(
        input_domain=input_domain,
        input_metric=input_metric,
        value=value))
    output.__opendp_dict__ = {
            '__function__': 'then_is_equal',
            '__module__': 'transformations',
            '__kwargs__': {
                'value': value
            },
        }
    return output



def make_is_null(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that checks if each element in a vector is null or nan.


    Required features: `contrib`

    [make_is_null in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_is_null.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<DIA>`
    * Output Domain:  `M`
    * Input Metric:   `VectorDomain<AtomDomain<bool>>`
    * Output Metric:  `M`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_is_null
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_is_null',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_is_null(

):  
    r"""Partial constructor of `make_is_null`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_is_null`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_is_null(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_is_null',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_lipschitz_float_mul(
    input_domain: Domain,
    input_metric: Metric,
    constant,
    bounds: tuple[Any, Any]
) -> Transformation:
    r"""Make a transformation that multiplies an aggregate by a constant.

    The bounds clamp the input, in order to bound the increase in sensitivity from float rounding.


    Required features: `contrib`

    [make_lipschitz_float_mul in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_lipschitz_float_mul.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<TA>`
    * Output Domain:  `AbsoluteDistance<TA>`
    * Input Metric:   `AtomDomain<TA>`
    * Output Metric:  `AbsoluteDistance<TA>`

    .. end-markdown

    :param input_domain: The domain of the input.
    :type input_domain: Domain
    :param input_metric: The metric of the input.
    :type input_metric: Metric
    :param constant: The constant to multiply aggregates by.
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TA = get_atom(get_carrier_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=TA)
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[TA, TA]))

    # Call library function.
    lib_function = lib.opendp_transformations__make_lipschitz_float_mul
    lib_function.argtypes = [Domain, Metric, ctypes.c_void_p, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_constant, c_bounds), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_lipschitz_float_mul',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'constant': constant, 'bounds': bounds
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_lipschitz_float_mul(
    constant,
    bounds: tuple[Any, Any]
):  
    r"""Partial constructor of `make_lipschitz_float_mul`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_lipschitz_float_mul`

    :param constant: The constant to multiply aggregates by.
    :param bounds: Tuple of inclusive lower and upper bounds.
    :type bounds: tuple[Any, Any]
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_lipschitz_float_mul(
        input_domain=input_domain,
        input_metric=input_metric,
        constant=constant,
        bounds=bounds))
    output.__opendp_dict__ = {
            '__function__': 'then_lipschitz_float_mul',
            '__module__': 'transformations',
            '__kwargs__': {
                'constant': constant, 'bounds': bounds
            },
        }
    return output



def make_mean(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that computes the mean of bounded data.

    This uses a restricted-sensitivity proof that takes advantage of known dataset size.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.


    Required features: `contrib`

    [make_mean in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_mean.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `MI`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_mean
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_mean',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_mean(

):  
    r"""Partial constructor of `make_mean`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_mean`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_mean(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_mean',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_metric_bounded(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that converts the unbounded dataset metric `MI`
    to the respective bounded dataset metric with a no-op.

    The constructor enforces that the input domain has known size,
    because it must have known size to be valid under a bounded dataset metric.

    | `MI`                 | `MI::BoundedMetric` |
    | -------------------- | ------------------- |
    | SymmetricDistance    | ChangeOneDistance   |
    | InsertDeleteDistance | HammingDistance     |


    Required features: `contrib`

    [make_metric_bounded in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_metric_bounded.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Domain:  `MI`
    * Input Metric:   `D`
    * Output Metric:  `MI::BoundedMetric`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_metric_bounded
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_metric_bounded',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_metric_bounded(

):  
    r"""Partial constructor of `make_metric_bounded`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_metric_bounded`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_metric_bounded(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_metric_bounded',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_metric_unbounded(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that converts the bounded dataset metric `MI`
    to the respective unbounded dataset metric with a no-op.

    | `MI`              | `MI::UnboundedMetric` |
    | ----------------- | --------------------- |
    | ChangeOneDistance | SymmetricDistance     |
    | HammingDistance   | InsertDeleteDistance  |


    Required features: `contrib`

    [make_metric_unbounded in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_metric_unbounded.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Domain:  `MI`
    * Input Metric:   `D`
    * Output Metric:  `MI::UnboundedMetric`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_metric_unbounded
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_metric_unbounded',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_metric_unbounded(

):  
    r"""Partial constructor of `make_metric_unbounded`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_metric_unbounded`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_metric_unbounded(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_metric_unbounded',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_ordered_random(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that converts the unordered dataset metric `SymmetricDistance`
    to the respective ordered dataset metric `InsertDeleteDistance` by assigning a random permutation.

    | `MI`              | `MI::OrderedMetric`  |
    | ----------------- | -------------------- |
    | SymmetricDistance | InsertDeleteDistance |
    | ChangeOneDistance | HammingDistance      |


    Required features: `contrib`

    [make_ordered_random in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_ordered_random.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Domain:  `MI`
    * Input Metric:   `D`
    * Output Metric:  `MI::OrderedMetric`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_ordered_random
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_ordered_random',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_ordered_random(

):  
    r"""Partial constructor of `make_ordered_random`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_ordered_random`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_ordered_random(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_ordered_random',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_quantile_score_candidates(
    input_domain: Domain,
    input_metric: Metric,
    candidates,
    alpha: float
) -> Transformation:
    r"""Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.


    Required features: `contrib`

    [make_quantile_score_candidates in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_quantile_score_candidates.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
    * Output Domain:  `MI`
    * Input Metric:   `VectorDomain<AtomDomain<u64>>`
    * Output Metric:  `LInfDistance<u64>`

    **Proof Definition:**

    [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/quantile_score_candidates/make_quantile_score_candidates.pdf)

    .. end-markdown

    :param input_domain: Uses a smaller sensitivity when the size of vectors in the input domain is known.
    :type input_domain: Domain
    :param input_metric: Either SymmetricDistance or InsertDeleteDistance.
    :type input_metric: Metric
    :param candidates: Potential quantiles to score
    :param alpha: a value in $[0, 1]$. Choose 0.5 for median
    :type alpha: float
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TIA = get_atom(get_type(input_domain)) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_candidates = py_to_c(candidates, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
    c_alpha = py_to_c(alpha, c_type=ctypes.c_double, type_name="f64")

    # Call library function.
    lib_function = lib.opendp_transformations__make_quantile_score_candidates
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_double]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_candidates, c_alpha), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_quantile_score_candidates',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'candidates': candidates, 'alpha': alpha
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_quantile_score_candidates(
    candidates,
    alpha: float
):  
    r"""Partial constructor of `make_quantile_score_candidates`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_quantile_score_candidates`

    :param candidates: Potential quantiles to score
    :param alpha: a value in $[0, 1]$. Choose 0.5 for median
    :type alpha: float
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_quantile_score_candidates(
        input_domain=input_domain,
        input_metric=input_metric,
        candidates=candidates,
        alpha=alpha))
    output.__opendp_dict__ = {
            '__function__': 'then_quantile_score_candidates',
            '__module__': 'transformations',
            '__kwargs__': {
                'candidates': candidates, 'alpha': alpha
            },
        }
    return output



def make_quantiles_from_counts(
    bin_edges,
    alphas,
    interpolation: str = "linear",
    TA: Optional[RuntimeTypeDescriptor] = None,
    F: RuntimeTypeDescriptor = "float"
) -> Function:
    r"""Postprocess a noisy array of summary counts into quantiles.


    Required features: `contrib`

    [make_quantiles_from_counts in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_quantiles_from_counts.html)

    **Supporting Elements:**

    * Input Type:     `Vec<TA>`
    * Output Type:    `Vec<TA>`

    .. end-markdown

    :param bin_edges: The edges that the input data was binned into before counting.
    :param alphas: Return all specified ``alpha``-quantiles.
    :param interpolation: Must be one of ``linear`` or ``nearest``
    :type interpolation: str
    :param TA: Atomic Type of the bin edges and data.
    :type TA: :py:ref:`RuntimeTypeDescriptor`
    :param F: Float type of the alpha argument. One of ``f32`` or ``f64``
    :type F: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    TA = RuntimeType.parse_or_infer(type_name=TA, public_example=get_first(bin_edges), generics=["F"])
    F = RuntimeType.parse_or_infer(type_name=F, public_example=get_first(alphas), generics=["TA"])
    TA = _substitute(TA, F=F) # type: ignore
    F = _substitute(F, TA=TA) # type: ignore

    # Convert arguments to c types.
    c_bin_edges = py_to_c(bin_edges, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TA]))
    c_alphas = py_to_c(alphas, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[F]))
    c_interpolation = py_to_c(interpolation, c_type=ctypes.c_char_p, type_name="String")
    c_TA = py_to_c(TA, c_type=ctypes.c_char_p)
    c_F = py_to_c(F, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_quantiles_from_counts
    lib_function.argtypes = [AnyObjectPtr, AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_bin_edges, c_alphas, c_interpolation, c_TA, c_F), Function))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_quantiles_from_counts',
            '__module__': 'transformations',
            '__kwargs__': {
                'bin_edges': bin_edges, 'alphas': alphas, 'interpolation': interpolation, 'TA': TA, 'F': F
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_resize(
    input_domain: Domain,
    input_metric: Metric,
    size: int,
    constant,
    MO: RuntimeTypeDescriptor = "SymmetricDistance"
) -> Transformation:
    r"""Make a Transformation that either truncates or imputes records
    with `constant` to match a provided `size`.


    Required features: `contrib`

    [make_resize in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_resize.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<TA>>`
    * Output Domain:  `MI`
    * Input Metric:   `VectorDomain<AtomDomain<TA>>`
    * Output Metric:  `MO`

    .. end-markdown

    :param input_domain: Domain of input data.
    :type input_domain: Domain
    :param input_metric: Metric of input data.
    :type input_metric: Metric
    :param size: Number of records in output data.
    :type size: int
    :param constant: Value to impute with.
    :param MO: Output Metric. One of ``InsertDeleteDistance`` or ``SymmetricDistance``
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :return: A vector of the same type `TA`, but with the provided `size`.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_constant = py_to_c(constant, c_type=AnyObjectPtr, type_name=get_atom(get_type(input_domain)))
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_resize
    lib_function.argtypes = [Domain, Metric, ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_size, c_constant, c_MO), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_resize',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'size': size, 'constant': constant, 'MO': MO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_resize(
    size: int,
    constant,
    MO: RuntimeTypeDescriptor = "SymmetricDistance"
):  
    r"""Partial constructor of `make_resize`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_resize`

    :param size: Number of records in output data.
    :type size: int
    :param constant: Value to impute with.
    :param MO: Output Metric. One of ``InsertDeleteDistance`` or ``SymmetricDistance``
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_resize(
        input_domain=input_domain,
        input_metric=input_metric,
        size=size,
        constant=constant,
        MO=MO))
    output.__opendp_dict__ = {
            '__function__': 'then_resize',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'constant': constant, 'MO': MO
            },
        }
    return output



@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_select_column(
    key,
    TOA: RuntimeTypeDescriptor,
    K: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that retrieves the column `key` from a dataframe as `Vec<TOA>`.


    Required features: `contrib`

    [make_select_column in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_select_column.html)

    **Supporting Elements:**

    * Input Domain:   `DataFrameDomain<K>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown

    :param key: categorical/hashable data type of the key/column name
    :param K: data type of key
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :param TOA: Atomic Output Type to downcast vector to
    :type TOA: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    K = RuntimeType.parse_or_infer(type_name=K, public_example=key, generics=["TOA"])
    TOA = RuntimeType.parse(type_name=TOA, generics=["K"])
    K = _substitute(K, TOA=TOA) # type: ignore
    TOA = _substitute(TOA, K=K) # type: ignore

    # Convert arguments to c types.
    c_key = py_to_c(key, c_type=AnyObjectPtr, type_name=K)
    c_K = py_to_c(K, c_type=ctypes.c_char_p)
    c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_select_column
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_key, c_K, c_TOA), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_select_column',
            '__module__': 'transformations',
            '__kwargs__': {
                'key': key, 'K': K, 'TOA': TOA
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_float_checked_sum(
    size: int,
    bounds: tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded floats with known dataset size.

    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.

    | S (summation algorithm) | input type     |
    | ----------------------- | -------------- |
    | `Sequential<S::Item>`   | `Vec<S::Item>` |
    | `Pairwise<S::Item>`     | `Vec<S::Item>` |

    `S::Item` is the type of all of the following:
    each bound, each element in the input data, the output data, and the output sensitivity.

    For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
    set `S` to `Pairwise<f32>`.


    Required features: `contrib`

    [make_sized_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_float_checked_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param S: Summation algorithm to use over some data type ``T`` (``T`` is shorthand for ``S::Item``)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_float_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_float_checked_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_float_ordered_sum(
    size: int,
    bounds: tuple[Any, Any],
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded floats with known ordering and dataset size.

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


    Required features: `contrib`

    [make_sized_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_float_ordered_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `InsertDeleteDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param S: Summation algorithm to use over some data type ``T`` (``T`` is shorthand for ``S::Item``)
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom_or_infer(S, get_first(bounds)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_float_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_float_ordered_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_int_checked_sum(
    size: int,
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints.
    The effective range is reduced, as (bounds * size) must not overflow.


    Required features: `contrib`

    [make_sized_bounded_int_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_int_checked_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_checked_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_int_checked_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_int_monotonic_sum(
    size: int,
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints,
    where all values share the same sign.


    Required features: `contrib`

    [make_sized_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_int_monotonic_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_monotonic_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_int_monotonic_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_int_ordered_sum(
    size: int,
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints with known dataset size.

    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    You may need to use `make_ordered_random` to impose an ordering on the data.


    Required features: `contrib`

    [make_sized_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_int_ordered_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `InsertDeleteDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_ordered_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_int_ordered_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sized_bounded_int_split_sum(
    size: int,
    bounds: tuple[Any, Any],
    T: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded ints with known dataset size.

    This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
    Adds the saturating sum of the positives to the saturating sum of the negatives.


    Required features: `contrib`

    [make_sized_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sized_bounded_int_split_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param size: Number of records in input data.
    :type size: int
    :param bounds: Tuple of lower and upper bounds for data in the input domain.
    :type bounds: tuple[Any, Any]
    :param T: Atomic Input Type and Output Type
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=get_first(bounds))

    # Convert arguments to c types.
    c_size = py_to_c(size, c_type=ctypes.c_size_t, type_name="usize")
    c_bounds = py_to_c(bounds, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Tuple', args=[T, T]))
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sized_bounded_int_split_sum
    lib_function.argtypes = [ctypes.c_size_t, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_size, c_bounds, c_T), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sized_bounded_int_split_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'size': size, 'bounds': bounds, 'T': T
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_split_dataframe(
    separator: str,
    col_names,
    K: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that splits each record in a String into a `Vec<Vec<String>>`,
    and loads the resulting table into a dataframe keyed by `col_names`.


    Required features: `contrib`

    [make_split_dataframe in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_split_dataframe.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<String>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `DataFrameDomain<K>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown

    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :param col_names: Column names for each record entry.
    :param K: categorical/hashable data type of column names
    :type K: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_split_dataframe',
            '__module__': 'transformations',
            '__kwargs__': {
                'separator': separator, 'col_names': col_names, 'K': K
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_split_lines(

) -> Transformation:
    r"""Make a Transformation that takes a string and splits it into a `Vec<String>` of its lines.


    Required features: `contrib`

    [make_split_lines in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_split_lines.html)

    **Supporting Elements:**

    * Input Domain:   `AtomDomain<String>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `VectorDomain<AtomDomain<String>>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_split_lines',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_split_records(
    separator: str
) -> Transformation:
    r"""Make a Transformation that splits each record in a `Vec<String>` into a `Vec<Vec<String>>`.


    Required features: `contrib`

    [make_split_records in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_split_records.html)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<String>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `VectorDomain<VectorDomain<AtomDomain<String>>>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown

    :param separator: The token(s) that separate entries in each record.
    :type separator: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_split_records',
            '__module__': 'transformations',
            '__kwargs__': {
                'separator': separator
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_stable_expr(
    input_domain: Domain,
    input_metric: Metric,
    expr
) -> Transformation:
    r"""Create a stable transformation from an [`Expr`].


    Required features: `contrib`

    [make_stable_expr in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_stable_expr.html)

    **Supporting Elements:**

    * Input Domain:   `WildExprDomain`
    * Output Domain:  `MI`
    * Input Metric:   `ExprDomain`
    * Output Metric:  `MO`

    .. end-markdown

    :param input_domain: The domain of the input data.
    :type input_domain: Domain
    :param input_metric: How to measure distances between neighboring input data sets.
    :type input_metric: Metric
    :param expr: The expression to be analyzed for stability.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_expr = py_to_c(expr, c_type=AnyObjectPtr, type_name="Expr")

    # Call library function.
    lib_function = lib.opendp_transformations__make_stable_expr
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_expr), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_stable_expr',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'expr': expr
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_stable_expr(
    expr
):  
    r"""Partial constructor of `make_stable_expr`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_stable_expr`

    :param expr: The expression to be analyzed for stability.
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_stable_expr(
        input_domain=input_domain,
        input_metric=input_metric,
        expr=expr))
    output.__opendp_dict__ = {
            '__function__': 'then_stable_expr',
            '__module__': 'transformations',
            '__kwargs__': {
                'expr': expr
            },
        }
    return output



def make_stable_lazyframe(
    input_domain: Domain,
    input_metric: Metric,
    lazyframe,
    MO: RuntimeTypeDescriptor = "MI"
) -> Transformation:
    r"""Create a stable transformation from a [`LazyFrame`].


    Required features: `contrib`

    [make_stable_lazyframe in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_stable_lazyframe.html)

    **Supporting Elements:**

    * Input Domain:   `LazyFrameDomain`
    * Output Domain:  `MI`
    * Input Metric:   `LazyFrameDomain`
    * Output Metric:  `MO`

    .. end-markdown

    :param input_domain: The domain of the input data.
    :type input_domain: Domain
    :param input_metric: How to measure distances between neighboring input data sets.
    :type input_metric: Metric
    :param lazyframe: The [``LazyFrame``] to be analyzed.
    :param MO: The type of the output metric.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    MO = RuntimeType.parse(type_name=MO)

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_lazyframe = py_to_c(lazyframe, c_type=AnyObjectPtr, type_name="LazyFrame")
    c_MO = py_to_c(MO, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_stable_lazyframe
    lib_function.argtypes = [Domain, Metric, AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_lazyframe, c_MO), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_stable_lazyframe',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'lazyframe': lazyframe, 'MO': MO
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_stable_lazyframe(
    lazyframe,
    MO: RuntimeTypeDescriptor = "MI"
):  
    r"""Partial constructor of `make_stable_lazyframe`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_stable_lazyframe`

    :param lazyframe: The [``LazyFrame``] to be analyzed.
    :param MO: The type of the output metric.
    :type MO: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_stable_lazyframe(
        input_domain=input_domain,
        input_metric=input_metric,
        lazyframe=lazyframe,
        MO=MO))
    output.__opendp_dict__ = {
            '__function__': 'then_stable_lazyframe',
            '__module__': 'transformations',
            '__kwargs__': {
                'lazyframe': lazyframe, 'MO': MO
            },
        }
    return output



@deprecated(version="0.12.0", reason="Use Polars instead.")
def make_subset_by(
    indicator_column,
    keep_columns,
    TK: Optional[RuntimeTypeDescriptor] = None
) -> Transformation:
    r"""Make a Transformation that subsets a dataframe by a boolean column.


    Required features: `contrib`

    [make_subset_by in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_subset_by.html)

    **Supporting Elements:**

    * Input Domain:   `DataFrameDomain<TK>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `DataFrameDomain<TK>`
    * Output Metric:  `SymmetricDistance`

    .. end-markdown

    :param indicator_column: name of the boolean column that indicates inclusion in the subset
    :param keep_columns: list of column names to apply subset to
    :param TK: Type of the column name
    :type TK: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
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
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_subset_by',
            '__module__': 'transformations',
            '__kwargs__': {
                'indicator_column': indicator_column, 'keep_columns': keep_columns, 'TK': TK
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_sum(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that computes the sum of bounded data.
    Use `make_clamp` to bound data.

    If dataset size is known, uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.


    Required features: `contrib`

    [make_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sum.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<T>>`
    * Output Domain:  `MI`
    * Input Metric:   `AtomDomain<T>`
    * Output Metric:  `AbsoluteDistance<T>`

    .. end-markdown

    :param input_domain: Domain of the input data.
    :type input_domain: Domain
    :param input_metric: One of ``SymmetricDistance`` or ``InsertDeleteDistance``.
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sum
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_sum(

):  
    r"""Partial constructor of `make_sum`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_sum`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_sum(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_sum',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_sum_of_squared_deviations(
    input_domain: Domain,
    input_metric: Metric,
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the sum of squared deviations of bounded data.

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


    Required features: `contrib`

    [make_sum_of_squared_deviations in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_sum_of_squared_deviations.html)

    **Citations:**

    * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
    * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param S: Summation algorithm to use on data type ``T``. One of ``Sequential<T>`` or ``Pairwise<T>``.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom(get_type(input_domain)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_sum_of_squared_deviations
    lib_function.argtypes = [Domain, Metric, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_sum_of_squared_deviations',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_sum_of_squared_deviations(
    S: RuntimeTypeDescriptor = "Pairwise<T>"
):  
    r"""Partial constructor of `make_sum_of_squared_deviations`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_sum_of_squared_deviations`

    :param S: Summation algorithm to use on data type ``T``. One of ``Sequential<T>`` or ``Pairwise<T>``.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_sum_of_squared_deviations(
        input_domain=input_domain,
        input_metric=input_metric,
        S=S))
    output.__opendp_dict__ = {
            '__function__': 'then_sum_of_squared_deviations',
            '__module__': 'transformations',
            '__kwargs__': {
                'S': S
            },
        }
    return output



def make_unordered(
    input_domain: Domain,
    input_metric: Metric
) -> Transformation:
    r"""Make a Transformation that converts the ordered dataset metric `MI`
    to the respective ordered dataset metric with a no-op.

    | `MI`                 | `MI::UnorderedMetric` |
    | -------------------- | --------------------- |
    | InsertDeleteDistance | SymmetricDistance     |
    | HammingDistance      | ChangeOneDistance     |


    Required features: `contrib`

    [make_unordered in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_unordered.html)

    **Supporting Elements:**

    * Input Domain:   `D`
    * Output Domain:  `MI`
    * Input Metric:   `D`
    * Output Metric:  `MI::UnorderedMetric`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)

    # Call library function.
    lib_function = lib.opendp_transformations__make_unordered
    lib_function.argtypes = [Domain, Metric]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_unordered',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_unordered(

):  
    r"""Partial constructor of `make_unordered`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_unordered`


    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_unordered(
        input_domain=input_domain,
        input_metric=input_metric))
    output.__opendp_dict__ = {
            '__function__': 'then_unordered',
            '__module__': 'transformations',
            '__kwargs__': {
                
            },
        }
    return output



def make_user_transformation(
    input_domain: Domain,
    input_metric: Metric,
    output_domain: Domain,
    output_metric: Metric,
    function,
    stability_map
) -> Transformation:
    r"""Construct a Transformation from user-defined callbacks.


    Required features: `contrib`, `honest-but-curious`

    **Why honest-but-curious?:**

    This constructor only returns a valid transformation if for every pair of elements $x, x'$ in `input_domain`,
    and for every pair `(d_in, d_out)`,
    where `d_in` has the associated type for `input_metric` and `d_out` has the associated type for `output_metric`,
    if $x, x'$ are `d_in`-close under `input_metric`, `stability_map(d_in)` does not raise an exception,
    and `stability_map(d_in) <= d_out`,
    then `function(x), function(x')` are d_out-close under `output_metric`.

    In addition, for every element $x$ in `input_domain`, `function(x)` is a member of `output_domain` or raises a data-independent runtime exception.

    In addition, `function` must not have side-effects, and `stability_map` must be a pure function.

    .. end-markdown

    :param input_domain: A domain describing the set of valid inputs for the function.
    :type input_domain: Domain
    :param input_metric: The metric from which distances between adjacent inputs are measured.
    :type input_metric: Metric
    :param output_domain: A domain describing the set of valid outputs of the function.
    :type output_domain: Domain
    :param output_metric: The metric from which distances between outputs of adjacent inputs are measured.
    :type output_metric: Metric
    :param function: A function mapping data from ``input_domain`` to ``output_domain``.
    :param stability_map: A function mapping distances from ``input_metric`` to ``output_metric``.
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib", "honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name="AnyDomain")
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name="AnyMetric")
    c_output_domain = py_to_c(output_domain, c_type=Domain, type_name="AnyDomain")
    c_output_metric = py_to_c(output_metric, c_type=Metric, type_name="AnyMetric")
    c_function = py_to_c(function, c_type=CallbackFnPtr, type_name=domain_carrier_type(output_domain))
    c_stability_map = py_to_c(stability_map, c_type=CallbackFnPtr, type_name=metric_distance_type(output_metric))

    # Call library function.
    lib_function = lib.opendp_transformations__make_user_transformation
    lib_function.argtypes = [Domain, Metric, Domain, Metric, CallbackFnPtr, CallbackFnPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_output_domain, c_output_metric, c_function, c_stability_map), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_user_transformation',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'output_domain': output_domain, 'output_metric': output_metric, 'function': function, 'stability_map': stability_map
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def make_variance(
    input_domain: Domain,
    input_metric: Metric,
    ddof: int = 1,
    S: RuntimeTypeDescriptor = "Pairwise<T>"
) -> Transformation:
    r"""Make a Transformation that computes the variance of bounded data.

    This uses a restricted-sensitivity proof that takes advantage of known dataset size.
    Use `make_clamp` to bound data and `make_resize` to establish dataset size.


    Required features: `contrib`

    [make_variance in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/transformations/fn.make_variance.html)

    **Citations:**

    * [DHK15 Differential Privacy for Social Science Inference](http://hona.kr/papers/files/DOrazioHonakerKingPrivacy.pdf)

    **Supporting Elements:**

    * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
    * Output Domain:  `SymmetricDistance`
    * Input Metric:   `AtomDomain<S::Item>`
    * Output Metric:  `AbsoluteDistance<S::Item>`

    .. end-markdown

    :param input_domain: Domain of input data
    :type input_domain: Domain
    :param input_metric: Metric on input domain
    :type input_metric: Metric
    :param ddof: Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
    :type ddof: int
    :param S: Summation algorithm to use on data type ``T``. One of ``Sequential<T>`` or ``Pairwise<T>``.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("contrib")

    # Standardize type arguments.
    S = RuntimeType.parse(type_name=S, generics=["T"])
    T = get_atom(get_type(input_domain)) # type: ignore
    S = _substitute(S, T=T) # type: ignore

    # Convert arguments to c types.
    c_input_domain = py_to_c(input_domain, c_type=Domain, type_name=None)
    c_input_metric = py_to_c(input_metric, c_type=Metric, type_name=None)
    c_ddof = py_to_c(ddof, c_type=ctypes.c_size_t, type_name="usize")
    c_S = py_to_c(S, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_transformations__make_variance
    lib_function.argtypes = [Domain, Metric, ctypes.c_size_t, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_input_domain, c_input_metric, c_ddof, c_S), Transformation))
    try:
        output.__opendp_dict__ = {
            '__function__': 'make_variance',
            '__module__': 'transformations',
            '__kwargs__': {
                'input_domain': input_domain, 'input_metric': input_metric, 'ddof': ddof, 'S': S
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output

def then_variance(
    ddof: int = 1,
    S: RuntimeTypeDescriptor = "Pairwise<T>"
):  
    r"""Partial constructor of `make_variance`.

    .. end-markdown

    .. seealso:: 
      Delays application of ``input_domain`` and ``input_metric`` in :py:func:`~opendp.transformations.make_variance`

    :param ddof: Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
    :type ddof: int
    :param S: Summation algorithm to use on data type ``T``. One of ``Sequential<T>`` or ``Pairwise<T>``.
    :type S: :py:ref:`RuntimeTypeDescriptor`
    """
    output = _PartialConstructor(lambda input_domain, input_metric: make_variance(
        input_domain=input_domain,
        input_metric=input_metric,
        ddof=ddof,
        S=S))
    output.__opendp_dict__ = {
            '__function__': 'then_variance',
            '__module__': 'transformations',
            '__kwargs__': {
                'ddof': ddof, 'S': S
            },
        }
    return output

