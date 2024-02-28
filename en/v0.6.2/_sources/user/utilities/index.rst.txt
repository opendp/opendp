Utilities
=========

Aside from the constructors, there are very few remaining library interfaces.
The typing module aids in managing the data types used at each step in the computation chain, 
and the accuracy module is used to construct confidence intervals.
There are also binary and exponential search algorithms for finding a free parameter in a computation chain.

.. toctree::
    :titlesonly:

    typing
    accuracy/index
    parameter-search

The :ref:`putting-together` section provides an end-to-end example that makes use of all of these utilities.