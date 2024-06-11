Aggregation: Mean
=================

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. code:: ipython3

    import opendp.prelude as dp
    dp.enable_features("contrib")

Known Dataset Size
------------------

The much easier case to consider is when the dataset size is known:

.. code:: ipython3

    input_space = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=10), dp.symmetric_distance()
    sb_mean_trans = dp.t.make_mean(*input_space)
    sb_mean_trans([5.] * 10)




.. parsed-literal::

    5.0



The sensitivity of this transformation is the same as in ``make_sum``
(when dataset size is known), but is divided by ``size``.

That is, :math:`map(d_{in}) = (d_{in} // 2) \cdot max(|L|, U) / size`,
where :math:`//` denotes integer division with truncation.

.. code:: ipython3

    # since we are in the bounded-DP model, d_in should be a multiple of 2, 
    # because it takes one removal and one addition to change one record
    sb_mean_trans.map(2)




.. parsed-literal::

    1.0000000000000169



Note that this operation does not divide by the length of the input
data, it divides by the size parameter passed to the constructor. As in
any other context, it is expected that the data passed into the function
is a member of the input domain, so no promises of privacy or
correctness are guaranteed when the data is not in the input domain. In
particular, the function may give a result with no error message.

.. code:: ipython3

    sb_mean_trans = dp.t.make_mean(*input_space)
    sb_mean_trans([5.])




.. parsed-literal::

    0.5



You can check that a dataset is a member of a domain by calling
``.member``:

.. code:: ipython3

    sb_mean_trans.input_domain.member([5.])




.. parsed-literal::

    False



In this case, ``[5.]`` is not a member because the input domain consists
of vectors of length ten.

Unknown Dataset Size
--------------------

There are several approaches for releasing the mean when the dataset
size is unknown.

The first approach is to use the resize transformation. You can
separately release an estimate for the dataset size, and then preprocess
the dataset with a resize transformation.

.. code:: ipython3

    data = [5.] * 10
    bounds = (0., 10.)
    
    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()
    
    # (where TIA stands for Atomic Input Type)
    count_meas = input_space >> dp.t.then_count() >> dp.m.then_laplace(1.)
    
    dp_count = count_meas(data)
    
    mean_meas = (
        input_space >>
        dp.t.then_clamp(bounds) >>
        dp.t.then_resize(dp_count, constant=5.) >> 
        dp.t.then_mean() >>
        dp.m.then_laplace(1.)
    )
    
    mean_meas(data)





.. parsed-literal::

    6.862637830873848



The total privacy expenditure is the composition of the ``count_meas``
and ``mean_meas`` releases.

.. code:: ipython3

    dp.c.make_basic_composition([count_meas, mean_meas]).map(1)




.. parsed-literal::

    2.000000000000017



Another approach is to compute the DP sum and DP count, and then
postprocess the output.

.. code:: ipython3

    dp_sum = input_space >> dp.t.then_clamp(bounds) >> dp.t.then_sum() >> dp.m.then_laplace(10.)
    dp_count = input_space >> dp.t.then_count() >> dp.m.then_laplace(1.)
    
    dp_fraction_meas = dp.c.make_basic_composition([dp_sum, dp_count])
    
    dp_sum, dp_count = dp_fraction_meas(data)
    print("dp mean:", dp_sum / dp_count)
    print("epsilon:", dp_fraction_meas.map(1))


.. parsed-literal::

    dp mean: 7.778118283305409
    epsilon: 2.000000009313226


The same approaches are valid for the variance estimator. The `Unknown
Dataset Size
notebook <../../../getting-started/examples/unknown-dataset-size.ipynb>`__
goes into greater detail on the tradeoffs of these approaches.
