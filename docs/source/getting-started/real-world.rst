Real World
==========

A differentially private analysis in OpenDP breaks down into the following steps:

1. Identify the unit of privacy
2. Set privacy loss parameters
3. Collect public information
4. Mediate access to data
5. Submit DP queries

OpenDP has two APIs and we'll demonstrate below how to use them both:

* The *Context API* is simpler and helps to enforce best practices; Available currently only for Python.
* The *Framework API* is lower-level; Available for Python and R, it mirrors the underlying Rust framework.

Because the Context API is a wrapper around the Framework API, it is easier to use but less flexible: All calls ultimately pass through the Framework API.

We'll illustrate these steps by doing a differentially-private analysis on a teacher survey (a tabular data set). The raw data consists of survey responses from teachers in primary and secondary schools in an unspecified U.S. state.

Context API
-----------

1. Identify the Unit of Privacy
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The first step in a differentially private analysis is to determine what you are protecting: the unit of privacy.

Releases on the teacher survey should conceal the addition or removal of any one teacher's data, and each teacher contributes at most one row to the data set, so the unit of privacy corresponds to one row contribution.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> import opendp.prelude as dp
            >>>
            >>> # we are also using library features that are marked "contrib":
            >>> dp.enable_features("contrib")
            >>>
            >>> privacy_unit = dp.unit_of(contributions=1)

Broadly speaking, differential privacy can be applied to any medium of data for which you can define a unit of privacy. In other contexts, the unit of privacy may correspond to multiple rows, a user ID, or nodes or edges in a graph.

The unit of privacy may also be more general or more precise than a single individual.

* *more general*: unit of privacy is an entire household, or a company
* *more precise*: unit of privacy is a person-month, or device

It is highly recommended to choose a unit of privacy that is at least as general as an individual.

2. Set Privacy Loss Parameters
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Next, you should determine what level of privacy protection to provide to your units of privacy. This choice may be governed by a variety of factors, such as the amount of harm that individuals could experience if their data were revealed, and your ethical and legal obligations as a data custodian.

The level of privacy afforded to units of privacy in a data set is quantified by *privacy loss parameters*. Under *pure* differential privacy, there is a single privacy-loss parameter, typically denoted epsilon (ε). Epsilon is a non-negative number, where larger values afford less privacy. Epsilon can be viewed as a proxy for the worst-case risk to a unit of privacy. It is customary to refer to a data release with such bounded risk as epsilon-differentially private (ε-DP).

A common rule-of-thumb is to limit ε to 1.0, but this limit will vary depending on the considerations mentioned above. See `Hsu et. al<https://arxiv.org/abs/1402.3329>`_ for a more elaborate discussion on setting epsilon.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> privacy_loss = dp.loss_of(epsilon=1.)

3. Collect Public Information
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The next step is to identify public information about the data set.

* information that is invariant across all potential input data sets (may include column names and per-column categories)
* information that is publicly available from other sources
* information from other DP releases

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> col_names = [
            ...    "name", "sex", "age", "maritalStatus", "hasChildren", "highestEducationLevel", 
            ...    "sourceOfStress", "smoker", "optimism", "lifeSatisfaction", "selfEsteem"
            ... ]

In this case (and in most cases), we consider column names public/invariant to the data because they weren't picked in response to the data, they were "fixed" before collecting the data.

A data invariant is information about your data set that you are explicitly choosing not to protect, typically under the basis that it is already public or that it does not contain sensitive information. Be careful because, if an invariant does, indeed, contain sensitive information, then you risk violating the privacy of individuals in your data set.

On the other hand, using public information significantly improves the utility of your results.

4. Mediate Access to Data
^^^^^^^^^^^^^^^^^^^^^^^^^

At this point, you ideally still haven't looked at the sensitive data set. This is the first and only point where we access the sensitive data set in this process. To ensure that your specified differential privacy protections are maintained, the OpenDP Library should mediate all access to the sensitive data set. This mediation is done via the Context API.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> import urllib.request
            >>> data_url = "https://raw.githubusercontent.com/opendp/opendp/sydney/teacher_survey.csv"
            >>> with urllib.request.urlopen(data_url) as data_req:
            ...     data = data_req.read().decode('utf-8')


            >>> context = dp.Context.compositor(
            ...     data=data,
            ...     privacy_unit=privacy_unit,
            ...     privacy_loss=privacy_loss,
            ...     split_evenly_over=3
            ... )

Since the privacy loss budget is at most ε = 1, and we are partitioning our budget evenly amongst three queries, then each query will be calibrated to satisfy ε = 1/3.

5. Submit DP Queries
^^^^^^^^^^^^^^^^^^^^

It is now time to create differentially private releases. The following query counts the number of records in the data set:

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> count_query = (
            ...     context.query()
            ...     .split_dataframe(",", col_names=col_names)
            ...     .select_column("age", str) # temporary until OpenDP 0.10 (Polars dataframe)
            ...     .count()
            ...     .laplace()
            ... )

The library uses the privacy unit and the query itself to determine the smallest amount of noise to add that will still satisfy the per-query privacy loss. Given these constraints, noise will be added to the count query with a scale of 3 (standard deviation of ~4.2).

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> scale = count_query.param()
            >>> scale
            3.0000000000000004

Here is the underlying mathematics that leads to this noise scale: if a teacher contributes at most one row, then the sensitivity of the count is one, because the addition or removal of a teacher can change the count by at most one. With the Laplace Mechanism, the noise scale (3) is the sensitivity (1) divided by the per-query privacy loss (ε = 1/3).

You can also create an accuracy estimate that is true at a (1 - α)100% confidence level:

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> accuracy = dp.discrete_laplacian_scale_to_accuracy(scale=scale, alpha=0.05)
            >>> accuracy
            9.445721638273584

When the discrete Laplace distribution's scale is 3, the DP estimate differs from the exact estimate by no more than 9.45 with 95% confidence.

If the accuracy of the query seems reasonable, then make a private release. Keep in mind, this action will permanently consume one of ``context``'s three queries we allocated when we launched the context API (each of which uses 1/3 of our privacy-loss budget).

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> dp_count = count_query.release()

The result is a random draw from the discrete Laplace distribution, centered at the true count of the number of records in the underlying data set (7000). Your previous accuracy estimate can now be used to create a confidence interval:

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> dp_count - accuracy, dp_count + accuracy

The exact count lies within the interval with 95% confidence.

This concludes the process of making a DP release.

Let's repeat this process more briefly for estimating the mean age. This time we benefit from having a DP count estimate in our public information: It is used to help calibrate the privacy guarantees for the mean.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> mean_query = (
            ...     context.query()
            ...     .split_dataframe(",", col_names=col_names)
            ...     .select_column("age", str)
            ...     .cast_default(float)
            ...     .clamp((18.0, 70.0))  # a best-guess based on public information
            ...     # Explanation for `constant=42`:
            ...     #    since dp_count may be larger than the true size, 
            ...     #    imputed rows will be given an age of 42.0 
            ...     #    (also a best guess based on public information)
            ...     .resize(size=dp_count, constant=42.0)
            ...     .mean()
            ...     .laplace()
            ... )

This measurement involves more preprocessing than the count did (casting, clamping, and resizing). The purpose of this preprocessing is to bound the sensitivity of the mean: the mean should only ever change by a small amount when any teacher is added or removed from the data set.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> mean_query.release()

The OpenDP Library supports more statistics, like the variance, various ways to compute histograms and quantiles, and PCA. The library also supports other mechanisms like the Gaussian Mechanism, which provides tighter privacy accounting when releasing a large number of queries, the Thresholded Laplace Mechanism, for releasing counts on data sets with unknown key sets, and variations of randomized response.

Framework API
-------------

The following sections show how the prior analysis looks in the Framework API.

1. Privacy Unit
^^^^^^^^^^^^^^^

The privacy unit is actually a 2-tuple:

input_metric, d_in = privacy_unit

assert d_in == 1 # neighboring data set distance is at most d_in...
assert input_metric == dp.symmetric_distance() # ...in terms of additions/removals

The privacy unit tuple specifies how distances are computed between two data sets (input_metric), and how large the distance can be (

).

2. Privacy Loss
^^^^^^^^^^^^^^^

The privacy loss is also a 2-tuple:

privacy_measure, d_out = privacy_loss

assert d_out == 1. # output distributions have distance at most d_out (ε)...
assert privacy_measure == dp.max_divergence(T=float) # ...in terms of pure-DP

The privacy loss tuple specifies how distances are measured between distributions (privacy_measure), and how large the distance can be (

).

3. Collect Public Information
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

TODO

4. Mediate Access to Data
^^^^^^^^^^^^^^^^^^^^^^^^^

dp.Context.compositor creates a sequential composition measurement.

m_sc = dp.c.make_sequential_composition(
    # data set is a single string, with rows separated by linebreaks
    input_domain=dp.atom_domain(T=str),
    input_metric=input_metric,
    output_measure=privacy_measure,
    d_in=d_in,
    d_mids=[d_out / 3] * 3,
)

The measurement is called with the data to create a compositor queryable:

qbl_sc = m_sc(data)

You can now submit up to three queries to qbl_sc, in the form of measurements.

5. Submit DP Queries
^^^^^^^^^^^^^^^^^^^^

First, create a count query.

t_count = (
    dp.t.make_split_dataframe(",", col_names=col_names)
    >> dp.t.make_select_column("age", str)
    >> dp.t.then_count()
)

    >> is a shorthand for chaining, or functional composition
    then_* uses the input domain and input metric from the prior transformation

With this lower-level API you get greater flexibility. For instance, you can see the sensitivity of the count query:

count_sensitivity = t_count.map(d_in)
count_sensitivity

1

A binary search is used to find the smallest noise scale that results in a measurement that satisfies

.

m_count = dp.binary_search_chain(
    lambda scale: t_count >> dp.m.then_laplace(scale), d_in, d_out / 3
)
dp_count = qbl_sc(m_count)

Similarly, construct a mean measurement and release it:

t_mean = (
    dp.t.make_split_dataframe(",", col_names=col_names) >>
    dp.t.make_select_column("age", str) >>
    dp.t.then_cast_default(float) >>
    dp.t.then_clamp((18.0, 70.0)) >>  # a best-guess based on public information
    dp.t.then_resize(size=dp_count, constant=42.0) >>
    dp.t.then_mean()
)

m_mean = dp.binary_search_chain(
    lambda scale: t_mean >> dp.m.then_laplace(scale), d_in, d_out / 3
)

qbl_sc(m_mean)

37.347899010945284

