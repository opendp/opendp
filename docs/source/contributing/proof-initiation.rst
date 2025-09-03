OpenDP Proof Initiation
=======================

This notebook is an introduction to writing proofs in the OpenDP
framework. It assumes you have already read about the :ref:`programming-framework`,
and you have some familiarity with the library interfaces.

Our goal is to prove that OpenDP produces stable transformations and
private measurements. This requires a large body of proofs, as
constructor functions typically depend on other functions or traits that
themselves need to be proven.

Proof Structure
---------------

Each document should at least have these components:

1. Hoare Triple

   1. Precondition
   2. Pseudocode
   3. Postcondition

2. Proof of postcondition, assuming precondition

The precondition is a predicate function that captures any restrictions
on the set of valid inputs to the function. The (Python-esque)
pseudocode may exploit the restrictions on the inputs provided by the
precondition and make use of other functions or traits. The
postcondition is also a predicate function that sharply characterizes
the guarantees the function makes on the output.

This gets more involved for measurement and transformation constructor
functions. Constructor functions need to argue the correctness of the
stability map or privacy map, which refers to the behavior on a pair of
inputs. A LaTex macro is mentioned below that generates the expected
postconditions for transformation and measurement constructors.

The proof should show that, for any setting of the input arguments for
which the precondition is true, the postcondition is also true.

Function Example
----------------

Let’s imagine we wanted to add a new function to
``rust/src/metrics/mod.rs`` that computes the absolute distance:

.. code:: rust

   /// # Proof Definition
   /// For any setting of the input parameters, 
   /// returns either "Ok(out)" where $out \ge |a - b|$, or "Err(e)".
   fn absolute_distance<T: InfSub + AlertingAbs>(a: T, b: T) -> Fallible<T> {
       a.inf_sub(&b)?.alerting_abs()
   }

``inf_sub`` is provided by the ``InfSub`` trait, and it computes the
subtraction of ``b`` from ``a``. By the proof definition of ``InfSub``,
if the outcome is not exactly representable in ``T``, then the outcome
is rounded towards positive infinity, to the nearest value in ``T``
(think of when ``T`` is float and arithmetic is not closed).

Again by the proof definition of ``InfSub``, ``inf_sub`` returns an
error if the computation overflows. The type of things that *might* be
an error is ``Fallible<T>``, which is an enum containing either
``Ok(value)``, where ``value`` is of type ``T``, or ``Err(e)``, where
``e`` is the error details (backtrace, message, debug info).

Both ``inf_sub`` and ``alerting_abs`` return a value of type
``Fallible<T>``. After calling ``inf_sub``, the question mark exits the
function early with the error, if there is one, otherwise execution
continues to ``alerting_abs`` with the value of type ``T`` stored inside
the ``Ok`` variant.

Notice that the function expects a return of type ``Fallible<T>``. This
is consistent with the two return sites of the function: the question
mark after ``inf_sub``, and the implicit return of the outcome of
``alerting_abs``.

When reading code in Rust for proof purposes, you can generally ignore
references (``&``), dereferences (``*``, not multiplication) and clones
(``.clone()``). We consider these aspects of the code proven by the Rust
compiler.

LaTex Template
~~~~~~~~~~~~~~

To prove ``fn absolute_distance``, we add a new file adjacent to the
function we want to prove, named after the function we want to prove:
``rust/src/metrics/absolute_distance.tex``.

Here’s a template for ``absolute_distance.tex``:

.. code:: latex

   \documentclass{article} % necessary for Overleaf to recognize the file
   \input{../lib.sty} % "rust/src/lib.sty" contains boilerplate and macros

   \title{\texttt{fn absolute\_distance}}
   \author{Your Name(s) Here}\date{}

   \begin{document}
   \maketitle\contrib
   Proves soundness of \rustdoc{transformations/fn}{absolute\_distance} in \asOfCommit{mod.rs}{f5bb719}.

   \subsection*{Vetting history}
   \begin{itemize}
       \item \vettingPR{519}
   \end{itemize}

   \section{Hoare Triple}
   \subsection*{Preconditions}
   \subsection*{Pseudocode}
   \subsection*{Postcondition}

   \section{Proof}

   \end{document}

This template uses several macros defined in ``rust/src/lib.sty``:

-  ``\contrib`` adds a header to the document indicating the proof is in
   ``"contrib"``.

-  ``\rustdoc{path/to/fn}{ident}`` creates a link to the rust
   documentation for the function we are proving. When you build the
   document, it should emit a link to the latest build of the docs on
   docs.rs, `which may not exist
   yet <https://docs.rs/opendp/latest/opendp/transformations/fn.absolute_distance.html>`__.
   When we cut a release, the ``docs.rs`` site is updated, and the links
   in your document will be fixed to the released version of OpenDP. The
   first argument to the macro is the subset of the path after
   ``opendp/``, up to the dot, and the second argument is the identifier
   name.

-  ``\asOfCommit{relative/path}{commit_hash}`` is how you specify which
   file you are proving, and the commit hash that last edited the file
   you are proving. You can retrieve the hash with
   ``git log -n 1 --pretty=format:%h -- path/to/file.rs``. If you are
   proofwriting within the git repository, you can also find this hash
   in the footnote. The resulting LaTex output is a permalink to the
   file and an indicator on if the file has been updated since. This
   makes is possible for proof documents to self-report when they go
   out-of-date.

-  ``\vettingPR{PR_number}`` is a simple macro to link a specific pull
   request.

-  ``\docsrs{crate}{path/to/fn}{ident}`` is not used in this template.
   It has a similar syntax to ``\rustdoc``, but with an extra leading
   argument to name a crate. It builds links to documentation in
   external crates on `docs.rs <https://docs.rs>`__.

-  ``\validTransformation{input_arguments}{function_name}`` is not used
   in this template, but is useful when writing a proof for a
   transformation constructor.

-  ``\validMeasurement{input_arguments}{function_name}`` same as above,
   but for measurements.

These macros are written such that your document will still compile
without ``--shell-escape`` enabled.

You can build this template with:

.. code:: shell

   pdflatex --synctex=1 --interaction=nonstopmode --file-line-error --aux-directory=out --output-directory=out --shell-escape absolute_distance.tex

If you use VSCode, the “Development Environment” documentation contains
some advice for integrating this with the LaTex-Workshop extension.

These options emit the build artifacts to ./out, which is configured to
be ignored by git. **This is intentional, you should only include the** 
``.tex`` **file when committing to OpenDP!** A bot will attempt to build
and link generated ``.pdf`` files from your PR.

We now continue by filling out the proof sections.

Preconditions
~~~~~~~~~~~~~

The function we are proving has three input parameters, consisting of
one generic (``T``) and two arguments (``a`` and ``b``), where the
arguments ``a`` and ``b`` are of type ``T``. One may call this function
with 32-bit signed integer arguments (``i32``):

.. code:: rust

   absolute_distance(1i32, 2i32)

In this case, the setting of the input parameters is
``(T=i32, a=1, b=2)``. The setting of ``T`` is inferred from the types
of the arguments.

The Rust syntax ``T: InfSub + AlertingAbs`` indicates that ``T`` is any
type for which the ``InfSub`` and ``AlertingAbs`` traits are
implemented. Thus, other valid types for ``T`` include the
single-precision float ``f32``, unsigned 32-bit integer ``u32``, as well
as other floats and integers with different bit depths. This may also
extend to fixed-point types, bignum integers and rationals.

Rust will only compile this code if the ``InfSub`` and ``AlertingAbs``
traits have been implemented for ``T``. These bounds on the type ``T``
become part of the precondition for the function.

.. code:: latex

   \subsection*{Preconditions}
   \begin{itemize}
       \item \texttt{T} is a type with traits \rustdoc{traits/trait}{InfSub} and \rustdoc{traits/trait}{AlertingAbs}.
   \end{itemize}

In other contexts, it may make sense to specify preconditions on the
arguments as well.

Pseudocode
~~~~~~~~~~

The pseudocode should mimic the logic and usage of traits in the actual
rust code. The pseudocode isn’t strictly-defined, it is a tool to
communicate the algorithm in a way that is more accessible than Rust.

.. code:: latex

   \section{Pseudocode}
   \begin{lstlisting}[language = Python, escapechar=|]
   def absolute_distance(a, b):
       a.inf_sub(b).alerting_abs() |\label{line:out}|
   \end{lstlisting}

This code snip leverages the preconditions to make use of the
``inf_sub`` method on ``a``.

Postcondition
~~~~~~~~~~~~~

The postcondition is essentially the same as the proof definition on the
Rust code.

.. code:: latex

   For any setting of the input parameters for which the precondition holds, \texttt{absolute_distance} returns either \texttt{Ok(out)} where $out \ge |a - b|$, or \texttt{Err(e)}.

:math:`|a - b|` denotes an idealized quantity computed with infinite
precision.

Our goal is to use the pseudocode to prove that the postcondition is
always true when the precondition is true.

Proof
~~~~~

Start by assuming the preconditions are met!

.. code:: latex

   \section{Proof}
   Assume the preconditions are met.

In order to use the properties guaranteed in the proof definition of
another function or trait, you must first prove that their preconditions
hold. ``InfSub`` and ``AlertingAbs`` don’t have any preconditions.

.. code:: latex

   The preconditions for \texttt{InfSub} and \texttt{AlertingAbs} are trivially met.

We now use these definitions to prove the postcondition:

.. code:: latex

   \begin{align*}
       \texttt{out} 
       &= a.inf_sub(b).alerting_abs() \\
       &= max(a.inf_sub(b), -a.inf_sub(b)) && \text{by \texttt{AlertingAbs}} \\
       &\ge max(a - b, -a.inf_sub(b)) && \text{by \texttt{InfSub}} \\
   \end{align*}

At this point, we get stuck. We can’t show the inequality we expected
because the code has a bug!

If the sign of the difference is negative, the round towards infinity is
a round towards zero, resulting in a smaller absolute distance than the
idealized absolute distance. This breaks the guarantee in our proof
definition. A bug like this could be abused by an adversary with a
sensitivity amplification widget; by carefully choosing constants that
exploit the gaps between floating-point numbers with large magnitudes.

This is why it is important to write proofs— it is easy to miss a detail
that can break privacy.
