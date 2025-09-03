Statistical Modeling
====================

Beyond the basic statistics offered by the Polars interface,
OpenDP offers differentially private versions of **linear regression** and **PCA**,
with APIs modelled after :py:mod:`sklearn <opendp.extras.sklearn>`.

Another more flexible option is to use OpenDP to create **synthetic data**.
Statistics calculated from synthetic data with a given privacy budget
will be less accurate than a tailored DP release using the same budget,
but if you are unsure what analyses will be needed,
or you need to provide tabular data so that downstream consumers can make their own analyses,
this may be a good option.

.. toctree::
  :maxdepth: 1

  pca
  synthetic-data
