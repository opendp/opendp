
.. _accuracy-user-guide:

Accuracy
--------

(See also :py:mod:`opendp.accuracy` in the API reference.)

The library contains utilities to estimate accuracy at a given noise scale and statistical significance level
or derive the necessary noise scale to meet a given target accuracy and statistical significance level.

.. note::

    This confidence interval is specifically for the input to the noise addition mechanism.
    The library currently does not compensate for the bias introduced from clipping or other preprocessing
    (`[KMRS+23] <https://arxiv.org/pdf/2301.13334.pdf>`_ shows that this is somewhat unavoidable).
    In the Theory section we have `a notebook demonstrating this limitation <../../../theory/accuracy-pitfalls.html>`_.

The noise distribution may be either Laplace or Gaussian.

:Laplacian: | Applies to any L1 noise addition mechanism.
  | :func:`make_laplace() <opendp.measurements.make_laplace>`
  | :func:`make_base_laplace_threshold() <opendp.measurements.make_base_laplace_threshold>`
:Gaussian: | Applies to any L2 noise addition mechanism.
  | :func:`make_gaussian() <opendp.measurements.make_gaussian>`

The library provides the following functions for converting between noise scale and accuracy:

* :func:`opendp.accuracy.laplacian_scale_to_accuracy`
* :func:`opendp.accuracy.accuracy_to_laplacian_scale`
* :func:`opendp.accuracy.gaussian_scale_to_accuracy`
* :func:`opendp.accuracy.accuracy_to_gaussian_scale`

To demonstrate, the following snippet finds the necessary gaussian scale such that the input to 
:code:`make_gaussian(input_domain, input_metric, scale=1.)` differs from the release by no more than 2 with 95% confidence.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> confidence = 95
        >>> dp.accuracy_to_gaussian_scale(accuracy=2., alpha=1. - confidence / 100)
        1.020426913849308

You can generally plug the distribution (Laplace or Gaussian), scale, accuracy and alpha
into the following statement to interpret these functions:

.. code-block:: python

    f"When the {distribution} scale is {scale}, "
    f"the DP estimate differs from the true value by no more than {accuracy} "
    f"at a statistical significance level alpha of {alpha}, "
    f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence."
