Accuracy: Pitfalls and Edge Cases
=================================

This notebook describes OpenDP’s accuracy calculations, and ways in
which an analyst might be tripped up by them.

Overview
~~~~~~~~

Accuracy vs. Confidence Intervals
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Each privatizing mechanism (e.g. Laplace, Gaussian) in OpenDP has an
associated accuracy that is a function of alpha and the noise scale.
Imagine you have data :math:`D`, and you want, for some function
:math:`\phi` to return :math:`\phi(D)` in a differentially private way –
we will call this value :math:`\phi_{dp}(D)`. An :math:`\alpha`-level
accuracy guarantee :math:`a` promises that, over infinite runs of the
privatizing mechanism on the data in question,

.. math::  \phi(D) \in [\phi_{dp}(D) - a, \phi_{dp}(D) + a] 

with probability at least :math:`1 - \alpha`.

This looks very much like the traditional confidence interval, but it is
important to note a major difference. In a canonical confidence
interval, the uncertainty being represented is due to sampling error –
that is, how often will it be the case that :math:`\phi(P)` (the value
of :math:`\phi` on the underlying population) is within some range of
the realized :math:`\phi(D)`.

In OpenDP (and differentially private data analysis generally), there is
an extra layer of uncertainty due to the noise added to :math:`\phi(D)`
to produce :math:`\phi_{dp}(D)`. OpenDP’s accuracy utilities described
below deal only with the uncertainty of :math:`\phi_{dp}(D)` relative to
:math:`\phi(D)` and not the uncertainty of :math:`\phi(D)` relative to
:math:`\phi(P)`, but there is ongoing work to provide methods that
incorporate both.

What is :math:`D`?
^^^^^^^^^^^^^^^^^^

OpenDP allows for analysis of data with an unknown number of rows by
resizing the data to ensure consistency with an estimated size (see the
`unknown dataset size
notebook <../getting-started/examples/unknown-dataset-size.ipynb>`__ for
more details). Accuracy guarantees are always relative to the
preprocessed data :math:`\tilde{D}` and operations such as imputation
and clipping are not factored into the accuracy.

Synopsis
^^^^^^^^

Say an analyst releases :math:`\phi_{dp}(D)` and gets an accuracy
guarantee of :math:`a` at accuracy-level :math:`\alpha` using the
accuracy utilities described below. :math:`D` is a dataset of unknown
size drawn from population :math:`P` and will be resized to
:math:`\tilde{D}`. This suggests that over infinite runs of this
procedure,

-  :math:`\phi_{dp}(D) \in [\phi(\tilde{D}) - a, \phi(\tilde{D}) + a]`
   with probability :math:`1 - \alpha`

-  It is likely that :math:`\phi_{dp}(D) \in [\phi(D) - a, \phi(D) + a]`
   with probability :math:`\approx 1 - \alpha`, though we cannot make
   any guarantee. For many cases (e.g. resizing the data based on
   :math:`n` obtained from a differentially private count and reasonable
   bounds on the data elements), this is likely to be approximately
   true. In the next section, we will explore some examples of cases
   where this statement holds to varying extents.

-  We cannot directly make statements about the relationship uncertainty
   of :math:`\phi_{dp}(D)` relative to :math:`\phi(P)`.

Accuracy Guarantees In Practice
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

We now move to some empirical evaluations of how well our accuracy
guarantees translate from :math:`\phi(\tilde{D})` to :math:`\phi(D)`. We
first consider the case where we actually know the size of the
underlying data and are able to set plausible lower/upper bounds on
``age``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import numpy as np
            >>> import pandas as pd
            >>> import opendp.prelude as dp
            
            >>> dp.enable_features("contrib")
            
            >>> var_names = ["age", "sex", "educ", "race", "income", "married", "pid"]
            >>> D = pd.read_csv(dp.examples.get_california_pums_path(), names=var_names)
            >>> age = D.age
            >>> D_mean_age = np.mean(age)
            
            >>> # This will provide the data that will be passed to the aggregator
            >>> data = dp.examples.get_california_pums_path().read_text()
            
            >>> # establish extra information for this simulation
            >>> age_bounds = (0., 100.)
            >>> n_sims = 100
            >>> epsilon = 1.
            >>> alpha = 0.05
            
            >>> D_tilde_mean_age = np.mean(np.clip(D.age, age_bounds[0], age_bounds[1]))
            >>> impute_constant = 50.
            
            >>> def make_mean_aggregator(data_size):
            ...     return (
            ...         # Convert data into a dataframe of string columns
            ...         dp.t.make_split_dataframe(separator=",", col_names=var_names) >>
            ...         # Selects a column of df, Vec<str>
            ...         dp.t.make_select_column(key="age", TOA=str) >>
            ...         # Cast the column as Vec<float>
            ...         dp.t.then_cast(TOA=float) >>
            ...         # Impute null values
            ...         dp.t.then_impute_constant(impute_constant) >>
            ...         # Clamp age values
            ...         dp.t.then_clamp(bounds=age_bounds) >>
            ...         # Resize the dataset to length `data_size`.
            ...         #     If there are fewer than `data_size` rows in the data, fill with a constant.
            ...         #     If there are more than `data_size` rows in the data, only keep `data_size` rows
            ...         dp.t.then_resize(size=data_size, constant=impute_constant) >>
            ...         dp.t.then_mean()
            ...     )
            

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> data_size = 1_000
            >>> mean_aggregator = make_mean_aggregator(data_size)
            >>> scale = dp.binary_search_param(lambda s: mean_aggregator >> dp.m.then_laplace(s), 1, epsilon)
            >>> measurement = mean_aggregator >> dp.m.then_laplace(scale)
            
            >>> releases = [measurement(data) for _ in range(n_sims)]
            >>> accuracy = dp.laplacian_scale_to_accuracy(scale, alpha)
            
            >>> print('Accuracy interval (with accuracy value {0}) contains the true mean on D_tilde with probability {1}'.format(
            ...     round(accuracy, 4),
            ...     np.mean([(D_tilde_mean_age >= val - accuracy) & (D_tilde_mean_age <= val + accuracy) for val in releases])))
            Accuracy interval (with accuracy value 0.2996) contains the true mean on D_tilde with probability ...
            
            >>> print('Accuracy interval (with accuracy value {0}) contains the true mean on D with probability {1}'.format(
            ...     round(accuracy, 4),
            ...     np.mean([(D_mean_age >= val - accuracy) & (D_mean_age <= val + accuracy) for val in releases])))
            Accuracy interval (with accuracy value 0.2996) contains the true mean on D with probability ...

This performance is as expected. :math:`D` and :math:`\tilde{D}` are
actually the exact same data (the maximum age in the raw data is 93, so
our clamp to :math:`[0, 100]` does not change any values, and we know
the correct :math:`n`), so our theoretical guarantees on
:math:`\tilde{D}` map exactly to guarantees on :math:`D`.

We now move to a scenario that is still realistic, but where the
performance does not translate quite as well. In this case, we imagine
that the analyst believes the data to be of size 1050 and uses the
default imputation within resize so that the extra 50 elements are
replaced with a constant.

Note that our diagnostic testing of :math:`\tilde{D}` in the code above
is not trivial in this case. In the first example, we knew that
clamp/resize did not change the underlying data, so we could predict
exactly the data on which the DP mean would actually be calculated. This
will not be true for the following examples, so we will simulate finding
the true underlying mean by releasing an extra DP mean with very high
epsilon.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # This estimate is larger than the true size of 1000, so we will impute 50 values using the impute constant
            >>> data_size = 1_050
            >>> mean_aggregator = make_mean_aggregator(data_size)
            >>> # This value contains the true mean of the data after resizing and imputation
            >>> D_tilde_mean = mean_aggregator(data)
            >>> scale = dp.binary_search_param(lambda s: mean_aggregator >> dp.m.then_laplace(s), 1, epsilon)
            >>> measurement = mean_aggregator >> dp.m.then_laplace(scale)

            >>> releases = [measurement(data) for _ in range(n_sims)]            
            >>> accuracy = dp.laplacian_scale_to_accuracy(scale, alpha)
            
            >>> print('Accuracy interval (with accuracy value {0}) contains the true mean on D_tilde with probability {1}'.format(
            ...     round(accuracy, 4),
            ...     np.mean([(D_tilde_mean >= dp_mean - accuracy) & (D_tilde_mean <= dp_mean + accuracy)
            ...              for dp_mean in releases])))
            Accuracy interval (with accuracy value 0.2853) contains the true mean on D_tilde with probability ...

            >>> print('Accuracy interval (with accuracy value {0}) contains the true mean on D with probability {1}'.format(
            ...     round(accuracy, 4),
            ...     np.mean([(D_mean_age >= dp_mean - accuracy) & (D_mean_age <= dp_mean + accuracy) for dp_mean in releases])))
            Accuracy interval (with accuracy value 0.2853) contains the true mean on D with probability ...

The accuracy guarantee still holds on :math:`\tilde{D}` (as it should),
but we now see much worse performance relative to the true underlying
data :math:`D`.
