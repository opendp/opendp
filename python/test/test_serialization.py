import json

import opendp.prelude as dp

dp.enable_features('serialization')


def test_noisy_sum():
    sum_trans = dp.t.make_sum(
        dp.vector_domain(dp.atom_domain(bounds=(0, 1))),
        dp.symmetric_distance()
    )
    laplace = dp.m.make_laplace(
        sum_trans.output_domain,
        sum_trans.output_metric,
        scale=1.0
    )
    noisy_sum = sum_trans >> laplace
    noisy_sum_str = str(noisy_sum)
    noisy_sum_json = noisy_sum.to_json()


def test_old_dataframe_api():
    preprocessor = (
        # load data into a dataframe where columns are of type Vec<str>
        dp.t.make_split_dataframe(separator=",", col_names=["hello", "world"])
        >>
        # select a column of the dataframe
        dp.t.make_select_column(key="income", TOA=str)
    )

    expected_str = '''Transformation(
    input_domain   = AtomDomain(T=String),
    output_domain  = VectorDomain(AtomDomain(T=String)),
    input_metric   = SymmetricDistance(),
    output_metric  = SymmetricDistance())'''

    assert str(preprocessor) == expected_str

    # serialize the chain to json
    json_str = preprocessor.to_json(indent=2)
    json_dict = json.loads(json_str)
    assert json_dict == {
        "ast": {
            "_type": "constructor",
            "func": "make_chain_tt",
            "module": "combinators",
            "args": [
                {
                    "_type": "constructor",
                    "func": "make_select_column",
                    "module": "transformations",
                    "kwargs": {
                        "key": "income",
                        "TOA": "String"
                    }
                },
                {
                    "_type": "constructor",
                    "func": "make_split_dataframe",
                    "module": "transformations",
                    "kwargs": {
                        "separator": ",",
                        "col_names": {
                            "_type": "list",
                            "_items": [
                                "hello",
                                "world"
                            ]
                        }
                    }
                }
            ]
        }
    }

    # reconstruct the obj from the json string
    assert str(dp.make_load_json(json_str)) == expected_str