import opendp.prelude as dp



def test_count():
    preprocess = (
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        dp.t.make_select_column("A", TOA=str) >>
        dp.t.then_count()
    )

    noisy_count_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_count_from_dataframe.check(1, 1.)

    k = 40
    data = "\n".join(map(str, range(k)))

    print("TODO: explain: noisy_count_from_dataframe(data)", noisy_count_from_dataframe(data))


def test_count_distinct():
    preprocess = (
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        dp.t.make_select_column("A", TOA=str) >>
        dp.t.then_count_distinct()
    )

    noisy_count_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_count_from_dataframe.check(1, 1.)

    k = 40
    data = "\n".join(map(str, range(k)))

    print("TODO: explain: noisy_count_from_dataframe(data)", noisy_count_from_dataframe(data))


def test_float_count():
    preprocess = (
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        dp.t.make_select_column("A", TOA=str) >>
        dp.t.then_count(TO=float)
    )

    k = 40
    data = "\n".join(map(str, range(k)))

    print("TODO: explain: (preprocess >> dp.m.then_laplace(1.))(data)", (preprocess >> dp.m.then_laplace(1.))(data))
    print("TODO: explain: (preprocess >> dp.m.then_gaussian(1.))(data)", (preprocess >> dp.m.then_gaussian(1.))(data))
