OpenDP Rust Initiation
======================

This notebook is an introduction to the Rust internals of the OpenDP
framework. I’m assuming you have already read about the programming
framework in the user guide, and you have some familiarity with the
library interfaces.

If you have not worked with Rust before, `a great place to get started
is the Rust
book <https://doc.rust-lang.org/stable/book/ch01-00-getting-started.html>`__.
This notebook will also reference sections of the Rust book that surface
commonly in the OpenDP library.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # we'll use this Python snip to demonstrate concepts later...
            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            
            >>> input_domain = dp.vector_domain(dp.atom_domain(T=str))
            >>> input_metric = dp.symmetric_distance()
            
            >>> default_cast_trans = dp.t.make_cast_default(input_domain, input_metric, TOA=int)
            

Transformation Structure
~~~~~~~~~~~~~~~~~~~~~~~~

The following snip is the definition of a Transformation, from
`rust/src/core/mod.rs <https://github.com/opendp/opendp/blob/main/rust/src/core/mod.rs>`__.
Transformations are structs (`see chapter
5 <https://doc.rust-lang.org/stable/book/ch05-00-structs.html>`__).

.. code:: rust


   struct Transformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
       pub input_domain: DI,
       pub output_domain: DO,
       pub function: Function<DI, DO>,
       pub input_metric: MI,
       pub output_metric: MO,
       pub stability_relation: StabilityRelation<MI, MO>,
   }

This struct has four generics (`see chapter
10.1 <https://doc.rust-lang.org/stable/book/ch10-00-generics.html>`__):

::

   - DI for input domain
   - DO for output domain
   - MI for input metric
   - MO for output metric

A generic is a type that has not yet been explicitly determined. These
generics let us build transformations out of many different types.

Notice that each of these generics are marked as either ``Domain`` or
``Metric``. These are called “trait bounds” (`see chapter
10.2 <https://doc.rust-lang.org/stable/book/ch10-02-traits.html#trait-bound-syntax>`__).

``Domain`` and ``Metric`` are both traits. Trait bounds constrain the
set of possible types that a generic may take on. In this case,
``DI: Domain`` indicates that ``DI`` may be any type that has the
``Domain`` trait implemented for it. There is a reasonably small set of
types that satisfy these trait bounds.

We now take a closer look at each of the struct members.

.. code:: rust

       ...
       pub input_domain: DI,
       pub output_domain: DO,
       ...

The input and output domains strictly define the set of permissible
input and output values. Examples of metrics are ``AtomDomain``,
``VectorDomain``, ``MapDomain`` and ``DataFrameDomain``. When you
attempt to chain any two transformations, the output domain of the first
transformation must match the input domain of the second transformation
(`via the PartialEq
trait <https://doc.rust-lang.org/std/cmp/trait.PartialEq.html>`__). The
resulting chained transformation contains the input domain of the first
transformation, the output domain of the second transformation, as well
as the functional composition of the two functions.

.. code:: rust

       ...
       pub function: Function<DI, DO>,
       ...

We wrap the closure in a ``Function`` struct that is generic over the
input domain and output domain. The definition of this struct `is in the
same
file <https://github.com/opendp/opendp/blob/main/rust/src/core/mod.rs>`__.

When we invoke the following transformation:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> default_cast_trans(["null", "1.", "2", "456"])
            [0, 0, 2, 456]

1. the Python data structure is translated into a low-level C
   representation and then into a Rust representation
2. the Rust ``function`` is evaluated on a Rust ``Vec<String>``
3. the result is shipped back out to familiar Python data structures

We also have input and output metrics.

.. code:: rust

       ...
       pub input_metric: MI,
       pub output_metric: MO,
       ...

Examples of metrics are ``HammingDistance``, ``SymmetricDistance``,
``AbsoluteDistance`` and ``L1Distance``. They behave in the same way
that the input and output domains do when chaining. Finally, the
stability map.

.. code:: rust

       ...
       pub stability_map: StabilityMap<MI, MO>,
       ...

It is a function that takes in an input distance, in the respective
metric space, and returns the smallest acceptable output distance in
terms of the output metric. The definition of this struct `is also in
the same
file <https://github.com/opendp/opendp/blob/main/rust/src/core/mod.rs>`__.

Invoking this function triggers a similar process as the function did:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> default_cast_trans.map(d_in=3)
            3

When any two compatible transformations are chained, the resulting
transformation contains a functional composition of the relations.

Ultimately, all pieces are used to construct the new transformation:

+----------------------+----------------------+----------------------+
| input                | chaining             | output               |
+======================+======================+======================+
| input_domain_1       | output_domain_1 ==   | output_domain_2      |
|                      | input_domain_2       |                      |
+----------------------+----------------------+----------------------+
| function_1           | composed with        | function_2           |
+----------------------+----------------------+----------------------+
| input_metric_1       | output_metric_1 ==   | output_metric_2      |
|                      | input_metric_2       |                      |
+----------------------+----------------------+----------------------+
| stability_relation_1 | composed with        | stability_relation_2 |
+----------------------+----------------------+----------------------+

As you’ve seen above, when we want to create a transformation, we use
“constructor” functions. These are, by convention, prefixed with
``make_``.

Example Transformation Constructor
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

An example implementation of the casting transformation constructor is
provided. I’ll break it down into three parts.

.. code:: rust

   // 1.
   pub fn make_cast_default<TIA, TOA, M>(
       input_domain: VectorDomain<AtomDomain<TIA>>,
       input_metric: M
   )
       -> Fallible<
           Transformation<
               VectorDomain<AtomDomain<TIA>>, 
               VectorDomain<AtomDomain<TOA>>, 
               M, 
               M>>

       // 2.
       where TIA: 'static + Clone + CheckNull, 
             TOA: 'static + RoundCast<TIA> + Default + CheckNull,
             M: DatasetMetric,
             (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
             (VectorDomain<AtomDomain<TOA>>, M): MetricSpace, {

       // 3.
       Transformation::new(
           input_domain.clone(),
           VectorDomain::new(AtomDomain::default(), input_domain.size),
           Function::new(move |arg: &Vec<TIA>|
               arg.iter().map(|v| TOA::round_cast(v.clone()).unwrap_or_default()).collect()),
           input_metric.clone(),
           input_metric,
           StabilityRelation::new_from_constant(1))
   }

The first part is the function signature:

.. code:: rust

   pub fn make_cast_default<TIA, TOA, M>(
       input_domain: VectorDomain<AtomDomain<TIA>>,
       input_metric: M
   )
       -> Fallible<
           Transformation<
               VectorDomain<AtomDomain<TIA>>, 
               VectorDomain<AtomDomain<TOA>>, 
               M, 
               M>>
       ...

Most of the signature consists of types. Rust is strictly typed, so the
code needs to be very explicit about what the type of the constructor
function’s inputs and outputs are.

This is a generic function with two type arguments ``TIA`` and ``TOA``,
standing for “atomic input type” and “atomic output type”, and one type
argument ``M``, standing for the type of the metric.

The function takes two concrete arguments, the ``input_domain`` and
``input_metric``. The types of these arguments are shown after the colon
``:``.

The constructor returns a fallible transformation. The last four lines
specify the types of the input/output domains/metrics, that is, what
``DI``, ``DO``, ``MI`` and ``MO`` (from the definition of a
Transformation) are.

The second part is the where clause:

.. code:: rust

       ...
       where TIA: 'static + Clone + CheckNull, 
           TOA: 'static + RoundCast<TIA> + Default + CheckNull,
           M: DatasetMetric,
           (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
           (VectorDomain<AtomDomain<TOA>>, M): MetricSpace, {
       ...

A where clause is another, equivalent way of listing trait bounds on
generics. You can interpret this as, “the compiler will enforce that
``TIA`` must be some type that has the ``Clone`` and ``CheckNull``
traits. In other words, while I don’t specify what ``TIA`` must be
up-front, I can bound what type it may be to types that are cloneable
and have some concept of null-checking. ``TOA``, in particular, has a
``RoundCast`` trait, which can be used to cast from type ``TIA`` to
``TOA``. For now, please feel free to ignore the ``'static`` trait
bounds.

We also restrict the set of valid types that ``M`` may take on to only
those which the ``DatasetMetric`` trait has been implemented:
``SymmetricDistance``, ``InsertDeleteDistance``, ``ChangeOneDistance``
and ``HammingDistance``. Finally, there is a trait bound specifying that
the input domain and input metric must, together, form a metric space,
and similarly for the output supporting elements.

The final part is the function body, which creates and implicitly
returns a Transformation struct.

.. code:: rust

       ...
       Transformation::new(
           input_domain.clone(),
           VectorDomain::new(AtomDomain::default(), input_domain.size),
           Function::new(move |arg: &Vec<TIA>|
               arg.iter().map(|v| TOA::round_cast(v.clone()).unwrap_or_default()).collect()),
           input_metric.clone(),
           input_metric,
           StabilityRelation::new_from_constant(1))
   }

Each argument corresponds to a struct member. To make the ``Function``,
we use a useful shorthand to create an anonymous closure (a function)
(`see chapter
13.1 <https://doc.rust-lang.org/stable/book/ch13-01-closures.html>`__).
For example, ``|a, b| a + b``. takes two arguments, ``a`` and ``b``. The
function body is ``a + b``.

This closure casts the data by iterating over each record ``v``,
casting, and replacing nulls with the default value for the type (`see
chapter
13.2 <https://doc.rust-lang.org/stable/book/ch13-02-iterators.html>`__).

We also take advantage of a convenient constructor for building
``c``-stable relations. Since the cast function is row-by-row, it is
1-stable.

Measurement Structure
~~~~~~~~~~~~~~~~~~~~~

Measurements are very similar to Transformations, with two key
differences.

.. code:: rust

   pub struct Measurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
       pub input_domain: DI,
       pub function: Function<DI, DO>,
       pub input_metric: MI,
       pub output_measure: MO,
       pub privacy_map: PrivacyMap<MI, MO>,
   }

First, the ``output_metric`` is replaced with an ``output_measure``, as
distances in the output space are measured in terms of divergences
between probability distributions.

Second, the name of the map has changed from a stability map to a
privacy map. This is because the relation between distances now carries
meaning with respect to privacy.

Developer Loop
~~~~~~~~~~~~~~

When writing code:

1. Make a change to the Rust source.
2. Use ``cargo check --all-features`` to do a quick check for compiler
   errors. A properly configured development environment will
   automatically run this command for you and highlight your code.
3. Read the compiler errors and iterate. Rust errors usually provide
   helpful explanations.

When testing code in Rust, a properly configured development environment
will mark up ``#[test]`` annotations with a button to execute the test.

When testing code in Python, run ``cargo build --all-features`` to
update the binary. You’ll need to restart the Python interpreter or
kernel for changes to appear. All folders named ``out`` are .gitignored,
so they’re a great place to throw scratch work that you don’t want to
commit.

If you are writing a new function, you’ll need to write FFI bindings
(``./ffi.rs``) and decorate the function with the ``bootstrap`` macro
before you can access the function from Python. Please don’t hesitate to
ask for help!

Next Steps
~~~~~~~~~~

1. If you are adding a new file, please place your code inside a
   ``mod.rs`` file in a new folder. This is to give room to place the
   proof file adjacent to the implementation.
2. Please accompany your sources with a testing module at the end of the
   file. Test modules are also a great way to play with your constructor
   before the FFI bindings are available.
3. Please format your code nicely (rustfmt), add documentation, and
   comment meaningfully!

The other constructor functions in the library are great to use as a
reference. It’s likely you have more questions — this short guide could
never possibly be complete. If you’d like to get more involved in OpenDP
development, don’t hesitate to send a message and we’ll help get you
bootstrapped!
