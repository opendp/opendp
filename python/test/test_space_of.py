import opendp.prelude as dp
from typing import Dict, List

def test_typed_space_of():
    # metric defaults to symmetric_distance on vector_domain
    space = dp.space_of(List[int])  # can also do list[int] if python 3.8+
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    # can specify metric explicitly. If not fully specified, will infer distance type from domain
    space = dp.space_of(List[int], dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    # when type is scalar, domain defaults to atom_domain and metric defaults to absolute_distance
    space = dp.space_of(float)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    # the variable from typing and a string are interchangeable
    space = dp.space_of(int, "DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    # can also pass an actual metric
    space = dp.space_of(int, dp.discrete_distance())
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    space = dp.space_of(Dict[str, int], dp.L1Distance[int])
    assert space == (dp.map_domain(dp.atom_domain(T=dp.String), dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

def test_infer_space_of():
    space = dp.space_of([1, 3], infer=True)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())

    space = dp.space_of([1, 3], infer=True, M=dp.L1Distance)
    assert space == (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.l1_distance(T=dp.i32))

    space = dp.space_of(1.0, infer=True)
    assert space == (dp.atom_domain(T=dp.f64), dp.absolute_distance(T=dp.f64))

    space = dp.space_of(1, infer=True, M=dp.DiscreteDistance)
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())

    space = dp.space_of(1, infer=True, M="DiscreteDistance")
    assert space == (dp.atom_domain(T=dp.i32), dp.discrete_distance())
