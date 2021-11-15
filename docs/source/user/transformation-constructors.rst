.. _transformation-constructors:

Transformation Constructors
===========================

This section gives a high-level overview of the transformations that are available in the library.
Refer to the :ref:`transformation` section for an explanation of what a transformation is.

As covered in the :ref:`chaining` section, the intermediate :ref:`domains <domains>` need to match when chaining.
Each transformation has a carefully chosen input domain and output domain that supports their relation.

Preprocessing is the series of transformations that shape the data into a domain that is conformable with the aggregator.

You will need to choose the proper transformations from the sections below in order to chain with the aggregator you intend to use.
The sections below are in the order you would typically chain transformations together,
but you may want to peek at the aggregator section at the end first,
to identify the input domain that you'll need to preprocess to.

Dataframe
---------
These transformations are for loading data into a dataframe and retrieving columns from a dataframe.
If you just want to load data from a CSV or TSV into a dataframe,
you'll probably want to use :func:`opendp.trans.make_split_dataframe`.

Use :func:`opendp.trans.make_select_column` to retrieve a column from the dataframe.

The other dataframe transformations are more situational.

Be warned that it is not currently possible to directly load and unload dataframes from the library in bindings languages!
You need to chain with ``make_select_column`` first.

.. list-table::
   :header-rows: 1

   * - Preprocessor
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_split_dataframe`
     - ``AllDomain<String>``
     - ``DataFrameDomain<K>``
   * - :func:`opendp.trans.make_select_column`
     - ``DataFrameDomain<K>``
     - ``VectorDomain<AllDomain<TOA>>``
   * - :func:`opendp.trans.make_split_lines`
     - ``AllDomain<String>``
     - ``VectorDomain<AllDomain<String>>``
   * - :func:`opendp.trans.make_split_records`
     - ``VectorDomain<AllDomain<String>>``
     - ``VectorDomain<VectorDomain<AllDomain<String>>>``
   * - :func:`opendp.trans.make_create_dataframe`
     - ``VectorDomain<VectorDomain<AllDomain<String>>>``
     - ``DataFrameDomain<K>``

Casting
-------
Any time you want to convert between data types, you'll want to use a casting transformation.
In particular, in pipelines that load dataframes from CSV files, it is very common to cast from Strings to some other type.

Depending on the caster you choose, the output data may be null and you will be required to chain with an imputer.

There is an unusual caster in this section that allows you to cast from a ``SubstituteDistance`` metric to a ``SymmetricDistance`` metric.
It is noted in the `linked dev issue <https://github.com/opendp/opendp/issues/156#issuecomment-867900184>`_ that most transformations need only be defined for ``SymmetricDistance``.
If you are more comfortable working with substitution distances,
you can chain this caster at the start of a computation pipeline to make ``d_in`` a substitution distance instead of a symmetric distance.

.. list-table::
   :header-rows: 1

   * - Caster
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_cast`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<OptionNullDomain<AllDomain<TOA>>>``
   * - :func:`opendp.trans.make_cast_default`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<AllDomain<TOA>>``
   * - :func:`opendp.trans.make_cast_inherent`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<InherentNullDomain<AllDomain<TOA>>>``
   * - :func:`opendp.trans.make_is_equal`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<AllDomain<bool>>``
   * - :func:`opendp.trans.make_is_null`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<AllDomain<bool>>``
   * - :func:`opendp.trans.make_cast_metric`
     - ``VectorDomain<AllDomain<TA>>``
     - ``VectorDomain<AllDomain<TA>>``


Imputation
----------

Null values are tricky to handle in a differentially private manner.
If we were to allow aggregations to propagate null,
then aggregations provide a non-differentially-private bit revealing the existence of nullity in the dataset.
If we were to implicitly drop nulls from sized aggregations, then the sensitivity of non-null individuals is underestimated.
Therefore, aggregators must be fed completely non-null data.
We can ensure data is non-null by imputing.

When you cast with :func:`opendp.trans.make_cast` or :func:`opendp.trans.make_cast_default`,
the cast may fail, so the output domain may include null values (``OptionNullDomain`` and ``InherentNullDomain``).
We have provided imputation transformations to transform the data domain to the non-null ``VectorDomain<AllDomain<TA>>``.

You may also be in a situation where you want to bypass dataframe loading and casting
because you already have a vector of floats loaded into memory.
In this case, you should start your chain with an imputer if the floats are potentially null.

:OptionNullDomain: A representation of nulls using an Option type (``Option<bool>``, ``Option<i32>``, etc).
:InherentNullDomain: A representation of nulls using the data type itself (``f32`` and ``f64``).

The :func:`opendp.trans.make_impute_constant` transformation supports imputing on either of these representations of nullity,
so long as you pass the DA (atomic domain) type argument.

.. list-table::
   :header-rows: 1

   * - Imputer
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_impute_constant`
     - ``VectorDomain<OptionNullDomain<AllDomain<TA>>>``
     - ``VectorDomain<AllDomain<TA>>``
   * - :func:`opendp.trans.make_impute_constant`
     - ``VectorDomain<InherentNullDomain<AllDomain<TA>>>``
     - ``VectorDomain<AllDomain<TA>>``
   * - :func:`opendp.trans.make_impute_uniform_float`
     - ``VectorDomain<InherentNullDomain<AllDomain<TA>>>``
     - ``VectorDomain<AllDomain<TA>>``
   * - :func:`opendp.trans.make_drop_null`
     - ``VectorDomain<OptionNullDomain<AllDomain<TA>>>``
     - ``VectorDomain<AllDomain<TA>>``
   * - :func:`opendp.trans.make_drop_null`
     - ``VectorDomain<InherentNullDomain<AllDomain<TA>>>``
     - ``VectorDomain<AllDomain<TA>>``

Indexing
--------
Indexing operations provide a way to relabel categorical data, or bin numeric data into categorical data.
These operations work with `usize` data types: an integral data type representing an index.
:func:`opendp.trans.make_find` finds the index of each input datum in a set of categories.
In other words, it transforms a categorical data vector to a vector of numeric indices.

.. testsetup::

    from opendp.trans import make_find, make_impute_constant, make_find_bin, make_index
    from opendp.typing import *
    from opendp.mod import enable_features
    enable_features('contrib')

.. doctest::

    >>> finder = (
    ...     make_find(categories=["A", "B", "C"]) >>
    ...     # impute any input datum that are not a part of the categories list as 3
    ...     make_impute_constant(3, DA=OptionNullDomain[AllDomain["usize"]])
    ... )
    >>> finder(["A", "B", "C", "A", "D"])
    [0, 1, 2, 0, 3]

:func:`opendp.trans.make_find_bin` is a binning operation that transforms numerical input data to a vector of bin indices.

.. doctest::

    >>> binner = make_find_bin(edges=[1., 2., 10.])
    >>> binner([0., 1., 3., 15.])
    [0, 1, 2, 3]

:func:`opendp.trans.make_index` uses each indicial input datum as an index into a category set.

.. doctest::

    >>> indexer = make_index(categories=["A", "B", "C"], null="D")
    >>> indexer([0, 1, 2, 3, 2342])
    ['A', 'B', 'C', 'D', 'D']

You can use combinations of the indicial transformers to map hashable data to integers, bin numeric types, relabel hashable types, and label bins.

.. list-table::
   :header-rows: 1

   * - Indexer
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_find`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<OptionNullDomain<AllDomain<usize>>>``
   * - :func:`opendp.trans.make_find_bin`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``VectorDomain<AllDomain<usize>>``
   * - :func:`opendp.trans.make_index`
     - ``VectorDomain<AllDomain<usize>>``
     - ``VectorDomain<AllDomain<TOA>>``

Clamping
--------
Many aggregators depend on bounded data to limit the influence that perturbing an individual may have on a query.
For example, the relation downstream for the :func:`opendp.trans.make_bounded_sum` aggregator is ``d_out >= d_in * max(|L|, |U|)``.
This relation states that adding or removing ``d_in`` records may influence the sum by ``d_in`` * the greatest magnitude of a record.

Any aggregator that needs bounded data will indicate it in the function name.
In these kinds of aggregators the relations make use of the clamping bounds ``L`` and ``U`` to translate ``d_in`` to ``d_out``.

Clamping happens after casting and imputation but before resizing.
Only chain with a clamp transformation if the aggregator you intend to use needs bounded data.

.. list-table::
   :header-rows: 1

   * - Clamper
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_clamp`
     - ``VectorDomain<AllDomain<TA>>``
     - ``VectorDomain<BoundedDomain<TA>>``
   * - :func:`opendp.trans.make_unclamp`
     - ``VectorDomain<BoundedDomain<TA>>``
     - ``VectorDomain<AllDomain<TA>>``


Resizing
--------
Similarly to data bounds, many aggregators depend on a known dataset size in their relation as well.
For example, the relation downstream for the :func:`opendp.trans.make_sized_bounded_mean` aggregator is ``d_out >= d_in * (U - L) / n / 2``.
Notice that any addition and removal may, in the worst case, change a record from ``L`` to ``U``.
Such a substitution would influence the mean by ``(U - L) / n``.

Any aggregator that needs sized data will indicate it in the function name.
In these kinds of aggregators, the relations need knowledge about the dataset size ``n`` to translate ``d_in`` to ``d_out``.

Resizing happens after clamping.
Only chain with a resize transformation if the aggregator you intend to use needs sized data.

At this time, there are two separate resize transforms:
one that works on unbounded data, and one that works on bounded data.
We intend to merge these in the future.

.. list-table::
   :header-rows: 1

   * - Resizer
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_resize`
     - ``VectorDomain<AllDomain<TA>>``
     - ``VectorDomain<AllDomain<TA>>``
   * - :func:`opendp.trans.make_bounded_resize`
     - ``VectorDomain<BoundedDomain<TA>>``
     - ``VectorDomain<BoundedDomain<TA>>``


.. _aggregators:

Aggregators
-----------
Aggregators compute a summary statistic on individual-level data.

Aggregators that produce scalar-valued statistics have a output_metric of ``AbsoluteDistance[TO]``.
This output metric can be chained with most noise-addition measurements interchangeably.

However, aggregators that produce vector-valued statistics like :func:`opendp.trans.make_count_by_categories`
provide the option to choose the output metric: ``L1Distance[TOA]`` or ``L2Distance[TOA]``.
These default to ``L1Distance[TOA]``, which chains with L1 noise mechanisms like :func:`opendp.meas.make_base_geometric` and :func:`opendp.meas.make_base_laplace`.
If you set the output metric to ``L2Distance[TOA]``, you can chain with L2 mechanisms like :func:`opendp.meas.make_base_gaussian`.

The constructor :func:`opendp.meas.make_count_by` does a similar aggregation as :func:`opendp.trans.make_count_by_categories <make_count_by_categories>`,
but does not need a category set (you instead chain with :func:`opendp.meas.make_base_ptr`, which uses the propose-test-release framework).

The ``make_sized_bounded_covariance`` aggregator is Rust-only at this time because data loaders for data of type ``Vec<(T, T)>`` are not implemented.

.. list-table::
   :header-rows: 1

   * - Aggregator
     - Input Domain
     - Output Domain
   * - :func:`opendp.trans.make_count`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``AllDomain<TO>``
   * - :func:`opendp.trans.make_count_distinct`
     - ``VectorDomain<AllDomain<TIA>>``
     - ``AllDomain<TO>``
   * - :func:`opendp.trans.make_count_by_categories`
     - ``VectorDomain<BoundedDomain<TIA>>``
     - ``VectorDomain<AllDomain<TOA>>``
   * - :func:`opendp.meas.make_count_by`
     - ``VectorDomain<BoundedDomain<TI>>``
     - ``MapDomain<AllDomain<TI>,AllDomain<TO>>``
   * - :func:`opendp.trans.make_bounded_sum`
     - ``VectorDomain<BoundedDomain<T>>``
     - ``AllDomain<T>``
   * - :func:`opendp.trans.make_sized_bounded_sum`
     - ``SizedDomain<VectorDomain<BoundedDomain<T>>>``
     - ``AllDomain<T>``
   * - :func:`opendp.trans.make_sized_bounded_mean`
     - ``SizedDomain<VectorDomain<BoundedDomain<T>>>``
     - ``AllDomain<T>``
   * - :func:`opendp.trans.make_sized_bounded_variance`
     - ``SizedDomain<VectorDomain<BoundedDomain<T>>>``
     - ``AllDomain<T>``
   * - make_sized_bounded_covariance (Rust only)
     - ``SizedDomain<VectorDomain<BoundedDomain<(T,T)>>>``
     - ``AllDomain<T>``
