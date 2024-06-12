Example Data Analysis
=====================

In this notebook we use OpenDP to conduct a brief data analysis of a
PUMS microdata sample from the US Census Bureau. The data consists of
demographic information of 1000 individuals. Each individual in the
dataset contributes at most one row.

.. code:: ipython3

    # the greatest number of records that any one individual can influence in the dataset
    max_influence = 1

We now establish what is considered “public knowledge” about the data.

.. code:: ipython3

    # establish public information
    col_names = ["age", "sex", "educ", "race", "income", "married"]
    # we can also reasonably intuit that age and income will be numeric,
    #     as well as bounds for them, without looking at the data
    age_bounds = (0, 100)
    income_bounds = (0, 150_000)

The vetting process is currently underway for the code in the OpenDP
Library. Any constructors that have not been vetted may still be
accessed if you opt-in to “contrib”.

.. code:: ipython3

    import opendp.prelude as dp
    dp.enable_features('contrib')

Working with CSV data
~~~~~~~~~~~~~~~~~~~~~

Let’s examine how we can process csv data with Transformations.

I’m going to pull a few constructors from the `Dataframes section in the
user
guide <https://docs.opendp.org/en/stable/user/transformations.html#dataframe>`__.

We start with ``make_split_dataframe`` to parse one large string
containing all the csv data into a dataframe. ``make_split_dataframe``
expects us to pass column names, which we can grab out of the public
information. ``make_select_column`` will then index into the dataframe
to pull out a column where the elements have a given type ``TOA``. The
``TOA`` argument won’t cast for you; the casting comes later!

.. code:: ipython3

    income_preprocessor = (
        # Convert data into a dataframe where columns are of type Vec<str>
        dp.t.make_split_dataframe(separator=",", col_names=col_names) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column(key="income", TOA=str)
    )

For the sake of exposition, we’re going to go ahead and load up the
data.

.. code:: ipython3

    ![ -e data.csv ] || wget https://raw.githubusercontent.com/opendp/opendp/main/docs/source/data/PUMS_california_demographics_1000/data.csv

.. code:: ipython3

    with open('data.csv') as input_file:
        data = input_file.read()
    
    print('\n'.join(data.split('\n')[:6]))


.. parsed-literal::

    59,1,9,1,0,1
    31,0,1,3,17000,0
    36,1,11,1,0,1
    54,1,11,1,9100,1
    39,0,5,3,37000,0
    34,0,9,1,0,1


As we can see from the first few rows, it is intentional that there are
no column names in the data. If your data has column names, you will
want to strip them out before passing data into your function.

Now if you run the transformation on the data, you will get a list of
incomes as strings. I’ve limited the output to just the first few income
values.

.. code:: ipython3

    transformed = income_preprocessor(data)
    print(type(transformed))
    print(transformed[:6])


.. parsed-literal::

    <class 'list'>
    ['0', '17000', '0', '9100', '37000', '0']


Casting
~~~~~~~

Income doesn’t make sense as a string for our purposes, so we can just
extend the previous preprocessor to also cast and impute.

.. code:: ipython3

    # make a transformation that casts from a vector of strings to a vector of ints
    cast_str_int = (
        # start with the output space of the income_preprocessor
        income_preprocessor.output_space >>
        # cast Vec<str> to Vec<Option<int>>
        dp.t.then_cast(TOA=int) >>
        # Replace any elements that failed to parse with 0, emitting a Vec<int>
        dp.t.then_impute_constant(0)
    )
    
    # replace the previous preprocessor: extend it with the caster
    income_preprocessor = income_preprocessor >> cast_str_int
    print(income_preprocessor(data)[:6])


.. parsed-literal::

    [0, 17000, 0, 9100, 37000, 0]


Great! Now we have integer income data from our CSV. A quick aside, keep
in mind that we can invoke transformations on almost anything to do some
testing. For example, we still have a handle on ``cast_str_int``, don’t
we?

.. code:: ipython3

    cast_str_int(["null", "1.", "2"])




.. parsed-literal::

    [0, 0, 2]



Private Count
~~~~~~~~~~~~~

Time to compute our first aggregate statistic. Suppose we want to know
the number of records in the dataset.

We can use the `list of
aggregators <https://docs.opendp.org/en/stable/user/transformations.html#aggregators>`__
in the Transformation Constructors section of the user guide to find
``make_count``.

.. code:: ipython3

    count = income_preprocessor >> dp.t.then_count()
    # NOT a DP release!
    count_response = count(data)

Be careful! ``count`` is still only a transformation, so the output in
``count_response`` is not a differentially private release. You will
need to chain with a measurement to create a differentially private
release.

When you use ``then_laplace`` below, it automatically chooses a discrete
variation of the mechanism for privatizing integers. Notice that the
function now comes from ``dp.m`` (denoting measurement constructors),
and the resulting ``type(dp_count)`` is ``Measurement``. This tells us
that the output will be a differentially private release.

.. code:: ipython3

    dp_count = count >> dp.m.then_laplace(scale=1.)

In any realistic situation, you would likely want to estimate the budget
utilization before you make a release. Use a search utility to quantify
the privacy expenditure of this release.

.. code:: ipython3

    # estimate the budget...
    epsilon = dp.binary_search(
        lambda eps: dp_count.check(d_in=max_influence, d_out=eps),
        bounds=(0., 100.))
    print("DP count budget:", epsilon)
    
    # ...and then release
    count_release = dp_count(data)
    print("DP count:", count_release)


.. parsed-literal::

    DP count budget: 1.0
    DP count: 1000


Private Sum
~~~~~~~~~~~

Suppose we want to know the total income of our dataset. First, take a
look at `the list of
aggregators <https://docs.opendp.org/en/stable/user/transformations.html#aggregators>`__.
``make_sum`` meets our requirements. As indicated by the function’s API
documentation, it expects bounded data, so we’ll also need to chain the
transformation from ``then_clamp`` with the ``income_preprocessor``.

.. code:: ipython3

    bounded_income_sum = (
        income_preprocessor >>
        # clamp income values. 
        # "then_*" means it uses the output domain and output metric from the previous transformation
        dp.t.then_clamp(bounds=income_bounds) >>
        # similarly, here we use "then_sum" to avoid needing to specify the input space.
        # the sum constructor gets told that the input consists of bounded data
        dp.t.then_sum()
    )

In this example, instead of just passing a scale into ``make_laplace``,
I want whatever scale will make my measurement 1-epsilon DP. Again, I
can use a search utility to find such a scale.

.. code:: ipython3

    discovered_scale = dp.binary_search_param(
        lambda s: bounded_income_sum >> dp.m.then_laplace(scale=s),
        d_in=max_influence,
        d_out=1.)
    
    dp_sum = bounded_income_sum >> dp.m.then_laplace(scale=discovered_scale)

Or more succinctly…

.. code:: ipython3

    dp_sum = dp.binary_search_chain(
        lambda s: bounded_income_sum >> dp.m.then_laplace(scale=s),
        d_in=max_influence,
        d_out=1.)
    
    # ...and make our 1-epsilon DP release
    print("DP sum:", dp_sum(data))


.. parsed-literal::

    DP sum: 30633527


Private Mean
~~~~~~~~~~~~

We may be more interested in the mean age. The constructor expects
sized, bounded data, and the docstring points us toward preprocessors we
can use.

Sized data is data that has a known number of rows. The constructor
enforces this requirement because knowledge of the dataset size is
necessary to bound the sensitivity of the function.

Luckily, we’ve already made a DP release of the number of rows in the
dataset, which we can reuse as an argument here.

Putting the previous sections together, our bounded mean age is:

.. code:: ipython3

    try:
        mean_age_preprocessor = (
            # Convert data into a dataframe of string columns
            dp.t.make_split_dataframe(separator=",", col_names=col_names) >>
            # Selects a column of df, Vec<str>
            dp.t.make_select_column(key="age", TOA=str) >>
            # Cast the column as Vec<float>, and fill nulls with the default value, 0.
            dp.t.then_cast_default(TOA=float) >>
            # Clamp age values
            dp.t.then_clamp(bounds=age_bounds)
        )
    except TypeError as err:
        assert str(err).startswith("inferred type is") # type: ignore
        print(err)


.. parsed-literal::

    inferred type is i32, expected f64. See https://github.com/opendp/opendp/discussions/298


Wait a second! The types don’t match? In this case, we casted to
float-valued data, but ``then_clamp`` was built with integer-valued
bounds, so the clamp is expecting integer data. Therefore, the output of
the cast is not a valid input to the clamp. We can fix this by adjusting
the bounds and trying again.

.. code:: ipython3

    float_age_bounds = tuple(map(float, age_bounds))
    
    dp_mean = (
        # Convert data into a dataframe of string columns
        dp.t.make_split_dataframe(separator=",", col_names=col_names) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column(key="age", TOA=str) >>
        # Cast the column as Vec<float>, and fill nulls with the default value, 0.
        dp.t.then_cast_default(TOA=float) >>
        # Clamp age values
        dp.t.then_clamp(bounds=float_age_bounds) >>
        # Resize the dataset to length `count_release`.
        #     If there are fewer than `count_release` rows in the data, fill with a constant of 20.
        #     If there are more than `count_release` rows in the data, only keep `count_release` rows
        dp.t.then_resize(size=count_release, constant=20.) >>
        # Compute the mean
        dp.t.then_mean() >>
        # add laplace noise
        dp.m.then_laplace(scale=1.0)
    )
    
    mean_release = dp_mean(data)
    print("DP mean:", mean_release)

Depending on your use-case, you may find greater utility separately
releasing a DP sum and a DP count, and then postprocessing them into the
mean. In the above mean example, you could even take advantage of this
to avoid using floating-point numbers.

Zero-Concentrated Differential Privacy
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In this example, I chain with the gaussian mechanism instead, with a
budget of .05 rho.

.. code:: ipython3

    variance = (
        # Convert data into a dataframe of string columns
        dp.t.make_split_dataframe(separator=",", col_names=col_names) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column(key="age", TOA=str) >>
        # Cast the column as Vec<float>, and fill nulls with the default value, 0.
        dp.t.then_cast_default(TOA=float) >>
        # Clamp age values
        dp.t.then_clamp(bounds=float_age_bounds) >>
        # Resize the dataset to length `count_release`.
        dp.t.then_resize(size=count_release, constant=20.) >>
        # Compute the variance
        dp.t.then_variance()
    )
    
    dp_variance = dp.binary_search_chain(
        lambda s: variance >> dp.m.then_gaussian(scale=s), 
        d_in=max_influence, d_out=.05)
    
    print("DP variance:", dp_variance(data))


.. parsed-literal::

    DP variance: 45.154334857855616


Measure Casting
~~~~~~~~~~~~~~~

In the previous example, we have a privacy parameter in terms of rho. We
can use ``make_zCDP_to_approxDP`` to convert the distance type to an
ε(δ) pareto-curve.

.. code:: ipython3

    app_dp_variance = dp.c.make_zCDP_to_approxDP(dp_variance)
    # evaluate the privacy map to get a curve
    curve = app_dp_variance.map(max_influence)
    # solve for epsilon when delta is fixed
    curve.epsilon(delta=1e-7)




.. parsed-literal::

    1.6194085342284823



We can use ``make_fix_delta`` to emit (ε, δ) pairs instead:

.. code:: ipython3

    fixed_app_dp_variance = dp.c.make_fix_delta(app_dp_variance, delta=1e-7)
    fixed_app_dp_variance.map(max_influence)




.. parsed-literal::

    (1.6194085342284823, 1e-07)



This can be used in conjunction with the binary search utilities to
solve for a scale parameter:

.. code:: ipython3

    budget = (1., 1e-7)
    def make_dp_variance(scale):
        dp_var = variance >> dp.m.then_gaussian(scale)
        return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp_var), delta=budget[1])
    
    dp_variance_lte_budget = dp.binary_search_chain(
        make_dp_variance, 
        d_in=max_influence, d_out=budget)
    
    # we know this measurement is calibrated to be lte budget
    assert dp_variance_lte_budget.check(max_influence, budget)

Composition
~~~~~~~~~~~

We can compose multiple measurements into one measurement with
``make_basic_composition``:

.. code:: ipython3

    composed = dp.c.make_basic_composition([dp_sum, dp_mean])
    composed(data)




.. parsed-literal::

    [30419934, 44.75968544038662]



In order to compose, all measurements must share the same input domain,
input metric and output measure. We can still use the privacy map to see
the epsilon usage of this new measurement:

.. code:: ipython3

    composed.map(max_influence)




.. parsed-literal::

    1.1000000000004568



Population Amplification
~~~~~~~~~~~~~~~~~~~~~~~~

Another type of combinator is an amplifier. In this example I’ll apply
the amplifier to a dp variance estimator:

.. code:: ipython3

    variance = dp.t.make_variance(
        input_domain=dp.vector_domain(dp.atom_domain(float_age_bounds), count_release),
        input_metric=dp.symmetric_distance())
    
    dp_variance = dp.binary_search_chain(
        lambda s: variance >> dp.m.then_laplace(scale=s),
        d_in=max_influence,
        d_out=1.
    )
    
    # requires a looser trust model, as the population size can be set arbitrarily
    dp.enable_features("honest-but-curious")
    
    dp.c.make_population_amplification(dp_variance, 100_000).map(1)




.. parsed-literal::

    0.017036863236176553



You’ll notice that we found a dp variance estimator that was 1
epsilon-DP, but after amplification, it now uses a much smaller epsilon.
We are taking advantage of the knowledge that the dataset was a simple
sample from a larger population with at least 100,000 individuals.
