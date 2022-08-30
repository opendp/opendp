from opendp.transformations import make_subset_by, make_quantiles_from_counts
from opendp.typing import *
from opendp.mod import enable_features

enable_features('contrib')

INT_DATA = list(range(1, 10))
FLOAT_DATA = list(map(float, INT_DATA))
STR_DATA = list(map(str, INT_DATA))


def test_cast_impute():
    from opendp.transformations import make_cast, make_impute_constant
    caster = make_cast(TIA=float, TOA=int) >> make_impute_constant(-1)
    assert caster([1., 2., 3.]) == [1, 2, 3]

    caster = make_cast(TIA=float, TOA=int) >> make_impute_constant(1)
    assert caster([float('nan'), 2.]) == [1, 2]


def test_cast_drop_null():
    from opendp.transformations import make_cast, make_drop_null, make_cast_inherent
    caster = make_cast(TIA=str, TOA=int) >> make_drop_null(DA=OptionNullDomain[AllDomain[int]])
    assert caster(["A", "2", "3"]) == [2, 3]

    caster = make_cast(TIA=float, TOA=int) >> make_drop_null(DA=OptionNullDomain[AllDomain[int]])
    assert caster([float('nan'), 2.]) == [2]

    caster = make_cast_inherent(TIA=str, TOA=float) >> make_drop_null(DA=InherentNullDomain[AllDomain[float]])
    assert caster(["a", "2."]) == [2]
    

def test_cast_inherent():
    from opendp.transformations import make_cast_inherent
    caster = make_cast_inherent(TIA=int, TOA=float)

    assert caster([1, 2]) == [1., 2.]


def test_impute_constant_inherent():
    from opendp.transformations import make_split_lines, make_cast, make_impute_constant
    tester = make_split_lines() >> make_cast(TIA=str, TOA=float) >> make_impute_constant(-1.)
    assert tester("nan\n1.") == [-1., 1.]


def test_cast_default():
    from opendp.transformations import make_cast_default
    caster = make_cast_default(TIA=float, TOA=int)
    assert caster([float('nan'), 2.]) == [0, 2]


def test_impute_uniform():
    from opendp.transformations import make_impute_uniform_float
    caster = make_impute_uniform_float(bounds=(-1., 2.))
    assert -1. <= caster([float('nan')])[0] <= 2.


def test_identity():
    from opendp.transformations import make_identity
    from opendp.typing import VectorDomain, AllDomain
    # test int
    transformation = make_identity(VectorDomain[AllDomain[int]], ChangeOneDistance)
    arg = [123]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(VectorDomain[AllDomain[float]], ChangeOneDistance)
    arg = [123.123]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(VectorDomain[AllDomain[str]], ChangeOneDistance)
    arg = ["hello, world"]
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity("VectorDomain<AllDomain<i32>>", ChangeOneDistance)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == arg


def test_is_equal():
    from opendp.transformations import make_is_equal
    tester = make_is_equal(3)
    assert tester([1, 2, 3]) == [False, False, True]


def test_is_null():
    from opendp.transformations import make_split_lines, make_cast_inherent, make_is_null
    tester = (
        make_split_lines() >>
        make_cast_inherent(TIA=str, TOA=float) >>
        make_is_null(DIA=InherentNullDomain[AllDomain[float]])
    )
    assert tester("nan\n1.\ninf") == [True, False, False]

    from opendp.transformations import make_split_lines, make_cast, make_is_null
    tester = (
        make_split_lines() >>
        make_cast(TIA=str, TOA=float) >>
        make_is_null(DIA=OptionNullDomain[AllDomain[float]])
    )
    assert tester("nan\n1.\ninf") == [True, False, False]


def test_split_lines__cast__impute():
    from opendp.transformations import make_split_lines, make_cast, make_impute_constant
    assert make_split_lines()("1\n2\n3") == ["1", "2", "3"]
    query = (
        make_split_lines() >>
        make_cast(TIA=str, TOA=int) >>
        make_impute_constant(constant=2)
    )

    assert query("1\n2\n3") == [1, 2, 3]
    query.check(1, 1)
    print(query.map(1))


def test_inherent_cast__impute():
    from opendp.transformations import make_split_lines, make_cast_inherent, make_impute_constant
    cast = make_split_lines() >> make_cast_inherent(TIA=str, TOA=float)
    constant = cast >> make_impute_constant(constant=9., DA=InherentNullDomain[AllDomain[float]])

    assert constant("a\n23.23\n12") == [9., 23.23, 12.]
    assert constant.check(1, 1)


def test_inherent_cast__impute_uniform():
    from opendp.transformations import make_split_lines, make_cast_inherent, make_impute_uniform_float
    cast = make_split_lines() >> make_cast_inherent(TIA=str, TOA=float)
    constant = cast >> make_impute_uniform_float(bounds=(23., 32.5))

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
    from opendp.transformations import make_clamp
    query = make_clamp(bounds=(-1, 1))
    assert query([-10, 0, 10]) == [-1, 0, 1]
    assert query.check(1, 1)


def test_bounded_mean():
    from opendp.transformations import make_sized_bounded_mean
    query = make_sized_bounded_mean(size=9, bounds=(0., 10.))
    assert query(FLOAT_DATA) == 5.
    assert query.check(2, 10. / 9. + 1e-6)


def test_bounded_sum():
    from opendp.transformations import make_bounded_sum
    query = make_bounded_sum(bounds=(0., 10.))
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten the check
    assert query.check(1, 20.)

    query = make_bounded_sum(bounds=(0, 10))
    assert query(INT_DATA) == 45
    # TODO: tighten the check
    assert query.check(1, 20)

    try:
        query(FLOAT_DATA)
        raise ValueError("should not accept float data")
    except TypeError:
        pass


def test_sized_bounded_sum():
    from opendp.transformations import make_sized_bounded_sum
    query = make_sized_bounded_sum(size=9, bounds=(0., 10.))
    assert query(FLOAT_DATA) == 45.
    assert query.check(1, 10. + 1e-12)

    query = make_sized_bounded_sum(size=10_000, bounds=(0., 10.))
    assert query.check(1, 10. + 1e-9)

    query = make_sized_bounded_sum(size=100_000, bounds=(0., 10.))
    assert query.check(1, 10. + 1e-8)

    query = make_sized_bounded_sum(size=1_000_000, bounds=(0., 10.))
    assert query.check(1, 10. + 1e-7)

    query = make_sized_bounded_sum(size=10_000_000, bounds=(0., 10.))
    assert query.check(1, 10. + 1e-5)


def test_bounded_variance():
    from opendp.transformations import make_sized_bounded_variance
    query = make_sized_bounded_variance(size=9, bounds=(0., 10.))
    print(query(FLOAT_DATA))
    assert query(FLOAT_DATA) == 7.5
    assert query.check(2, 11.111111 + 1e-6)

def test_count():
    from opendp.transformations import make_count
    transformation = make_count(TIA=int, TO=int)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == 3
    assert transformation.check(1, 1)

def test_count_distinct():
    from opendp.transformations import make_count_distinct
    transformation = make_count_distinct(str, int)
    arg = list(map(str, [1, 2, 3, 2, 7, 3, 4]))
    ret = transformation(arg)
    assert ret == 5
    assert transformation.check(1, 1)


def test_count_by():
    from opendp.transformations import make_count_by
    query = make_count_by(MO=L1Distance[float], TK=str, TV=float)
    assert query(STR_DATA) == {str(i + 1): 1 for i in range(9)}
    print('first')
    assert query.check(1, 2.)


def test_count_by_categories():
    from opendp.transformations import make_count_by_categories
    query = make_count_by_categories(categories=["1", "3", "4"], MO=L1Distance[int])
    assert query(STR_DATA) == [1, 1, 1, 6]
    assert query.check(1, 1)


def test_resize():
    from opendp.transformations import make_bounded_resize
    query = make_bounded_resize(size=4, bounds=(0, 10), constant=0)
    assert sorted(query([-1, 2, 5])) == [-1, 0, 2, 5]
    assert not query.check(1, 1)
    assert query.check(1, 2)
    assert query.check(2, 4)

    from opendp.transformations import make_resize
    query = make_resize(size=4, constant=0)
    assert sorted(query([-1, 2, 5])) == [-1, 0, 2, 5]
    assert not query.check(1, 1)
    assert query.check(1, 2)
    assert query.check(2, 4)


def test_count_by_categories_str():
    from opendp.transformations import make_count_by_categories
    query = make_count_by_categories(categories=["1", "3", "4"], MO=L1Distance[int])
    assert query(STR_DATA) == [1, 1, 1, 6]
    assert query.check(1, 1)


def test_indexing():
    from opendp.transformations import make_find, make_impute_constant, make_find_bin, make_index

    find = make_find(categories=["1", "3", "4"]) >> make_impute_constant(3, DA=OptionNullDomain[AllDomain["usize"]])
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
    from opendp.transformations import make_split_dataframe, make_df_cast_default, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["23", "17"]) >>
        make_df_cast_default(column_name="23", TIA=str, TOA=int) >>
        make_df_cast_default(column_name="23", TIA=int, TOA=bool) >>
        make_select_column(key="23", TOA=bool)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == [False, True, True, True]
    assert query.check(1, 1)


def test_df_is_equal():
    from opendp.transformations import make_split_dataframe, make_df_is_equal, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["23", "17"]) >>
        make_df_is_equal(column_name="17", value="2.") >>
        make_select_column(key="17", TOA=bool)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == [False, False, True, False]
    assert query.check(1, 1)


def test_df_subset():
    from opendp.transformations import make_split_dataframe, make_df_is_equal, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=["A", "B"]) >>
        make_df_is_equal(column_name="B", value="2.") >>
        make_subset_by(indicator_column="B", keep_columns=["A"]) >>
        make_select_column(key="A", TOA=str)
    )
    assert query("0,0.\n1,1.\n2,2.\n3,3.") == ["2"]
    assert query.check(1, 1)

def test_lipschitz_b_ary_tree():
    from opendp.transformations import make_count_by_categories, make_b_ary_tree, make_consistent_b_ary_tree, make_cdf, choose_branching_factor
    from opendp.measurements import make_base_geometric
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
        make_count_by_categories(categories=["A", "B", "C", "D", "E", "F"]) >> 
        tree_builder >> 
        make_base_geometric(1., D=VectorDomain[AllDomain[int]]) >> 
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
