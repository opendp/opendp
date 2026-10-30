import opendp.prelude as dp

dp.enable_features("contrib", "honest-but-curious")

# used for testing a successful release


def main():
    identity = dp.t.make_identity(
        domain=dp.vector_domain(dp.atom_domain(T=str)), metric=dp.symmetric_distance()
    )
    print(identity(["hello, world!"]))

    space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    noisy_sum = (
        space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=1.0)
    )
    noisy_count = space >> dp.t.then_count() >> dp.m.then_laplace(scale=1.0)
    everything = dp.c.make_composition([noisy_sum, noisy_count])
    print(everything([1, 2, 3]))


if __name__ == "__main__":
    main()
