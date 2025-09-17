These snippets are built into the reference API documentation and run as tests, although the path is indirect:

## For Python or R:

- From the `rust` directory, runs `cargo build --all-features` to rebuild generated code.
(`cargo build --features bindings` should work but currently fails. See https://github.com/opendp/opendp/issues/1343.)

## For Python:

- `python.rs` generates Python source code, and if a function name matches a file name with an `.rst` extension here, it will be included as part of the docstring. (Note that if there's a typo in the snippet filename the file will be silently ignored.)
- All Python tests are run by a single invocation of pytest. The `.pytest.ini` file configures two ways that doctests are run:
  - Doctests in Python source, including the generated sources which include these snippets, are run by `--doctest-modules`
  - Doctests in `.rst` files under `docs/` (but not `.rst` files under `rust/`!) are run by `--doctest-glob '*.rst'`
- Globals for the doctests are defined by `python/conftest.py`.

When debugging tests, rather than going through a full build of Python sources, I've found it easier to just use global search and replace to make edits to the snippet and the generated python at the same time.

## For R:

- `r.rs` generates R source code, and if a function name matches a file name with an `.R` extension, the snippet will be included in the generated source code.
- For now, the `then_` function docs use the snippet of the corresponding `make_` function.
- `tools/r_stage.sh -d` will rebuild the R docs from the previously generated R source code.
- In contrast to Python doctests, in general there are no assertions on expected output in the snippets themselves: Instead, by default, the R docs build will run examples, and include output in generated documentation, but fail if an example has an error.
