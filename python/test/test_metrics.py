import opendp.prelude as dp


def test_partition_distance():
    domain = dp.vector_domain(dp.atom_domain(T=float))
    metric = dp.partition_distance(dp.symmetric_distance())
    assert metric != str(metric)
    trans = dp.t.make_user_transformation(
        domain, metric, domain, metric,
        function=lambda x: x,
        stability_map=lambda d_in: d_in
    )
    
    assert trans.map((3, 4, 3)) == (3, 4, 3)
