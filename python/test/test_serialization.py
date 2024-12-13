import json

import opendp.prelude as dp

dp.enable_features('serialization')

def test_serialization():
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