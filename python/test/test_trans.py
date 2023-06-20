import opendp.prelude as dp
from opendp.transformations import make_subset_by, make_quantiles_from_counts
from opendp.domains import atom_domain, option_domain
from opendp.typing import *
from opendp.mod import enable_features

enable_features('contrib')

INT_DATA = list(range(1, 10))
FLOAT_DATA = list(map(float, INT_DATA))
STR_DATA = list(map(str, INT_DATA))


def test_cast_impute():
    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()
    caster = dp.t.make_cast(*input_space, TOA=int) >> dp.t.make_impute_constant(dp.option_domain(dp.atom_domain(T=int)), -1)
    assert caster([1., 2., 3.]) == [1, 2, 3]

    caster = dp.t.make_cast(*input_space, TOA=int) >> dp.t.make_impute_constant(dp.option_domain(dp.atom_domain(T=int)), 1)
    assert caster([float('nan'), 2.]) == [1, 2]


def test_cast_drop_null():
    input_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    caster = dp.t.make_cast(*input_space, TOA=int) >> dp.t.make_drop_null(dp.option_domain(dp.atom_domain(T=int)))
    assert caster(["A", "2", "3"]) == [2, 3]

    caster = dp.t.make_cast_inherent(*input_space, TOA=float) >> dp.t.make_drop_null(dp.atom_domain(nullable=True, T=float))
    assert caster(["a", "2."]) == [2]

    input_space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()
    caster = dp.t.make_cast(*input_space, TOA=int) >> dp.t.make_drop_null(dp.option_domain(dp.atom_domain(T=int)))
    assert caster([float('nan'), 2.]) == [2]


def test_cast_inherent():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    caster = dp.t.make_cast_inherent(*input_space, TOA=float)

    assert caster([1, 2]) == [1., 2.]


def test_impute_constant_inherent():
    tester = dp.t.make_split_lines() >> dp.t.then_cast_inherent(TOA=float) >> dp.t.make_impute_constant(dp.atom_domain(nullable=True, T=float), -1.)
    assert tester("nan\n1.") == [-1., 1.]


def test_cast_default():
    from opendp.transformations import make_cast_default
    from opendp.domains import vector_domain, atom_domain
    from opendp.metrics import symmetric_distance

    input_domain = vector_domain(atom_domain(T=float))
    input_metric = symmetric_distance()
    caster = make_cast_default(
        input_domain, input_metric, TOA=int)
    assert caster([float('nan'), 2.]) == [0, 2]


def test_impute_uniform():
    from opendp.transformations import make_impute_uniform_float
    caster = make_impute_uniform_float(bounds=(-1., 2.))
    assert -1. <= caster([float('nan')])[0] <= 2.


def test_identity():
    from opendp.transformations import make_identity
    from opendp.typing import VectorDomain, AtomDomain
    # test int
    transformation = make_identity(VectorDomain[AtomDomain[int]], SymmetricDistance)
    arg = [123]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(VectorDomain[AtomDomain[float]], SymmetricDistance)
    arg = [123.123]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(VectorDomain[AtomDomain[str]], SymmetricDistance)
    arg = ["hello, world"]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity("VectorDomain<AtomDomain<i32>>", SymmetricDistance)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == arg


def test_is_equal():
    from opendp.transformations import make_is_equal
    from opendp.domains import vector_domain, atom_domain
    from opendp.metrics import symmetric_distance
    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()
    tester = make_is_equal(input_domain, input_metric, 3)
    assert tester([1, 2, 3]) == [False, False, True]


def test_is_null():
    tester = (
        dp.t.make_split_lines() >>
        dp.t.then_cast_inherent(TOA=float) >>
        dp.t.make_is_null(atom_domain(nullable=True, T=float))
    )
    assert tester("nan\n1.\ninf") == [True, False, False]

    tester = (
        dp.t.make_split_lines() >>
        dp.t.then_cast(TOA=float) >>
        dp.t.make_is_null(option_domain(atom_domain(T=float)))
    )
    assert tester("nan\n1.\ninf") == [True, False, False]


def test_split_lines__cast__impute():
    assert dp.t.make_split_lines()("1\n2\n3") == ["1", "2", "3"]
    query = (
        dp.t.make_split_lines() >>
        dp.t.then_cast(TOA=int) >>
        dp.t.make_impute_constant(dp.option_domain(dp.atom_domain(T=int)), constant=2)
    )

    assert query("1\n2\n3") == [1, 2, 3]
    query.check(1, 1)
    print(query.map(1))


def test_inherent_cast__impute():
    cast = dp.t.make_split_lines() >> dp.t.then_cast_inherent(TOA=float)
    constant = cast >> dp.t.make_impute_constant(dp.atom_domain(nullable=True, T=float), constant=9.)

    assert constant("a\n23.23\n12") == [9., 23.23, 12.]
    assert constant.check(1, 1)

def test_inherent_cast__impute_uniform():
    cast = dp.t.make_split_lines() >> dp.t.then_cast_inherent(TOA=float)
    constant = cast >> dp.t.make_impute_uniform_float(bounds=(23., 32.5))

    res = constant("a\n23.23\n12")
    assert res[1:] == [23.23, 12.]
    assert 23. <= res[0] <= 32.5
    assert constant.check(1, 1)


def test_dataframe_pipeline():
    from opendp.transformations import make_split_lines, make_split_records, \
        make_create_dataframe, make_select_column

    query = (
        make_split_lines() >>
        make_split_records(separator=",") >>
        make_create_dataframe(col_names=["A", "B"]) >>
        make_select_column(key="A", TOA=str)
    )
    assert query("1,1.\n2,2.\n3,3.") == ["1", "2", "3"]
    assert query.check(1, 1)


def test_split_dataframe():
    from opendp.transformations import make_split_dataframe, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["23", "17"]) >>
        make_select_column(key="23", TOA=str)
    )
    assert query("1,1.\n2,2.\n3,3.") == ["1", "2", "3"]
    assert query.check(1, 1)


def test_clamp():
    from opendp.transformations import then_clamp, make_clamp
    from opendp.domains import vector_domain, atom_domain
    from opendp.metrics import symmetric_distance
    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()
    query = (input_domain, input_metric) >> then_clamp(bounds=(-1, 1))
    assert query([-10, 0, 10]) == [-1, 0, 1]
    assert query.check(1, 1)

    query2 = make_clamp(input_domain, input_metric, bounds=(-1, 1))
    assert query2([-10, 0, 10]) == [-1, 0, 1]
    assert query2.check(1, 1)


def test_bounded_mean():
    query = dp.t.make_mean(dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=9), dp.symmetric_distance())
    assert query(FLOAT_DATA) == 5.
    assert query.check(2, 10. / 9. + 1e-6)


def test_bounded_sum():
    import opendp.prelude as dp
    query = dp.t.make_sum(dp.vector_domain(dp.atom_domain(bounds=(0., 10.))), dp.symmetric_distance())
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten the check
    assert query.check(1, 20.)

    query = dp.t.make_sum(dp.vector_domain(dp.atom_domain(bounds=(0, 10))), dp.symmetric_distance())
    assert query(INT_DATA) == 45
    # TODO: tighten the check
    assert query.check(1, 20)

    try:
        query(FLOAT_DATA)
        raise ValueError("should not accept float data")
    except TypeError:
        pass


def test_sized_bounded_sum():
    import opendp.prelude as dp

    domain = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=9)
    metric = dp.symmetric_distance()
    query = (domain, metric) >> dp.t.then_sum()
    assert query(FLOAT_DATA) == 45.
    assert query.check(1, 10. + 1e-12)

    domain = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=10_000)
    query = (domain, metric) >> dp.t.then_sum()
    assert query.check(1, 10. + 1e-9)

    domain = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=100_000)
    query = (domain, metric) >> dp.t.then_sum()
    assert query.check(1, 10. + 1e-8)

    domain = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=1_000_000)
    query = (domain, metric) >> dp.t.then_sum()
    assert query.check(1, 10. + 1e-7)

    domain = dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=10_000_000)
    query = (domain, metric) >> dp.t.then_sum()
    assert query.check(1, 10. + 1e-5)


def test_bounded_variance():
    query = dp.t.make_variance(
        dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=9),
        dp.symmetric_distance())
    print(query(FLOAT_DATA))
    assert query(FLOAT_DATA) == 7.5
    assert query.check(2, 11.111111 + 1e-6)

def test_sum_of_squared_deviances():
    query = dp.t.make_sum_of_squared_deviations(
        dp.vector_domain(dp.atom_domain(bounds=(0., 10.)), size=9),
        dp.symmetric_distance())
    assert query(FLOAT_DATA) == 60.0
    assert query.check(2, 88.888888 + 1e-4)

def test_count():
    transformation = dp.t.make_count(dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == 3
    assert transformation.check(1, 1)

def test_count_distinct():
    transformation = dp.t.make_count_distinct(dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance())
    arg = list(map(str, [1, 2, 3, 2, 7, 3, 4]))
    ret = transformation(arg)
    assert ret == 5
    assert transformation.check(1, 1)


def test_count_by():
    input_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    query = input_space >> dp.t.then_count_by(MO=L1Distance[float], TV=float)
    assert query(STR_DATA) == {str(i + 1): 1 for i in range(9)}
    assert query.check(1, 2.)


def test_count_by_categories():
    input_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    query = dp.t.make_count_by_categories(*input_space, categories=["1", "3", "4"], MO=L1Distance[int])
    assert query(STR_DATA) == [1, 1, 1, 6]
    assert query.check(1, 1)


def test_resize():
    input_space = dp.vector_domain(dp.atom_domain(bounds=(0, 10))), dp.symmetric_distance()
    query = dp.t.make_resize(*input_space, size=4, constant=0)
    assert sorted(query([-1, 2, 5])) == [-1, 0, 2, 5]
    assert not query.check(1, 1)
    assert query.check(1, 2)
    assert query.check(2, 4)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    query = dp.t.make_resize(*input_space, size=4, constant=0)
    assert sorted(query([-1, 2, 5])) == [-1, 0, 2, 5]
    assert not query.check(1, 1)
    assert query.check(1, 2)
    assert query.check(2, 4)


def test_indexing():
    from opendp.transformations import make_find, make_impute_constant, make_find_bin, make_index

    find = make_find(categories=["1", "3", "4"]) >> make_impute_constant(option_domain(atom_domain(T=usize)), 3)
    assert find(STR_DATA) == [0, 3, 1, 2, 3, 3, 3, 3, 3]
    assert find.check(1, 1)

    binner = make_find_bin(edges=[2, 3, 5])
    assert binner(INT_DATA) == [0, 1, 2, 2, 3, 3, 3, 3, 3]

    indexer = make_index(categories=["A", "B", "C"], null="NA")
    assert indexer([0, 1, 3, 1, 5]) == ['A', 'B', 'NA', 'B', 'NA']

    assert (find >> indexer)(STR_DATA) == ['A', 'NA', 'B', 'C', 'NA', 'NA', 'NA', 'NA', 'NA']
    assert (binner >> indexer)(INT_DATA) == ['A', 'B', 'C', 'C', 'NA', 'NA', 'NA', 'NA', 'NA']


def test_lipschitz_mul_float():
    from opendp.transformations import make_lipschitz_float_mul, make_sized_bounded_float_ordered_sum
    trans = make_sized_bounded_float_ordered_sum(10, (0., 10.)) >> make_lipschitz_float_mul(1 / 10, (-3., 4.))

    print(trans([3.] * 10))
    print(trans.map(2))


def test_df_cast_default():
    from opendp.transformations import make_split_dataframe, then_df_cast_default, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["23", "17"]) >>
        then_df_cast_default(column_name="23", TIA=str, TOA=int) >>
        then_df_cast_default(column_name="23", TIA=int, TOA=bool) >>
        make_select_column(key="23", TOA=bool)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == [False, True, True, True]
    assert query.check(1, 1)


def test_df_is_equal():
    from opendp.transformations import make_split_dataframe, then_df_is_equal, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["23", "17"]) >>
        then_df_is_equal(column_name="17", value="2.") >>
        make_select_column(key="17", TOA=bool)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == [False, False, True, False]
    assert query.check(1, 1)


def test_df_subset():
    from opendp.transformations import make_split_dataframe, then_df_is_equal, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["A", "B"]) >>
        then_df_is_equal(column_name="B", value="2.") >>
        make_subset_by(indicator_column="B", keep_columns=["A"]) >>
        make_select_column(key="A", TOA=str)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == ["2"]
    assert query.check(1, 1)

def test_lipschitz_b_ary_tree():
    from opendp.transformations import then_count_by_categories, make_b_ary_tree, make_consistent_b_ary_tree, make_cdf, choose_branching_factor
    from opendp.measurements import then_base_geometric
    leaf_count = 7
    branching_factor = 2
    tree_builder = make_b_ary_tree(leaf_count, branching_factor, M=L1Distance[int])
    assert tree_builder([1] * leaf_count) == [7, 4, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1]
    #                                  level: 1  2     3           4
    # top of tree is at level 1

    suggested_factor = choose_branching_factor(size_guess=10_000)
    print("suggested_factor", suggested_factor)

    # the categories are bin names!
    meas_base = (
        (dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()) >>
        then_count_by_categories(categories=["A", "B", "C", "D", "E", "F"]) >> 
        tree_builder >> 
        then_base_geometric(1.) >> 
        make_consistent_b_ary_tree(branching_factor)
    )

    meas_cdf = meas_base >> make_cdf()
    meas_quantiles = meas_base >> make_quantiles_from_counts(
        bin_edges=[0., 10., 13., 17., 26., 70., 84., 100.],
        alphas=[0., .1, .2, .3, .4, .5, .6, .7, .8, .9, 1.])

    data = ["A"] * 34 + ["B"] * 23 + ["C"] * 12 + ["D"] * 84 + ["E"] * 34 + ["F"] * 85 + ["G"] * 75
    print(meas_cdf(data))
    print(meas_quantiles(data))

    assert meas_cdf.map(1) == 4.
