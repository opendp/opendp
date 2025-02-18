import json
import re

import pytest

import opendp.prelude as dp
from opendp._lib import import_optional_dependency

atom = dp.atom_domain(bounds=(0., 10.))
input_space = dp.vector_domain(atom, size=10), dp.symmetric_distance()
chained = input_space >> dp.t.then_mean() >> dp.m.then_laplace(scale=0.5)


@pytest.mark.parametrize(
    "_readable_name,dp_obj",
    [
        (str(obj), obj)
        for obj in [
            # Python objects:
            ('nested', ('tuple', ('containing', ('domain', (atom,))))),
            {'dict key': atom},
            input_space,
            # Domains:
            atom,
            dp.categorical_domain(['A', 'B', 'C']),
            dp.series_domain('A', atom),
            dp.lazyframe_domain([dp.series_domain('A', atom)]),
            dp.wild_expr_domain([]),
            # Metrics:
            dp.absolute_distance("int"),
            dp.change_one_distance(),
            dp.linf_distance("float", True),
            dp.user_distance("user_distance"),
            # Measures:
            dp.m.max_divergence(),
            dp.m.approximate(dp.m.max_divergence()),
            dp.m.user_divergence("user_divergence"),
            # Measurements:
            dp.m.make_gaussian(atom, dp.absolute_distance(float), 1),
            dp.m.then_gaussian(1),
            # Compositions:
            chained,
            dp.c.make_population_amplification(chained, population_size=100),
        ]
    ],
)
def test_serializable(_readable_name, dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    # We don't want to define __eq__ just for the sake of testing,
    # so check the serializations before and after.
    # (We should remember that if the first serialization
    #  dropped some detail, this test wouldn't catch it.)
    assert serialized == dp.serialize(deserialized)


@pytest.mark.parametrize(
    "_readable_name,dp_obj",
    [
        (str(obj), obj)
        for obj in [
            dp.user_domain("trivial_user_domain", lambda: True),
            dp.m.new_privacy_profile(lambda x: x),
        ]
    ],
)
def test_not_ever_serializable(_readable_name, dp_obj):
    with pytest.raises(Exception, match=r"OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)


@pytest.mark.parametrize(
    "_readable_name,dp_obj",
    [
        (str(obj), obj)
        for obj in [
            {('tuple', 'key'): 'value'},
        ]
    ],
)
def test_not_json_serializable(_readable_name, dp_obj):
    with pytest.raises(Exception, match=r"keys must be str, int, float, bool or None, not tuple"):
        dp.serialize(dp_obj)


def test_version_mismatch_warning():
    bad_serialized = json.dumps({
        "__function__": "atom_domain",
        "__module__": "domains",
        "__kwargs__": {
            "bounds": {"__tuple__": [0, 10]}, 
            "nullable": False, 
            "T": "i32"
        },
        "__version__": "bad-version"
    })
    with pytest.warns(UserWarning, match=re.escape('(bad-version) != this version')):
        dp.deserialize(bad_serialized)



# Would normally put the conditional inside the test,
# but since we need polars at test collection time,
# in this case it needs to wrap the tests.
pl = import_optional_dependency('polars', raise_error=False)
if pl is not None:
    lf = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
    lf_domain = dp.lazyframe_domain([
        dp.series_domain("A", dp.atom_domain(T="i32")), 
        dp.series_domain("B", dp.atom_domain(T=str))
    ])
    lf_domain_with_margin = dp.with_margin(lf_domain, by=[], max_partition_length=1000)

    context = dp.Context.compositor(
        data=pl.LazyFrame({"age": [1, 2, 3]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=10,
    )
    query = context.query().select(dp.len())

    @pytest.mark.parametrize(
        "_readable_name,dp_obj",
        [
            (str(obj), obj)
            for obj in [
                lf_domain,
                lf_domain_with_margin,
                dp.m.make_private_lazyframe(
                    lf_domain_with_margin,
                    dp.symmetric_distance(),
                    dp.max_divergence(),
                    lf.select([dp.len(), pl.col("A").dp.sum((0, 1))]),
                    global_scale=1.0
                ),
                dp.m.make_private_expr(
                    dp.wild_expr_domain([], by=[]),
                    dp.partition_distance(dp.symmetric_distance()),
                    dp.max_divergence(),
                    dp.len(scale=1.0)
                ),
            ]
        ],
    )
    def test_serializable_polars(_readable_name, dp_obj):
        serialized = dp.serialize(dp_obj)
        deserialized = dp.deserialize(serialized)
        assert serialized == dp.serialize(deserialized)


    @pytest.mark.parametrize(
        "_readable_name,dp_obj",
        [
            (str(obj), obj)
            for obj in [
                context,
                dp.Queryable('value', 'query_type'),
                query
            ]
        ],
    )
    def test_not_currently_serializable(_readable_name, dp_obj):
        with pytest.raises(Exception, match=r"OpenDP JSON Encoder currently does not handle"):
            dp.serialize(dp_obj)
