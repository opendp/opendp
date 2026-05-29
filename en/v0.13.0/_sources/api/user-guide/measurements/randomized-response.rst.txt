Randomized Response
===================

Randomized response is used to release categorical survey responses in
the local-DP, or per-user, model. The randomized response algorithm is
typically meant to be run on the edge, at the user’s device, before data
is submitted to a central server. Local DP is a stronger privacy model
than the central model, because the central data aggregator is only ever
privileged to privatized data.

OpenDP currently only provides mechanisms that may be run on the edge
device: You must handle network communication and aggregation.

--------------

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            

Privatization
-------------

We’ll start by privatizing a boolean response. Boolean randomized
response is fully characterized by a *measurement* containing the
following six elements:

.. dropdown:: Elements of a Boolean Randomized Response Measurement

   1. We first define the **function** :math:`f(\cdot)`, that applies the
      randomized response to some boolean argument :math:`x`. The function
      returns the correct answer with probability ``prob``, otherwise it
      flips the answer.

   .. math:: f(x) = x \wedge \neg \mathrm{sample\_bernoulli}(prob)

   2. :math:`f(\cdot)` is only well-defined for boolean inputs. This
      (small) set of permitted inputs is described by the **input domain**
      (denoted ``AtomDomain<bool>``).

   3. The set of possible outputs is described by the **output domain**
      (also ``AtomDomain<bool>``).

   4. Randomized response has a privacy guarantee in terms of epsilon. This
      guarantee is represented by a **privacy map**, a function that
      computes the privacy loss :math:`\epsilon` for any choice of
      sensitivity :math:`\Delta`.

   .. math:: map(d_{in}) = d_{in} \cdot \ln(\mathrm{prob} / (1 - \mathrm{prob}))

   5. This map requires that :math:`d_{in}` be a discrete distance, which
      is either 0 if the elements are the same, or 1 if the elements are
      different. This is used as the **input metric**
      (``DiscreteDistance``).

   6. We similarly describe units on the output (:math:`\epsilon`) via the
      **output measure** (``MaxDivergence``).

``make_randomized_response_bool`` returns the equivalent measurement:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # construct the measurement
            >>> rr_bool_meas = dp.m.make_randomized_response_bool(prob=0.75)
            
            >>> # invoke the measurement on some survey response to execute the randomized response algorithm
            >>> alice_survey_response = True
            >>> print("noisy release:", rr_bool_meas(alice_survey_response))
            noisy release: ...

            >>> # determine epsilon by invoking the map
            >>> print("epsilon:", rr_bool_meas.map(d_in=1))
            epsilon: 1.0986...

A simple generalization of the previous algorithm is to randomize over a
custom category set:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # construct the measurement
            >>> categories = ["A", "B", "C", "D"]
            >>> rr_meas = dp.m.make_randomized_response(categories, prob=0.75)
            
            >>> # invoke the measurement on some survey response, to execute the randomized response algorithm
            >>> alice_survey_response = "C"
            >>> print("noisy release:", rr_meas(alice_survey_response))
            noisy release: ...

            >>> # determine epsilon by invoking the map
            >>> print("epsilon:", rr_meas.map(d_in=1))
            epsilon: 2.197...

Aggregation: Mean
-----------------

The privatized responses from many individuals may be aggregated to form
a population-level inference. In the case of the boolean randomized
response, you may want to estimate the proportion of individuals who
actually responded with ``True``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import numpy as np
            >>> num_responses = 1000
            
            >>> true_probability = .23
            
            >>> private_bool_responses = []
            
            >>> for _ in range(num_responses):
            ...     response = bool(np.random.binomial(n=1, p=true_probability))
            ...     randomized_response = rr_bool_meas(response)
            ...     private_bool_responses.append(randomized_response)
            
            >>> naive_proportion = np.mean(private_bool_responses)
            >>> print('naive proportion:', naive_proportion)
            naive proportion: ...

We know the true probability is .23, so our estimate is off!

The naive proportions can be corrected for bias via the following
derivation:

.. dropdown:: Derivation of Boolean RR Bias Correction

   We want an unbiased estimate of :math:`\frac{\sum X_i}{n}`. Denote the
   randomized response :math:`Y_i = \texttt{rr\_bool\_meas}(X_i)`. We first
   find the expectation of :math:`Y_i`:

   .. math::

      \begin{align*}
         E[Y_i] &= p X_i + (1 - p) (1 - X_i) \\
            &= p X_i + p X_i - p - X_i + 1 \\
            &= (2 p - 1) X_i - p + 1
      \end{align*}

   This can be used as an unbiased estimator for the proportion of true
   answers:

   .. math::

      \begin{align*}
         E[X_i] = \frac{E[Y_i] + p - 1}{2 p - 1}
      \end{align*}

The resulting expression is distilled into the following function:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def debias_randomized_response_bool(mean_release, p):
            ...     """Adjust for the bias of the mean of a boolean RR dataset."""
            ...     assert 0 <= mean_release <= 1
            ...     assert 0 <= p <= 1
            ...     
            ...     return (mean_release + p - 1) / (2 * p - 1)
            
            >>> estimated_bool_proportion = debias_randomized_response_bool(naive_proportion, .75)
            >>> print('estimated:', estimated_bool_proportion)
            estimated: ...

As expected, the bias correction admits a useful estimate of the
population proportion (``.23``).

The categorical randomized response will suffer the same bias:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import numpy as np
            >>> num_responses = 1000
            
            >>> true_probability = [0.1, 0.4, 0.3, 0.2]
            
            >>> private_cat_responses = []
            
            >>> for _ in range(num_responses):
            ...     response = np.random.choice(categories, p=true_probability)
            ...     randomized_response = rr_meas(response)
            ...     private_cat_responses.append(randomized_response)
            
            >>> from collections import Counter
            
            >>> counter = Counter(private_cat_responses)
            >>> naive_cat_proportions = [counter[cat] / num_responses for cat in categories]
            >>> naive_cat_proportions
            [..., ..., ..., ...]

We can do the same analysis to de-bias the categorical estimate:

.. dropdown:: Derivation of Categorical RR Bias Correction

   Denote the randomized response :math:`Y_i = \texttt{rr\_meas}(X_i)`, and
   the :math:`k^{th}` category as :math:`C_k`.

   We first find :math:`E[I[Y_i = C_k]]` (the expectation that noisy
   responses are equal to the :math:`k^{th}` category). This is done by
   considering the law of total probability over all categories.

   .. math::

      \begin{align*}
         E[I[Y_i = C_k]] &= p \cdot I[X_i = C_k] + \sum_{j \ne k} \frac{1 - p}{K - 1} \cdot I[X_i = C_j] \\
            &= p \cdot I[X_i = C_k] + \frac{1 - p}{K - 1} \cdot (1 - I[X_i = C_k])
      \end{align*}

   Then solve for :math:`E[I[X_i = C_k]]` (the expectation that raw
   responses are equal to the :math:`k^{th}` category):

   .. math::

      \begin{align*}
         E[I[Y_i = C_k]] (K - 1) &= p \cdot E[I[X_i = C_k]] (K - 1) + (1 - p)(1 - E[I[X_i = C_k]]) \\
         E[I[Y_i = C_k]] (K - 1) &= p \cdot E[I[X_i = C_k]] K - p - E[I[X_i = C_k]] + 1 \\
         E[I[Y_i = C_k]] (K - 1) + p - 1 &= E[I[X_i = C_k]] (pK - 1) \\
         \frac{E[I[Y_i = C_k]] (K - 1) + p - 1}{pK - 1} &= E[I[X_i = C_k]]
      \end{align*}


This formula is represented in the following function:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def debias_randomized_response(mean_releases, p):
            ...     """Adjust for the bias of the mean of a categorical RR dataset."""
            ...     mean_releases = np.array(mean_releases)
            ...     assert all(mean_releases >= 0) and abs(sum(mean_releases) - 1) < 1e-6
            ...     assert 0 <= p <= 1
            ...     
            ...     k = len(mean_releases)
            ...     return (mean_releases * (k - 1) + p - 1) / (p * k - 1)
            

We similarly estimate population parameters in the categorical setting:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> estimated_cat_proportions = debias_randomized_response(naive_cat_proportions, .75)
            
            >>> print("true probability:", true_probability)
            true probability: [0.1, 0.4, 0.3, 0.2]
            >>> print("estimated probability:", estimated_cat_proportions)
            estimated probability: [... ... ... ...]

Aggregation: Count
------------------

Just like the mean was biased, so is a simple count of responses for
each category:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("biased boolean count:", np.sum(private_bool_responses))
            biased boolean count: ...
            >>> print("biased categorical count:", dict(sorted(Counter(private_cat_responses).items())))
            biased categorical count: {'A': ..., 'B': ..., 'C': ..., 'D': ...}

Since the dataset size is known, simply post-process the mean estimates:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> estimated_bool_count = int(estimated_bool_proportion * num_responses)
            >>> estimated_cat_count = dict(zip(categories, (estimated_cat_proportions * num_responses).astype(int)))
            
            >>> print("unbiased boolean count:", estimated_bool_count)
            unbiased boolean count: ...
            >>> print("unbiased categorical count:", estimated_cat_count)
            unbiased categorical count: {'A': ..., 'B': ..., 'C': ..., 'D': ...}
            