:orphan:

.. code:: pycon

    # enable-features
    >>> import opendp.prelude as dp
    >>> dp.enable_features("honest-but-curious", "contrib")

    # /enable-features

    # make-repeat
    >>> def make_repeat(multiplicity):
    ...     """Construct a Transformation that duplicates each record `multiplicity` times."""
    ...     def function(arg: list[int]) -> list[int]:
    ...         return arg * multiplicity
    ...     def stability_map(d_in: int) -> int:
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

    # /make-repeat

    # use-transformation
    >>> trans = (
    ...     (
    ...         dp.vector_domain(dp.atom_domain(T=str)),
    ...         dp.symmetric_distance(),
    ...     )
    ...     >> dp.t.then_cast_default(TOA=int)
    ...     >> make_repeat(2)
    ...     >> dp.t.then_clamp((1, 2))
    ...     >> dp.t.then_sum()
    ...     >> dp.m.then_laplace(1.0)
    ... )
    >>> release = trans(["0", "1", "2", "3"])
    >>> trans.map(1)
    4.0

    # /use-transformation
