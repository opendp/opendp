.. _transformations-user-guide:

Transformations
===============

(See also :py:mod:`opendp.transformations` in the API reference.)

This section gives a high-level overview of the transformations that are available in the library.
Refer to the :ref:`transformation` section for an explanation of what a transformation is.

As covered in the :ref:`chaining` section, the intermediate :ref:`domains <domains-user-guide>` need to match when chaining.
Each transformation has a carefully chosen input domain and output domain that supports their relation.


.. note::
  If you pass information collected directly from the dataset into constructors, the differential privacy guarantees may be compromised.
  Constructor arguments should be either:

  * Public information, like information from a codebook or prior domain expertise
  * Other DP releases on the data


Preprocessing is the series of transformations that shape the data into a domain that is conformable with the aggregator.

You will need to choose the proper transformations from the sections below in order to chain with the aggregator you intend to use.
The sections below are in the order you would typically chain transformations together,
but you may want to peek at the aggregator section at the end first,
to identify the input domain that you'll need to preprocess to.

Dataframe
---------
These transformations are for loading data into a dataframe and retrieving columns from a dataframe.
If you just want to load data from a CSV or TSV into a dataframe,
you'll probably want to use :func:`opendp.transformations.make_split_dataframe`.

Use :func:`opendp.transformations.make_select_column` to retrieve a column from the dataframe.

The other dataframe transformations are more situational.

Be warned that it is not currently possible to directly load and unload dataframes from the library in bindings languages!
You need to chain with ``make_select_column`` first.

.. list-table::
   :header-rows: 1

   * - Preprocessor
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_split_dataframe`
     - ``AtomDomain<String>``
     - ``DataFrameDomain<K>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_split_lines`
     - ``AtomDomain<String>``
     - ``VectorDomain<AtomDomain<String>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_split_records`
     - ``VectorDomain<AtomDomain<String>>``
     - ``VectorDomain<VectorDomain<AtomDomain<String>>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_create_dataframe`
     - ``VectorDomain<VectorDomain<AtomDomain<String>>>``
     - ``DataFrameDomain<K>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_select_column`
     - ``DataFrameDomain<K>``
     - ``VectorDomain<AtomDomain<TOA>>``
     - ``SymmetricDistance``


Dataframe Subsetting
--------------------
It can be useful to subset to data that meets a certain condition. 
The library contains some transformations that can be used to create a predicate column,
and :func:`opendp.transformations.make_subset_by` to filter by the predicate column.

.. list-table::
   :header-rows: 1

   * - Preprocessor
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_df_cast_default`
     - ``DataFrameDomain<K>``
     - ``DataFrameDomain<K>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_df_is_equal`
     - ``DataFrameDomain<K>``
     - ``DataFrameDomain<K>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_subset_by`
     - ``DataFrameDomain<K>``
     - ``DataFrameDomain<K>``
     - ``SymmetricDistance``
     

Casting
-------
Any time you want to convert between data types, you'll want to use a casting transformation.
In particular, in pipelines that load dataframes from CSV files, it is very common to cast from Strings to some other type.

Depending on the caster you choose, the output data may be null and you will be required to chain with an imputer.

.. list-table::
   :header-rows: 1

   * - Caster
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_cast`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<OptionDomain<AtomDomain<TOA>>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_cast_default`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<TOA>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_cast_inherent`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<TOA>>`` (with nullity)
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_is_equal`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<bool>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_is_null`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<bool>>``
     - ``SymmetricDistance``


Imputation
----------

Null values are tricky to handle in a differentially private manner.
If we were to allow aggregations to propagate null,
then aggregations provide a non-differentially-private bit revealing the existence of nullity in the dataset.
If we were to implicitly drop nulls from sized aggregations
(like a dataset mean where we divide by the number of non-null elements),
then the sensitivity of non-null individuals is underestimated.
Therefore, most aggregators must be fed completely non-null data.
We can ensure data is non-null by imputing.

When you cast with :func:`opendp.transformations.make_cast` or :func:`opendp.transformations.make_cast_default`,
the cast may fail, so the output domain may include null values (``OptionDomain`` or ``AtomDomain`` with nullity).
We have provided imputation transformations to transform the data domain to the non-null ``VectorDomain<AtomDomain<TA>>``.

You may also be in a situation where you want to bypass dataframe loading and casting
because you already have a vector of floats loaded into memory.
In this case, you should start your chain with an imputer if the floats are potentially null.

:OptionDomain: A representation of nulls using an Option type (``Option<bool>``, ``Option<i32>``, etc).
:AtomDomain: A representation of nulls using the data type itself (``f32`` and ``f64``).

The :func:`opendp.transformations.make_impute_constant` transformation supports imputing on either of these representations of nullity,
so long as you pass the DA (atomic domain) type argument.

.. list-table::
   :header-rows: 1

   * - Imputer
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_impute_constant`
     - ``VectorDomain<OptionDomain<AtomDomain<TA>>>``
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_impute_constant`
     - ``VectorDomain<AtomDomain<TA>>`` (with nullity)
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_impute_uniform_float`
     - ``VectorDomain<AtomDomain<TA>>`` (with nullity)
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_drop_null`
     - ``VectorDomain<OptionDomain<AtomDomain<TA>>>``
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_drop_null`
     - ``VectorDomain<AtomDomain<TA>>`` (with nullity)
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``

Indexing
--------
Indexing operations provide a way to relabel categorical data, or bin numeric data into categorical data.
These operations work with ``usize`` data types: an integer data type representing an index.
:func:`opendp.transformations.make_find` finds the index of each input datum in a set of categories.
In other words, it transforms a categorical data vector to a vector of numeric indices.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> dp.enable_features('contrib')
        >>> finder = (
        ...     # define the input space
        ...     (dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()) >>
        ...     # find the index of each input datum in the categories list
        ...     dp.t.then_find(categories=["A", "B", "C"]) >>
        ...     # impute any input datum that are not a part of the categories list as 3
        ...     dp.t.then_impute_constant(3)
        ... )
        >>> finder(["A", "B", "C", "A", "D"])
        [0, 1, 2, 0, 3]

:func:`opendp.transformations.make_find_bin` is a binning operation that transforms numerical input data to a vector of bin indices.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> binner = dp.t.make_find_bin(
        ...     dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance(),
        ...     edges=[1., 2., 10.])
        >>> binner([0., 1., 3., 15.])
        [0, 1, 2, 3]

:func:`opendp.transformations.make_index` uses each indicial input datum as an index into a category set.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> indexer = dp.t.make_index(
        ...     dp.vector_domain(dp.atom_domain(T=dp.usize)), dp.symmetric_distance(),
        ...     categories=["A", "B", "C"], null="D")
        >>> indexer([0, 1, 2, 3, 2342])
        ['A', 'B', 'C', 'D', 'D']

You can use combinations of the indicial transformers to map hashable data to integers, bin numeric types, relabel hashable types, and label bins.

.. list-table::
   :header-rows: 1

   * - Indexer
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_find`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<OptionDomain<AtomDomain<usize>>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_find_bin`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<usize>>``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_index`
     - ``VectorDomain<AtomDomain<usize>>``
     - ``VectorDomain<AtomDomain<TOA>>``
     - ``SymmetricDistance``


Clamping
--------
Many aggregators depend on bounded data to limit the influence that perturbing an individual may have on a query.
For example, the stability map for the :func:`opendp.transformations.make_sum` aggregator is ``d_out = d_in * max(|L|, U)``.
This relation states that adding or removing ``d_in`` records may influence the sum by ``d_in`` * the greatest magnitude of a record.

Any aggregator that needs bounded data will indicate it in the function name.
In these kinds of aggregators the relations make use of the clamping bounds ``L`` and ``U`` to translate ``d_in`` to ``d_out``.

Clamping generally happens after casting and imputation but before resizing.
Only chain with a clamp transformation if the aggregator you intend to use needs bounded data.

.. list-table::
   :header-rows: 1

   * - Clamper
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_clamp`
     - ``VectorDomain<AtomDomain<TA>>``
     - ``VectorDomain<AtomDomain<TA>>`` (with bounds)
     - ``SymmetricDistance``

Dataset Ordering
----------------
Most dataset-to-dataset transformations are not sensitive to the order of elements within the dataset.
This includes all row-by-row transformations. 
These transformations are written to operate with symmetric distances.

Transformations that are sensitive to the order of elements in the dataset use the InsertDeleteDistance metric instead.
It is common for aggregators to be sensitive to the dataset ordering.

The following transformations are used to relate dataset metrics that are not sensitive to ordering (``SymmetricDistance`` and ``ChangeOneDistance``) 
to metrics that are sensitive to ordering (``InsertDeleteDistance`` and ``HammingDistance`` respectively).

.. list-table::
   :header-rows: 1

   * - Caster
     - Input/Output Domain
     - Input Metric
     - Output Metric
   * - :func:`opendp.transformations.make_ordered_random`
     - ``VectorDomain<AtomDomain<TA>>``
     - ``SymmetricDistance``
     - ``InsertDeleteDistance``
   * - :func:`opendp.transformations.make_unordered`
     - ``VectorDomain<AtomDomain<TA>>``
     - ``InsertDeleteDistance``
     - ``SymmetricDistance``

Bounded Metrics
---------------
You may be more familiar with "bounded" differential privacy, where dataset distances are expressed in terms of the number of changed rows.
Expressing dataset distances in this manner is more restrictive, as edit distances are only valid for datasets with a fixed size.
Generally speaking, if a dataset differs from a neighboring dataset by no more than ``k`` edits, then they differ by no more than ``2k`` additions and removals.
We therefore write all transformations in terms of the more general "unbounded"-dp metrics ``SymmetricDistance`` and ``InsertDeleteDistance``, 
and provide the following constructors to convert to/from "bounded"-dp metrics ``ChangeOneDistance`` and ``HammingDistance`` respectively.

.. list-table::
   :header-rows: 1

   * - Caster
     - Input/Output Domain
     - Input Metric
     - Output Metric
   * - :func:`opendp.transformations.make_metric_bounded`
     - ``SizedDomain<VectorDomain<AtomDomain<TA>>>``
     - ``SymmetricDistance``
     - ``ChangeOneDistance``
   * - :func:`opendp.transformations.make_metric_bounded`
     - ``SizedDomain<VectorDomain<AtomDomain<TA>>>``
     - ``InsertDeleteDistance``
     - ``HammingDistance``
   * - :func:`opendp.transformations.make_metric_unbounded`
     - ``SizedDomain<VectorDomain<AtomDomain<TA>>>``
     - ``ChangeOneDistance``
     - ``SymmetricDistance``
   * - :func:`opendp.transformations.make_metric_unbounded`
     - ``SizedDomain<VectorDomain<AtomDomain<TA>>>``
     - ``HammingDistance``
     - ``InsertDeleteDistance``


Resizing
--------
The resize transformation takes a dataset with unknown size, and a target size (that could itself be estimated with a DP count).
If the dataset has fewer records than the target size, additional rows will be imputed.
If the dataset has more records than the target size, a simple sample of the rows is taken.

In the case that a neighboring dataset adds one record to the dataset, and it causes one fewer imputation,
the resulting dataset distance is 2.
Therefore, the resize transformation is 2-stable: ``map(d_in) = 2 * d_in``.

Similarly to data bounds, many aggregators calibrate their stability map based on knowledge of a known dataset size.
For example, the relation downstream for the :func:`opendp.transformations.make_mean` aggregator is ``map(d_in) = d_in // 2 * (U - L) / n``.
Notice that any addition and removal may, in the worst case, change a record from ``L`` to ``U``.
Such a substitution would influence the mean by ``(U - L) / n``.

Any aggregator that needs sized data will indicate it in the function name.
In these kinds of aggregators, the relations need knowledge about the dataset size ``n`` to translate ``d_in`` to ``d_out``.

Resizing generally happens after clamping.
Only chain with a resize transformation if the aggregator you intend to use needs sized data.

The input and output metrics may be configured to any combination of ``SymmetricDistance`` and ``InsertDeleteDistance``.

.. list-table::
   :header-rows: 1

   * - Resizer
     - Input Domain
     - Output Domain
     - Input/Output Metric
   * - :func:`opendp.transformations.make_resize`
     - ``VectorDomain<DA>``
     - ``SizedDomain<VectorDomain<DA>>``
     - ``SymmetricDistance/InsertDeleteDistance``


.. _aggregators:

Aggregators
-----------
Aggregators compute a summary statistic on individual-level data.

Aggregators that produce scalar-valued statistics have an output_metric of ``AbsoluteDistance[TO]``.
This output metric can be chained with most noise-addition measurements interchangeably.

However, aggregators that produce vector-valued statistics like :func:`opendp.transformations.make_count_by_categories`
provide the option to choose the output metric: ``L1Distance[TOA]`` or ``L2Distance[TOA]``.
These default to ``L1Distance[TOA]``, which chains with L1 noise mechanisms like :func:`opendp.measurements.make_laplace` and :func:`opendp.measurements.make_laplace`.
If you set the output metric to ``L2Distance[TOA]``, you can chain with L2 mechanisms like :func:`opendp.measurements.make_gaussian`.

The constructor :func:`opendp.transformations.make_count_by` does a similar aggregation as :func:`opendp.transformations.make_count_by_categories`,
but does not need a category set (you instead chain with :func:`opendp.measurements.make_base_laplace_threshold`).

The ``make_sized_bounded_covariance`` aggregator is Rust-only at this time because data loaders for data of type ``Vec<(T, T)>`` are not implemented.

See the notebooks for code examples and deeper explanations:

.. toctree::
   :glob:
   :titlesonly:

   aggregation-sum
   aggregation-mean


.. list-table::
   :header-rows: 1

   * - Aggregator
     - Input Domain
     - Output Domain
     - Input Metric
     - Output Metric
   * - :func:`opendp.transformations.make_count`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``AtomDomain<TO>``
     - ``SymmetricDistance``
     - ``AbsoluteDistance<TO>``
   * - :func:`opendp.transformations.make_count_distinct`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``AtomDomain<TO>``
     - ``SymmetricDistance``
     - ``AbsoluteDistance<TO>``
   * - :func:`opendp.transformations.make_count_by_categories`
     - ``VectorDomain<AtomDomain<TIA>>``
     - ``VectorDomain<AtomDomain<TOA>>``
     - ``SymmetricDistance``
     - ``L1Distance<TOA>/L2Distance<TOA>``
   * - :func:`opendp.transformations.make_count_by`
     - ``VectorDomain<AtomDomain<TI>>``
     - ``MapDomain<AtomDomain<TI>, AtomDomain<TO>>``
     - ``SymmetricDistance``
     - ``L1Distance<TO>``
   * - :func:`opendp.transformations.make_sum`
     - ``VectorDomain<AtomDomain<T>>`` (with bounds and optionally size)
     - ``AtomDomain<T>``
     - ``SymmetricDistance/InsertDeleteDistance``
     - ``AbsoluteDistance<TO>``
   * - :func:`opendp.transformations.make_mean`
     - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
     - ``AtomDomain<T>``
     - ``SymmetricDistance``
     - ``AbsoluteDistance<TO>``
   * - :func:`opendp.transformations.make_variance`
     - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
     - ``AtomDomain<T>``
     - ``SymmetricDistance``
     - ``AbsoluteDistance<TO>``
   * - make_sized_bounded_covariance (Rust only)
     - ``VectorDomain<AtomDomain<(T,T)>>`` (with size and bounds)
     - ``AtomDomain<T>``
     - ``SymmetricDistance``
     - ``AbsoluteDistance<TO>``


:func:`opendp.transformations.make_sum` makes a best guess as to which summation strategy to use based on the input space.
Should you need it, the following constructors give greater control over the sum.

.. dropdown:: Algorithm Details

  The following strategies are ordered by computational efficiency:

  * ``checked`` can be used when the dataset size multiplied by the bounds doesn't overflow.
  * ``monotonic`` can be used when the bounds share the same sign.
  * ``ordered`` can be used when the input metric is ``InsertDeleteDistance``.
  * ``split`` separately sums positive and negative numbers, and then adds these sums together.

  .. ``monotonic``, ``ordered`` and ``split`` are implemented with saturation arithmetic. 
  .. ``checked``, ``monotonic`` and ``split`` protect against underestimating sensitivity by preserving associativity.

  All four algorithms are valid for integers, but only ``checked`` and ``ordered`` are available for floats.
  There are separate constructors for integers and floats, because floats additionally need a dataset truncation and a slightly larger sensitivity.
  The increase in float sensitivity accounts for inexact floating-point arithmetic, and is calibrated according to the length of the mantissa and underlying summation algorithm. 

  Floating-point summation may be further configured to either ``Sequential<T>`` or ``Pairwise<T>`` (default).
  Sequential summation results in an ``O(n^2 / 2^k)`` increase in sensitivity, while pairwise summation results only in a ``O(log_2(n)n / 2^k))`` increase, 
  where ``k`` is the bit length of the mantissa in the floating-point numbers used.


  .. list-table::
    :header-rows: 1

    * - Aggregator
      - Input Domain
      - Input Metric
    * - :func:`opendp.transformations.make_sized_bounded_int_checked_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_bounded_int_monotonic_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_sized_bounded_int_monotonic_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_bounded_int_ordered_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with bounds)
      - ``InsertDeleteDistance``
    * - :func:`opendp.transformations.make_sized_bounded_int_ordered_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``InsertDeleteDistance``
    * - :func:`opendp.transformations.make_bounded_int_split_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_sized_bounded_int_split_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_bounded_float_checked_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_sized_bounded_float_checked_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``SymmetricDistance``
    * - :func:`opendp.transformations.make_bounded_float_ordered_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with bounds)
      - ``InsertDeleteDistance``
    * - :func:`opendp.transformations.make_sized_bounded_float_ordered_sum`
      - ``VectorDomain<AtomDomain<T>>`` (with size and bounds)
      - ``InsertDeleteDistance``
  

Quantiles via Trees
-------------------

Building off of the binning transformation, quantiles can be estimated by privatizing a b-ary tree and then postprocessing.
See the following notebook for more information:

.. toctree::
   :glob:
   :titlesonly:

   aggregation-quantile

These use :func:`opendp.transformations.make_b_ary_tree`, :func:`opendp.transformations.make_consistent_b_ary_tree` and :func:`opendp.transformations.make_quantiles_from_counts`.


Bring Your Own
--------------

Use :func:`opendp.transformations.make_user_transformation` to construct your own transformation.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. code:: python

        >>> import opendp.prelude as dp
        >>> dp.enable_features("honest-but-curious")

In this example, we mock the typical API of the OpenDP library to make a transformation that duplicates each record `multiplicity` times:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> from typing import List
        ...
        >>> def make_repeat(multiplicity):
        ...     """Constructs a Transformation that duplicates each record `multiplicity` times"""
        ...     def function(arg: List[int]) -> List[int]:
        ...         return arg * multiplicity
        ... 
        ...     def stability_map(d_in: int) -> int:
        ...         # if a user could influence at most `d_in` records before, 
        ...         # they can now influence `d_in` * `multiplicity` records
        ...         return d_in * multiplicity
        ...
        ...     return dp.t.make_user_transformation(
        ...         input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        ...         input_metric=dp.symmetric_distance(),
        ...         output_domain=dp.vector_domain(dp.atom_domain(T=int)),
        ...         output_metric=dp.symmetric_distance(),
        ...         function=function,
        ...         stability_map=stability_map,
        ...     )
    
The resulting Transformation may be used interchangeably with those constructed via the library:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> trans = (
        ...     (dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance())
        ...     >> dp.t.then_cast_default(TOA=int)
        ...     >> make_repeat(2)  # our custom transformation
        ...     >> dp.t.then_clamp((1, 2))
        ...     >> dp.t.then_sum()
        ...     >> dp.m.then_laplace(1.0)
        ... )
        ...
        >>> release = trans(["0", "1", "2", "3"])
        >>> trans.map(1) # computes epsilon
        4.0

The code snip may form a basis for you to create your own data transformations, 
and mix them into an OpenDP analysis.

You can also define your own measurements (:func:`opendp.measurements.make_user_measurement`) and postprocessors (:func:`opendp.core.new_function`).
