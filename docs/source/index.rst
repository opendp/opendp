Welcome
=======

What is differential privacy?
-----------------------------

Differential privacy is a rigorous mathematical definition of privacy.
Consider an algorithm that analyzes a dataset and releases statistics:
The algorithm is differentially private if by looking at the output,
you cannot tell whether any individual's data was included in the original dataset or not.
Differential privacy achieves this by carefully injecting random noise into the released statistics to hide the effects of each individual. 

For more background on differential privacy and its applications:

* "`Designing Access with Differential Privacy <https://admindatahandbook.mit.edu/book/v1.0/diffpriv.html>`_" from *Handbook on Using Administrative Data for Research and Evidence-based Policy*
* `Resources list <https://differentialprivacy.org/resources/>`_ from *differentialprivacy.org*

Why OpenDP?
-----------

* OpenDP is based on a `solid conceptual framework <https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf>`_ for expressing privacy-aware computations.
* OpenDP is built on a Rust core for memory and thread safety and performance.
* OpenDP has a process for independent review of algorithms and implementations.
* OpenDP has performed well in `independent security audits <https://www.sri.inf.ethz.ch/publications/lokna2023groupandattack>`_.
* OpenDP supports a range of differential privacy algorithms.
* OpenDP has bindings for Python and R, both built on the same Rust core for consistency and security.
* OpenDP is a community effort and is not owned or directed by a single corporation.

That said, OpenDP is not the best tool for every job.
In particular, it is a fairly low-level interface:
There are a number of other projects which try to make it easy to add
differential privacy to existing SQL interfaces or ML frameworks.
One such tool is `SmartNoise SDK <https://github.com/opendp/smartnoise-sdk>`_,
which is built on the OpenDP library.

What next?
----------

There are multiple tracks through the documentation:

* New users of the Python library should begin with :doc:`getting-started/index`.
* For users of other languages, and for more detail, see the :doc:`api/index`.
* If you want to understand how the fundamentals of DP are applied in OpenDP, see :doc:`theory/index`.
* Finally, if you're joining the project, see :doc:`contributing/index`.

.. toctree::
  :hidden:

  getting-started/index
  api/index
  theory/index
  contributing/index
