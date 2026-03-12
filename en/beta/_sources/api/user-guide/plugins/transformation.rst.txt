
Transformation example
======================

Use :func:`~opendp.transformations.make_user_transformation` to construct your own transformation.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. code:: pycon

        >>> import opendp.prelude as dp
        >>> dp.enable_features("honest-but-curious")

In this example, we mock the typical API of the OpenDP library to make a transformation that duplicates each record ``multiplicity`` times:

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> import opendp.prelude as dp
        >>> def make_repeat(multiplicity):
        ...     """Constructs a Transformation that duplicates each record `multiplicity` times"""
        ...     def function(arg: list[int]) -> list[int]:
        ...         return arg * multiplicity
        ...     def stability_map(d_in: int) -> int:
        ...         # if a user could influence at most (d_in) records before,
        ...         # they can now influence (d_in * multiplicity) records
        ...         return d_in * multiplicity
        ...     return dp.t.make_user_transformation(
        ...         input_domain=dp.vector_domain(
        ...             dp.atom_domain(T=int)
        ...         ),
        ...         input_metric=dp.symmetric_distance(),
        ...         output_domain=dp.vector_domain(
        ...             dp.atom_domain(T=int)
        ...         ),
        ...         output_metric=dp.symmetric_distance(),
        ...         function=function,
        ...         stability_map=stability_map,
        ...     )
        ...
    
The resulting Transformation may be used interchangeably with those constructed via the library:

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> trans = (
        ...     (
        ...         dp.vector_domain(dp.atom_domain(T=str)),
        ...         dp.symmetric_distance(),
        ...     )
        ...     >> dp.t.then_cast_default(TOA=int)
        ...     >> make_repeat(2)  # our custom transformation
        ...     >> dp.t.then_clamp((1, 2))
        ...     >> dp.t.then_sum()
        ...     >> dp.m.then_laplace(1.0)
        ... )
        >>> release = trans(["0", "1", "2", "3"])
        >>> trans.map(1)  # computes epsilon
        4.0

The code snip may form a basis for you to create your own data transformations, 
and mix them into an OpenDP analysis.
