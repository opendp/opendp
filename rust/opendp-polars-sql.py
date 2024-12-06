import tomlkit
from pathlib import Path
import os
import requests
import tarfile
import shutil

os.chdir(os.path.dirname(os.path.abspath(__file__)))

shutil.rmtree("opendp-polars-sql", ignore_errors=True)

version = "0.44.2"
url = f"https://crates.io/api/v1/crates/polars-sql/{version}/download"

with requests.get(url, stream=True) as res, tarfile.open(fileobj=res.raw, mode="r") as tarobj:
    tarobj.extractall(filter="tar")

os.rename(f"polars-sql-{version}", "opendp-polars-sql")

# update Cargo.toml
with open("opendp-polars-sql/Cargo.toml", "r") as cargo_toml_file:
    cargo_toml = tomlkit.load(cargo_toml_file)

cargo_toml["package"]["name"] = "opendp-polars-sql"
cargo_toml["package"]["authors"] = ["The OpenDP Project <info@opendp.org>"]
cargo_toml["package"]["repository"] = "https://github.com/opendp/polars-sql"
cargo_toml["lib"]["name"] = "opendp_polars_sql"

with open("opendp-polars-sql/Cargo.toml", "w") as cargo_toml_file:
    tomlkit.dump(cargo_toml, cargo_toml_file)


def replace(path, old, new):
    source_path = Path(path)
    source = source_path.read_text()
    source_path.write_text(source.replace(old, new))


def replace_all(directory, old, new):
    for path, _, files in os.walk(os.path.abspath(directory)):
        for filename in files:
            if not filename.endswith(".rs"):
                continue
            filepath = os.path.join(path, filename)
            replace(filepath, old, new)

# patch the package
old_text = """
        self.ctx
            .function_registry
            .get_udf(func_name)?
            .ok_or_else(|| polars_err!(SQLInterface: "UDF {} not found", func_name))?
            .call(args)"""

new_text = """
        Ok(self.ctx
            .function_registry
            .get_udf(func_name)?
            .ok_or_else(|| polars_err!(SQLInterface: "UDF {} not found", func_name))?
            .call_unchecked(args))"""

replace("opendp-polars-sql/src/functions.rs", old_text, new_text)

# change crate name
replace_all("opendp-polars-sql/src", "polars_sql", "opendp_polars_sql")
replace_all("opendp-polars-sql/tests", "polars_sql", "opendp_polars_sql")

old_text = "# polars-sql"
new_text = """# opendp-polars-sql

This is a fork of [polars-sql](https://crates.io/crates/polars-sql) with a patch to avoid schema checks in UDFs, 
[reported in Polars issue #15159](https://github.com/pola-rs/polars/pull/15159).

# polars-sql
"""
replace("opendp-polars-sql/README.md", old_text, new_text)
