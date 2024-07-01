import opendp.prelude as dp

dp.enable_features("contrib", "honest-but-curious")


def test_partition_distance():
    domain = dp.vector_domain(dp.atom_domain(T=float))
    metric = dp.partition_distance(dp.symmetric_distance())
    def stability_map(d_in):
        print(d_in)
        return d_in
    trans = dp.t.make_user_transformation(
        domain, metric, domain, metric,
        function=lambda x: x,
        stability_map=stability_map
    )
    
    assert trans.map((3, 4, 3)) == (3, 4, 3)