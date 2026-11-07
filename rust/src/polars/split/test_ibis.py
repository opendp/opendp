"""Test-only materialization through the actual sibling polars-to-ibis package."""

import io
import json
import pathlib
import re
import sys
import warnings

import ibis
import polars as pl
from polars_to_ibis import convert_polars_to_ibis

root = pathlib.Path(sys.argv[1])
connection = ibis.duckdb.connect()


def scan(node):
    if isinstance(node, dict):
        if "DataFrameScan" in node:
            return node["DataFrameScan"]
        for child in node.values():
            found = scan(child)
            if found is not None:
                return found
    elif isinstance(node, list):
        for child in node:
            found = scan(child)
            if found is not None:
                return found
    return None


for path in sorted(root.glob("head_*.bin")):
    index = int(path.stem.split("_")[1])
    head = pl.LazyFrame.deserialize(path)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        wire = json.loads(head.serialize(format="json"))
    source = pl.DataFrame.deserialize(io.BytesIO(bytes(scan(wire)["df"])))
    connection.create_table("source", source, overwrite=True)
    try:
        query = convert_polars_to_ibis(head, table_name="source", backend=connection)
    except Exception as error:
        error.add_note(f"generated OpenDP case {sys.argv[2]}, head {index}")
        raise
    sql = str(query.to_sql())
    (root / f"sql_{index}.txt").write_text(sql)
    assert not re.search(r"\b(?:ARRAY_AGG|LIST|UNNEST)\s*\(", sql, re.IGNORECASE), sql
    if "discrete_quantile_histogram_plugin" in json.dumps(wire):
        assert re.search(r"\bCOUNT\s*\(\s*\*\s*\)", sql, re.IGNORECASE), sql
        assert len(re.findall(r"\bGROUP\s+BY\b", sql, re.IGNORECASE)) == 1, sql
    result = connection.to_polars(query)
    reference = pl.read_ipc(root / f"reference_{index}.arrow")
    assert result.schema == reference.schema, (sql, result.schema, reference.schema)
    sort = sorted(result.columns)
    assert result.sort(sort).equals(reference.sort(sort)), (sql, result, reference)
    result.write_ipc(root / f"{index}.arrow")

result = pl.LazyFrame.deserialize(root / "continuation.bin").collect()
expected = pl.read_ipc(root / "expected.arrow")
if "key" in result.columns:
    result, expected = result.sort("key"), expected.sort("key")
assert result.equals(expected), (result, expected)
connection.disconnect()
