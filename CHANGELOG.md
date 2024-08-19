# OpenDP Changelog

This file documents the version history of OpenDP. The links on each version number will take you to a comparison
showing the source changes from the previous version.



## [0.11.1](https://github.com/opendp/opendp/compare/v0.11.0...HEAD) - TBD

### Added

- Randomized response bitvec [#1279](https://github.com/opendp/opendp/pull/1279)
- Randomized Response bitvec ffi [#1680](https://github.com/opendp/opendp/pull/1680)
- Polars:
    - Auto-calibrate noise scale in bounded-dp [#1943](https://github.com/opendp/opendp/pull/1943)
    - Rename accuracy to summarize [#1942](https://github.com/opendp/opendp/pull/1942)
    - Support means in accuracy utility [#1922](https://github.com/opendp/opendp/pull/1922)
- A script to help create changelog [#1672](https://github.com/opendp/opendp/pull/1672)
- Proof for `make_randomized_response` [#1315](https://github.com/opendp/opendp/pull/1315)


### Docs

- Computing fundamental statistics notebook [#1834](https://github.com/opendp/opendp/pull/1834)
- Polars groupby [#1836](https://github.com/opendp/opendp/pull/1836)
- Pointer to list of features from api docs [#1931](https://github.com/opendp/opendp/pull/1931)
- Documentation for python vs r cargo build [#1927](https://github.com/opendp/opendp/pull/1927)
- Js redirect for getting-started [#1935](https://github.com/opendp/opendp/pull/1935)
- Logo on readme [#1936](https://github.com/opendp/opendp/pull/1936)
- Link to the new learning resources list, and to hands-on differential privacy [#1896](https://github.com/opendp/opendp/pull/1896)
- Switch typical-workflow from old dataframe methods to just a vector of numbers [#1909](https://github.com/opendp/opendp/pull/1909)
- Copy existing code-of-conduct [#1905](https://github.com/opendp/opendp/pull/1905)
- Clean up old maintainer notes [#1879](https://github.com/opendp/opendp/pull/1879)
- Remove unused todo files in docs [#1907](https://github.com/opendp/opendp/pull/1907)

### Change

- Don't mask predicate error in binary search [#1956](https://github.com/opendp/opendp/pull/1956)

### Fix

- Nightly docs.rs build fails on shadowed generics [#1947](https://github.com/opendp/opendp/pull/1947)
- Infinite loop in data download [#1945](https://github.com/opendp/opendp/pull/1945)
- Fill in rust error message templates [#1917](https://github.com/opendp/opendp/pull/1917)
- `assert_feature` check [#1933](https://github.com/opendp/opendp/pull/1933)
- `master` to `main` in two notebooks [#1915](https://github.com/opendp/opendp/pull/1915)



## [0.11.0](https://github.com/opendp/opendp/compare/v0.10.0...v0.11.0) - 2024-08-13

### Fixed

- Fix Windows install problem [#1779](https://github.com/opendp/opendp/pull/1779)
- R: Fix null pointer error [#1820](https://github.com/opendp/opendp/pull/1820)
- Fix Python freeze: Don't allow iteration over base classes [#1823](https://github.com/opendp/opendp/pull/1823)
- R: Fix docs build by using `stable` [#1894](https://github.com/opendp/opendp/pull/1894)

- URLs:
    - Update urls in docs to reflect new structure [#1692](https://github.com/opendp/opendp/pull/1692)
    - Links in pums-data-analysis [#1729](https://github.com/opendp/opendp/pull/1729)

### Added

- Polars:
    - Opendp_lib_path -> opendp_polars_lib_path [#1839](https://github.com/opendp/opendp/pull/1839)
    - Test pickle bombs [#1841](https://github.com/opendp/opendp/pull/1841)
    - Shuffle onceframe output [#1827](https://github.com/opendp/opendp/pull/1827)
    - Evenly allocate budget amongst queries, by default [#1825](https://github.com/opendp/opendp/pull/1825)
    - Nit error messages [#1824](https://github.com/opendp/opendp/pull/1824)
    - Emulate polars apis for type annotations [#1786](https://github.com/opendp/opendp/pull/1786)
    - Rewrite binary paths in polars plugins [#1785](https://github.com/opendp/opendp/pull/1785)
    - No kwargs in public apis [#1767](https://github.com/opendp/opendp/pull/1767)
    - Laplace thresholding for private key-sets [#1716](https://github.com/opendp/opendp/pull/1716)
    - Utility estimation [#1760](https://github.com/opendp/opendp/pull/1760)
    - Update Polars dependency to 1.1.0 [#1703](https://github.com/opendp/opendp/pull/1703)
    - Enable cutqcut [#1883](https://github.com/opendp/opendp/pull/1883)

- Testing:
    - Improved test coverage in generated code [#1656](https://github.com/opendp/opendp/pull/1656)
    - Improved test coverage in `typing.py` [#1652](https://github.com/opendp/opendp/pull/1652)
    - Treat test warnings as errors [#1727](https://github.com/opendp/opendp/pull/1727)

- Docs:
    - Noise mechanism comparison [#1735](https://github.com/opendp/opendp/pull/1735)
    - Features and env vars [#1815](https://github.com/opendp/opendp/pull/1815)
    - Revise tabular data index.rst [#1864](https://github.com/opendp/opendp/pull/1864)
    - Stub out getting-started/utility [#1797](https://github.com/opendp/opendp/pull/1797)
    - getting-started/stats [#1799](https://github.com/opendp/opendp/pull/1799)
    - Added dependencies for polars notebooks [#1855](https://github.com/opendp/opendp/pull/1855)
    - Partitioning with known and unknown lengths [#1845](https://github.com/opendp/opendp/pull/1845)
    - Add examples in Polars API docs [#1688](https://github.com/opendp/opendp/pull/1688)
    - Tabular data index.rst [#1812](https://github.com/opendp/opendp/pull/1812)
    - Plugin example with regression [#1770](https://github.com/opendp/opendp/pull/1770)

- misc:
    - `with_privacy` now passes `args` and `kwargs` [#1784](https://github.com/opendp/opendp/pull/1784)
    - More informative error message if python and rust polars versions don't match [#1695](https://github.com/opendp/opendp/pull/1695)
    - Include directory in error message if binary not found [#1886](https://github.com/opendp/opendp/pull/1886)

### Changed

- Typing:
    - Add release dtypes to ffi for atomdomain and optiondomain [#1705](https://github.com/opendp/opendp/pull/1705)
    - Remove `unknowntype` and instead immediately raise exception [#1694](https://github.com/opendp/opendp/pull/1694)
    - Cast int to float if needed in `py_to_c` [#1509](https://github.com/opendp/opendp/pull/1509)

- Introduce `extras` namespace
    - Rename `_extrinsics` to `extras` [#1452](https://github.com/opendp/opendp/pull/1452)
    - `dp.np_array2_domain` -> `dp.numpy.array2_domain` [#1861](https://github.com/opendp/opendp/pull/1861)
    - `polars` to `extras` [#1844](https://github.com/opendp/opendp/pull/1844)
    - Follow sklearn namespace precedents: `extras.sklearn.decomposition.PCA` [#1866](https://github.com/opendp/opendp/pull/1866)

- Docs:
    - On docs homepage, replace youtube link with Swiss FSO blog post [#1696](https://github.com/opendp/opendp/pull/1696)
    - Standardize notation for neighboring data sets [#1319](https://github.com/opendp/opendp/pull/1319)
    - Document API layers [#1800](https://github.com/opendp/opendp/pull/1800)
    - Update release process notes [#1817](https://github.com/opendp/opendp/pull/1817)
    - Notebooks without images or wget to RST [#1700](https://github.com/opendp/opendp/pull/1700)
    - Improved explanation of `honest-but-curious` [#1810](https://github.com/opendp/opendp/pull/1810),  [#1730](https://github.com/opendp/opendp/pull/1730)
    - Remove epsilon/delta units in docs [#1822](https://github.com/opendp/opendp/pull/1822)

Testing:
    - Enable all features at start of test run [#1858](https://github.com/opendp/opendp/pull/1858)

### Removed

- Remove references to Binder [#1877](https://github.com/opendp/opendp/pull/1877)
- Remove old dataframe docs from API User Guide [#1875](https://github.com/opendp/opendp/pull/1875)
- gitignore the generated bindings [#1430](https://github.com/opendp/opendp/pull/1430)
    - Update bindings in release channels [#1856](https://github.com/opendp/opendp/pull/1856)
    - Build python bindings before weeks-doc-check [#1847](https://github.com/opendp/opendp/pull/1847)



## [0.10.0](https://github.com/opendp/opendp/compare/v0.9.2...v0.10.0) - 2024-06-20

### Added

- Polars:
    - add `make_private_quantile_expr` [#908](https://github.com/opendp/opendp/pull/908)
    - bounded-DP mean via postprocessing [#890](https://github.com/opendp/opendp/pull/890)
    - add `make_expr_laplace` [#829](https://github.com/opendp/opendp/pull/829)
    - add `make_expr_sum` [#819](https://github.com/opendp/opendp/pull/819)
    - add `make_expr_clip` [#868](https://github.com/opendp/opendp/pull/868)
    - add `make_private_aggregate` [#847](https://github.com/opendp/opendp/pull/847)
    - initial LazyFrame and Expr parsers [#1454](https://github.com/opendp/opendp/pull/1454)
    - add `ExprDomain` [#795](https://github.com/opendp/opendp/pull/795)
    - `lazyframe_domain` ffi [#769](https://github.com/opendp/opendp/pull/769)
    - `series_domain` ffi [#767](https://github.com/opendp/opendp/pull/767)
    - add `FrameDomain` [#765](https://github.com/opendp/opendp/pull/765)
    - add `SeriesDomain` [#763](https://github.com/opendp/opendp/pull/763)
    - add `make_expr_col` [#797](https://github.com/opendp/opendp/pull/797)
    - add `OnceFrame` [#1602](https://github.com/opendp/opendp/pull/1602)
    - And context api integration [#1604](https://github.com/opendp/opendp/pull/1604)
    - Separate public module [#1597](https://github.com/opendp/opendp/pull/1597)
    - Add gaussian, generalize laplace [#1596](https://github.com/opendp/opendp/pull/1596)
    - Alias [#1591](https://github.com/opendp/opendp/pull/1591)
    - Add horizontal stack [#1676](https://github.com/opendp/opendp/pull/1676)
    - Filter [#1534](https://github.com/opendp/opendp/pull/1534)
    - Boolean binary operations [#1600](https://github.com/opendp/opendp/pull/1600)
    - Select measurement [#1539](https://github.com/opendp/opendp/pull/1539)
    - Length transformation [#1540](https://github.com/opendp/opendp/pull/1540)
    - Boolean functions [#1586](https://github.com/opendp/opendp/pull/1586)
    - Fill_nan [#1590](https://github.com/opendp/opendp/pull/1590)
    - Fill_null [#1584](https://github.com/opendp/opendp/pull/1584)
    - Literal transformation [#1542](https://github.com/opendp/opendp/pull/1542)
    - Quantile postprocessing [#1498](https://github.com/opendp/opendp/pull/1498)
    - Add `make_expr_report_noisy_max_gumbel` [#1479](https://github.com/opendp/opendp/pull/1479)
    - Onceframe [#1602](https://github.com/opendp/opendp/pull/1602)
- Usability:
    - Steer users in the right direction if they try to call a domain descriptor [#1512](https://github.com/opendp/opendp/pull/1512)
    - Warn if large priv loss [#1457](https://github.com/opendp/opendp/pull/1457)
    - xfail usability tests [#1465](https://github.com/opendp/opendp/pull/1465)
    - Measure `__str__` -> `__repr__` [#1401](https://github.com/opendp/opendp/pull/1401)
    - Runtime error if non-function is passed to `new_function` [#1355](https://github.com/opendp/opendp/pull/1355)
    - Error if missing `arg` param on measurement in R [#1559](https://github.com/opendp/opendp/pull/1559)
    - Specialized message for mismatched domain [#1511](https://github.com/opendp/opendp/pull/1511)
- Python testing, linting, and typing:
    - Call mypy and flake8 as subprocesses from pytest [#1359](https://github.com/opendp/opendp/pull/1359)
    - More Python typing on context module [#1472](https://github.com/opendp/opendp/pull/1472)
    - Require explicit imports [#1220](https://github.com/opendp/opendp/pull/1220)
    - Use isinstance where appropriate [#1221](https://github.com/opendp/opendp/pull/1221)
    - Fix unneeded f-strings [#1217](https://github.com/opendp/opendp/pull/1217)
    - Leave only the ignores that we actually need [#1219](https://github.com/opendp/opendp/pull/1219)
    - No more bare `except` [#1303](https://github.com/opendp/opendp/pull/1303)
    - Fix masked mypy errors [#1265](https://github.com/opendp/opendp/pull/1265)
    - Fix the flake8 warnings that really need it [#1261](https://github.com/opendp/opendp/pull/1261)
    - Remove `Any` from generated python [#1507](https://github.com/opendp/opendp/pull/1507)
    - Python typing: float implies int [#1486](https://github.com/opendp/opendp/pull/1486)
    - Fix return signature on `loss_of` [#1524](https://github.com/opendp/opendp/pull/1524)
    - Improved test coverage and typing:
        - `_convert.py` [#1645](https://github.com/opendp/opendp/pull/1645)
        - `_lib.py` [#1646](https://github.com/opendp/opendp/pull/1646)
        - `context.py` [#1648](https://github.com/opendp/opendp/pull/1648)
        - `mod.py` [#1649](https://github.com/opendp/opendp/pull/1649)
    - Confirm python coverage ran [#1633](https://github.com/opendp/opendp/pull/1633)
- Sphinx docs and examples:
    - Grouping columns example [#1508](https://github.com/opendp/opendp/pull/1508)
    - Python measurement examples [#1550](https://github.com/opendp/opendp/pull/1550)
    - Use `import opendp.prelude as dp` in docs [#1442](https://github.com/opendp/opendp/pull/1442)
    - Add link to Reference page [#1297](https://github.com/opendp/opendp/pull/1297)
    - Python and R examples in tabs on quickstart (just CI) [#1262](https://github.com/opendp/opendp/pull/1262)
    - link to language specific docs [#1516](https://github.com/opendp/opendp/pull/1516)
    - Python example in API docs proposal [#1439](https://github.com/opendp/opendp/pull/1439)
    - links between user guide and api reference [#1458](https://github.com/opendp/opendp/pull/1458)
    - Update introductory paragraph, remove last outdated section [#1446](https://github.com/opendp/opendp/pull/1446)
    - API ToC just down to modules [#1447](https://github.com/opendp/opendp/pull/1447)
    - Enhance context docs [#1386](https://github.com/opendp/opendp/pull/1386)
    - parallel directory for ancillary doc files [#1371](https://github.com/opendp/opendp/pull/1371)
    - use sphinx-design; avoid raw html [#1351](https://github.com/opendp/opendp/pull/1351)
    - List production applications [#1352](https://github.com/opendp/opendp/pull/1352)
    - Update 404 template [#1354](https://github.com/opendp/opendp/pull/1354)
    - Documentation reorg [#1177](https://github.com/opendp/opendp/pull/1177)
    - Use appropriate shell syntax in notebook examples [#1406](https://github.com/opendp/opendp/pull/1406)
    - In Python API docs, use the same examples for `make` and `then` [#1576](https://github.com/opendp/opendp/pull/1576) [#1575](https://github.com/opendp/opendp/pull/1575)
    - Update "who's using OpenDP" section [#1661](https://github.com/opendp/opendp/pull/1661)
    - Update links to match new docs structure [#1638](https://github.com/opendp/opendp/pull/1638)
    - Misc. copy-edits on the API User Guide [#1659](https://github.com/opendp/opendp/pull/1659)
    - Upgrade pydata theme for search snippets [#1640](https://github.com/opendp/opendp/pull/1640)
    - Update for "who's using opendp" section  #1593 [#1661](https://github.com/opendp/opendp/pull/1661)
    - Update links to match new docs structure [#1638](https://github.com/opendp/opendp/pull/1638)
- R docs and linting:
    - R examples in docs via stand-alone files [#1494](https://github.com/opendp/opendp/pull/1494)
    - Fix r-doc comment with missing closing tag [#1493](https://github.com/opendp/opendp/pull/1493)
    - Add R example to "typical workflow" [#1466](https://github.com/opendp/opendp/pull/1466)
    - One measurement example for R [#1557](https://github.com/opendp/opendp/pull/1557)
    - R favicons [#1298](https://github.com/opendp/opendp/pull/1298)
    - A concept on every R function, so the page is better organized [#1299](https://github.com/opendp/opendp/pull/1299)
    - R doc README [#1247](https://github.com/opendp/opendp/pull/1247)
    - Tidy up R docs header [#1246](https://github.com/opendp/opendp/pull/1246)
    - Make the dependency on r-docs explicit... at the cost of slowing down the docs build [#1248](https://github.com/opendp/opendp/pull/1248)
    - Subheadings in R docs [#1245](https://github.com/opendp/opendp/pull/1245)
    - Do not generate NEWS.md [#1302](https://github.com/opendp/opendp/pull/1302)
    - R linting [#1344](https://github.com/opendp/opendp/pull/1344) [#1408](https://github.com/opendp/opendp/pull/1408)

### Changed

- Renaming:
    - In a handful of locations, change "udf" to "plugin" [#1528](https://github.com/opendp/opendp/pull/1528)
    - rename `TotalOrd` trait to `ProductOrd` [#1362](https://github.com/opendp/opendp/pull/1362)
- Mechanisms:
    - Implement partition distance [#1167](https://github.com/opendp/opendp/pull/1167)
    - Refactor output perturbation mechanisms [#1318](https://github.com/opendp/opendp/pull/1318)
    - Break apart bernoulli sampler traits, add constant-time impl [#1325](https://github.com/opendp/opendp/pull/1325)
- Developer docs and comments:
    - For Rust example in getting-started docs, use normal `cargo run` rather than trying to run as script with nightly [#1612](https://github.com/opendp/opendp/pull/1612)
    - Devs should install all optional dependencies [#1522](https://github.com/opendp/opendp/pull/1522)
    - Explain docs build in each target language [#1499](https://github.com/opendp/opendp/pull/1499)
    - Update LICENSE [#1455](https://github.com/opendp/opendp/pull/1455)
    - Update and fix typos on dev instructions [#1231](https://github.com/opendp/opendp/pull/1231)
    - Explain relationship between `bindings` and `derive` [#1229](https://github.com/opendp/opendp/pull/1229)
    - Consolidate tools requirements [#1444](https://github.com/opendp/opendp/pull/1444)
    - Add a badge for docs.rs [#1435](https://github.com/opendp/opendp/pull/1435)
    - Explain extra installs for R / Add "is" on homepage [#1448](https://github.com/opendp/opendp/pull/1448)
    - Explain that R install does not require pre-compiled code [#1423](https://github.com/opendp/opendp/pull/1423)
    - Just add a note to explain duplication [#1484](https://github.com/opendp/opendp/pull/1484)
- CI and utilities:
    - util script for RST to NB [#1483](https://github.com/opendp/opendp/pull/1483)
    - replace list of "rm" with "git clean" [#1301](https://github.com/opendp/opendp/pull/1301)
    - manylinux2014 -> manylinux_2_24 [#1268](https://github.com/opendp/opendp/pull/1268)
    - Now if "stable" and "dry-run" are selected, will append "-dev" [#1267](https://github.com/opendp/opendp/pull/1267)
    - Upgrade github actions [#1378](https://github.com/opendp/opendp/pull/1378)
    - Minimal cargo test [#1356](https://github.com/opendp/opendp/pull/1356)
    - Add `number_of_spaces` param to `indent` [#1368](https://github.com/opendp/opendp/pull/1368)
    - LaTeX cache, temp, and output files [#1361](https://github.com/opendp/opendp/pull/1361)
    - Speed up smoke-test, mostly by no longer freeing disk space [#1579](https://github.com/opendp/opendp/pull/1579)
    - Package Python with cibuildwheel and setuptools-rust [#1519](https://github.com/opendp/opendp/pull/1519)
    - Move Rust tests to standalone files [#1533](https://github.com/opendp/opendp/pull/1533) [#1548](https://github.com/opendp/opendp/pull/1548)
    - For consistency and simplicity, use `--all-features` [#1526](https://github.com/opendp/opendp/pull/1526)
        - Follow-up with separate builds in smoke-test to fix R/Polars CI [#1515](https://github.com/opendp/opendp/pull/1515)
    - Test coverage and typing in  _convert.py [#1645](https://github.com/opendp/opendp/pull/1645)
    - Test coverage in _lib.py [#1646](https://github.com/opendp/opendp/pull/1646)
    - Test coverage in `context.py` [#1648](https://github.com/opendp/opendp/pull/1648)
    - Test coverage in `mod.py` [#1649](https://github.com/opendp/opendp/pull/1649)
    - Confirm Python coverage ran [#1633](https://github.com/opendp/opendp/pull/1633)
- Minimum Python version:
    - Use features of Python 3.9 [#1558](https://github.com/opendp/opendp/pull/1558)
    - 3.8 -> 3.12 in CI (except for smoke-test) [#1398](https://github.com/opendp/opendp/pull/1398)

### Removed

- Remove build_tool.py [#1300](https://github.com/opendp/opendp/pull/1300)
- Remove sphinx doctest tags [#1450](https://github.com/opendp/opendp/pull/1450)
- Remove `dead_code` markers that do not cause warnings in IDE [#1380](https://github.com/opendp/opendp/pull/1380)
- Remove putting-it-together.rst, and move its diagram [#1353](https://github.com/opendp/opendp/pull/1353)
- Remove `@versioned` from generated code [#1263](https://github.com/opendp/opendp/pull/1263)
- Fix Rust build warning by removing reference to `poly` [#1226](https://github.com/opendp/opendp/pull/1226)

### Fixed

- CI
    - Cancel old smoke-test CI runs [#1566](https://github.com/opendp/opendp/pull/1566)
    - In CI, reverse ternary, so it is not confused by false-y empty string [#1249](https://github.com/opendp/opendp/pull/1249)
    - Fix nightly docs build [#1464](https://github.com/opendp/opendp/pull/1464)
    - Remove check from nightly, so it only calls release [#1417](https://github.com/opendp/opendp/pull/1417)
    - Fix nightly by checking `inputs.fake` on each [#1350](https://github.com/opendp/opendp/pull/1350)
    - Fix weekly-doc-check [#1429](https://github.com/opendp/opendp/pull/1429)
    - When publishing, pip cache's post-setup fails [#1686](https://github.com/opendp/opendp/pull/1686)
- Python
    - Regenerate python code to include example [#1615](https://github.com/opendp/opendp/pull/1615)
    - Add `get_np_csprng` wrapper function, so we can remove the last `skipif` [#1562](https://github.com/opendp/opendp/pull/1562)
    - Replace datetime.now() with constant: Previously, tests would only pass within a certain date range [#1561](https://github.com/opendp/opendp/pull/1561)
    - Fix split_by_weights [#1456](https://github.com/opendp/opendp/pull/1456)
    - Address numpy test failures [#1348](https://github.com/opendp/opendp/pull/1348)
    - Add setuptools to requirements [#1415](https://github.com/opendp/opendp/pull/1415)
    - Add setuptools, fix nightly? [#1427](https://github.com/opendp/opendp/pull/1427)
    - Fix subcontext metric space [#1443](https://github.com/opendp/opendp/pull/1443)
    - Bound Numpy version to 1.x [#1682](https://github.com/opendp/opendp/pull/1682)
    - Misc copy-edits on the api user guide [#1659](https://github.com/opendp/opendp/pull/1659)
    - Upgrade Pydata theme for search snippets [#1640](https://github.com/opendp/opendp/pull/1640)
- Rust
    - For generate_header `def` and `use`, change feature from `ffi` to `bindings` [#1434](https://github.com/opendp/opendp/pull/1434)
    - Avoid panic in ALP histogram [#1240](https://github.com/opendp/opendp/pull/1240)
- R
    - Resolve R warning when calling Rf_error in C [#1536](https://github.com/opendp/opendp/pull/1536)
    - R-docs artifact: We were uploading r-docs with v2, downloading with v4 [#1480](https://github.com/opendp/opendp/pull/1480)
    - delete duplicate def of `parse_or_infer` [#1475](https://github.com/opendp/opendp/pull/1475)
    - Fix erratic R linting errors [#1402](https://github.com/opendp/opendp/pull/1402)
    - Generated changes to R conf [#1252](https://github.com/opendp/opendp/pull/1252)
    - Error on warning from devtools::check [#1253](https://github.com/opendp/opendp/pull/1253)
    - Patch R docs build failure on ci [#1669](https://github.com/opendp/opendp/pull/1669)


## [0.9.2](https://github.com/opendp/opendp/compare/v0.9.1...v0.9.2) - 2024-02-08

### Fixed
- Ignore nitpicky Sphinx warnings on old library versions [#1218](https://github.com/opendp/opendp/pull/1218)

## [0.9.1](https://github.com/opendp/opendp/compare/v0.9.0...v0.9.1) - 2024-02-07

### Fixed
- Fix CI for GitHub release [#1215](https://github.com/opendp/opendp/pull/1215)

## [0.9.0](https://github.com/opendp/opendp/compare/v0.8.0...v0.9.0) - 2024-02-07

### Added
- R language bindings [#679](https://github.com/opendp/opendp/pull/679)
    - All library functionality is available, except for defining your own library primitives in R code
- New transformations/measurements
    - DP PCA [#1045](https://github.com/opendp/opendp/pull/1045)
    - Exponential mechanism via `make_report_noisy_max_gumbel` [#704](https://github.com/opendp/opendp/pull/704)
    - Quantile scoring transformation `make_quantile_score_candidates` [#702](https://github.com/opendp/opendp/pull/702)
    - `make_alp_queryable` may now be used from Python [#747](https://github.com/opendp/opendp/pull/747)
    - All compositors now allow concurrent composition of interactive measurements [#958](https://github.com/opendp/opendp/pull/958)
- Expanded functionality of user-defined library primitives
    - Define your own domains, metrics and measures from Python [#871](https://github.com/opendp/opendp/pull/871) [#873](https://github.com/opendp/opendp/pull/873)
    - Domains may carry arbitrary descriptors [#1044](https://github.com/opendp/opendp/pull/1044)
    - Construct your own queryables from Python [#870](https://github.com/opendp/opendp/pull/870)
- Proofs from Vicki Xu, Hanwen Zhang, Zachary Ratliff and Michael Shoemate
    - `make_randomized_response_bool` [#490](https://github.com/opendp/opendp/pull/490)
    - `SampleBernoulli` [#496](https://github.com/opendp/opendp/pull/496)
    - `make_is_equal` [#514](https://github.com/opendp/opendp/pull/514)
    - `SampleUniformIntBelow` [#1183](https://github.com/opendp/opendp/pull/1183)
- The OpenDP Python package now supports PEP 561 type information [#738](https://github.com/opendp/opendp/pull/738)
- The OpenDP Rust crate is now thread-safe [#874](https://github.com/opendp/opendp/pull/874)
- Documentation, Typing and CI improvements from Chuck McCallum
    - CI: MyPy type-checking, link-checking in docs, code coverage, Rust formatting
    - Rust stack traces are now hidden by default [#1138](https://github.com/opendp/opendp/pull/1138)
- FFI module in Rust is now public, allowing you to write your own lightweight FFI [#1150](https://github.com/opendp/opendp/pull/1150)
- C dependencies on GMP/MPFR have been replaced with dashu [#1141](https://github.com/opendp/opendp/pull/1141)
    - The OpenDP Rust library can now be built easily on Windows and is a much more lightweight Rust dependency

### Changed
- `TO` argument on user-defined measurements is now optional [#1147](https://github.com/opendp/opendp/pull/1147)
- raw functions can now be chained as postprocessors onto measurements

### Fixed
- Imports in the Python `context` module no longer pollute the prelude [#1187](https://github.com/opendp/opendp/pull/1187)


## [0.8.0](https://github.com/opendp/opendp/compare/v0.7.0...v0.8.0) - 2023-08-11

### Added
- Partial constructors: each `make_*` constructor now has a `then_*` variant [#689](https://github.com/opendp/opendp/pull/689) [#761](https://github.com/opendp/opendp/pull/761)
    - all `make_*` have gained two leading arguments: `input_domain` and `input_metric`
    - all `then_*` have same arguments as `make_*`, sans `input_domain` and `input_metric`
        - when chaining, `then_*` tunes to the previous transformation/metric space
    - to migrate, replace `make_*` with `then_*`, and then remove redundant arguments
    - [#687](https://github.com/opendp/opendp/pull/687) [#690](https://github.com/opendp/opendp/pull/690) [#692](https://github.com/opendp/opendp/pull/692) [#712](https://github.com/opendp/opendp/pull/712) [#713](https://github.com/opendp/opendp/pull/713) [#798](https://github.com/opendp/opendp/pull/798) [#799](https://github.com/opendp/opendp/pull/799) [#802](https://github.com/opendp/opendp/pull/802) [#803](https://github.com/opendp/opendp/pull/803) [#804](https://github.com/opendp/opendp/pull/804) [#808](https://github.com/opendp/opendp/pull/808) [#810](https://github.com/opendp/opendp/pull/810) [#813](https://github.com/opendp/opendp/pull/813) [#815](https://github.com/opendp/opendp/pull/815) [#816](https://github.com/opendp/opendp/pull/816)
- (preview) Context API for Python, giving a more succinct alternative to `>>` [#750](https://github.com/opendp/opendp/pull/750)
    - `context.query().clamp(bounds).sum().laplace().release()`
    - automatically tunes a free parameter (like the scale) to satisfy privacy-loss bound
    - mediates queries to the interactive compositor/dataset inside `context`
    - [#749](https://github.com/opendp/opendp/pull/749)
- Support for `aarch64` architecture on Linux [#843](https://github.com/opendp/opendp/pull/843)
- Nightly builds can now be downloaded from PyPi: `pip install opendp --pre` [#879](https://github.com/opendp/opendp/pull/879) [#880](https://github.com/opendp/opendp/pull/880)
- Proofs for `make_row_by_row` [#688](https://github.com/opendp/opendp/pull/688), `make_clamp` [#512](https://github.com/opendp/opendp/pull/512)
- Transformations throughout library support any valid combination of domain descriptors
    - for example, all data preprocessors now also work under bounded DP

### Changed
- Changed constructor names: 
    - `make_base_laplace`, `make_base_discrete_laplace` -> `make_laplace` [#736](https://github.com/opendp/opendp/pull/736)
    - `make_base_gaussian`, `make_base_discrete_gaussian` -> `make_gaussian` [#800](https://github.com/opendp/opendp/pull/800)
    - `make_sized_bounded_sum`, `make_bounded_sum` -> `make_sum` [#801](https://github.com/opendp/opendp/pull/801)
    - `make_sized_bounded_mean` -> `make_mean` [#806](https://github.com/opendp/opendp/pull/806)
    - `make_sized_bounded_variance` -> `make_variance` [#807](https://github.com/opendp/opendp/pull/807)
    - `dp.c.make_user_measurement` -> `dp.m.make_user_measurement` [#884](https://github.com/opendp/opendp/pull/884)
    - `dp.c.make_user_transformation` -> `dp.m.make_user_transformation` [#884](https://github.com/opendp/opendp/pull/884)
    - `dp.c.make_user_postprocessor` -> `dp.new_function` [#884](https://github.com/opendp/opendp/pull/884)
    - `make_base_ptr` -> `make_base_laplace_threshold` [#849](https://github.com/opendp/opendp/pull/849)
        - changed the privacy map to emit fixed (ε, δ) pairs
- Reordered arguments to `make_user_transformation` and `make_user_measurement` 
    - `input_domain` and `input_metric` now leading to enable `then_*` variants
- `make_identity` is now `honest-but-curious` in Python, but is general over all choices of domains/metrics [#814](https://github.com/opendp/opendp/pull/814)
- (Rust-only) sparse histogram APIs have been updated to prepare for Python [#756](https://github.com/opendp/opendp/pull/756)
    - `make_base_alp_with_hashers` -> `make_alp_state_with_hashers`
    - `make_base_alp` -> `make_alp_state`
    - `make_alp_histogram_post_process` -> `make_alp_queryable`
    - thank you Christian Lebeda! (https://github.com/ChristianLebeda)
- (Rust-only) Transformations and Measurements made read-only [#706](https://github.com/opendp/opendp/pull/706)

### Fixed
- Infinite loop converting from ρ to ε when δ=0 [#845](https://github.com/opendp/opendp/pull/845)

### Deprecated
- All dataframe transformations, in anticipation of a new Polars backend in an upcoming release


## [0.7.0] - 2023-05-18
[0.7.0]: https://github.com/opendp/opendp/compare/v0.6.2...v0.7.0

### Added
- FFI and Python interfaces for creating and accessing Domains, Metrics, and Measures ([#637](https://github.com/opendp/opendp/pull/637))
- Queryables and supporting infrastructure for interactive Measurements ([#618](https://github.com/opendp/opendp/pull/618)), ([#675](https://github.com/opendp/opendp/pull/675))
- Constructor for sequential composition of Measurements ([#674](https://github.com/opendp/opendp/pull/674))
- Checks for compatibility between pairings of Domains and Metrics/Measures ([#604](https://github.com/opendp/opendp/pull/604))
- Python `opendp.extrinsics` module for code contributions and proofs outside of Rust ([#693](https://github.com/opendp/opendp/pull/693))
- Docs: [First Look at DP](https://docs.opendp.org/en/v0.7.0/user/first-look-at-DP.html) notebook ([#666](https://github.com/opendp/opendp/pull/666))
- Docs: [Compositors](https://docs.opendp.org/en/v0.7.0/user/combinators/compositors.html) notebook, with usage of interactive Measurements ([#735](https://github.com/opendp/opendp/pull/735))

### Changed
- Incorporated Domain instances into some constructor signatures ([#650](https://github.com/opendp/opendp/pull/650))
- Simplified postprocessors to Function (from previous full Transformation) ([#648](https://github.com/opendp/opendp/pull/648))
- Moved some Domain logic from type-inherent constraints to runtime checks of more general types ([#645](https://github.com/opendp/opendp/pull/645)), ([#696](https://github.com/opendp/opendp/pull/696))
  - Remove SizedDomain in favor of a runtime size descriptor on VectorDomain
  - Remove BoundedDomain in favor of a runtime bounds descriptor on AtomDomain
  - Remove InherentNullDomain in favor of a runtime nullity descriptor on AtomDomain
- Removed the default Domain limitation on [user-defined callbacks](https://docs.opendp.org/en/v0.7.0/user/combinators.html#user-defined-callbacks),
  and renamed constructors from `make_default_user_XXX()` to `make_user_XXX` ([#650](https://github.com/opendp/opendp/pull/650))
- Docs: Improved the clarity of the [User Guide](https://docs.opendp.org/en/v0.7.0/user/index.html) based on feedback ([#639](https://github.com/opendp/opendp/pull/639))
- Docs: Renamed the Developer Guide to [Contributor Guide](https://docs.opendp.org/en/v0.7.0/contributor/index.html) ([#639](https://github.com/opendp/opendp/pull/639))

### Deprecated
- AllDomain in the Python bindings, with a warning to switch to AtomDomain ([#645](https://github.com/opendp/opendp/pull/645))

### Removed
- The `output_domain` field of Measurement struct ([#647](https://github.com/opendp/opendp/pull/647))

### Fixed
- Switched to from `backtrace` crate to `std::backtrace`, and fixed some corner cases, for much faster backtrace resolution ([#691](https://github.com/opendp/opendp/pull/691))
- Whole-codebase reformat using `rustfmt` to minimize spurious churn in the future ([#669](https://github.com/opendp/opendp/pull/669))


## [0.6.2] - 2023-02-06
[0.6.2]: https://github.com/opendp/opendp/compare/v0.6.1...v0.6.2

### Added
- [support for user-defined callbacks under explicit opt-in](https://docs.opendp.org/en/v0.6.2/user/combinators.html#user-defined-callbacks)
    - researchers may construct their own transformations, measurements and postprocessors in Python
    - these "custom" components may be interleaved with other components in the library
- expanded docs.opendp.org User Guide with more explanatory notebooks
- "contrib" proofs for CKS20 sampler algorithms
- "contrib" proof for ρ-zCDP to ε(δ)-DP conversion
- CITATION.cff [#552](https://github.com/opendp/opendp/pull/552)

### Fixed
- cleanup of accuracy utilities [#626](https://github.com/opendp/opendp/issues/626)
    * `discrete_gaussian_scale_to_accuracy` returns an accuracy one too large when the scale is on the lower edge
    * improve float precision of `laplacian_scale_to_accuracy` and `accuracy_to_laplacian_scale`
    * Reported by Alex Whitworth (@alexWhitworth). Thank you!
- clamp negative epsilon in `make_zCDP_to_approxDP` when delta is large [#621](https://github.com/opendp/opendp/issues/621)
    * Reported by Marika Swanberg and Shlomi Hod. Thank you!
- resolve build warnings from metadata in version tags


## [0.6.1] - 2022-10-27
[0.6.1]: https://github.com/opendp/opendp/compare/v0.6.0...v0.6.1

### Fixed
- docs.rs failed to render due to Katex dependency


## [0.6.0] - 2022-10-26
[0.6.0]: https://github.com/opendp/opendp/compare/v0.5.0...v0.6.0

### Added
- Restructured and expanded documentation on docs.opendp.org
    - Moved notebooks into the documentation site
    - Updated developer documentation and added introductions to Rust and proof-writing
- Much more thorough API documentation and links to corresponding Rust documentation
- Documentation throughout the Rust library, as well as proof definition stubs
- Additional combinators for converting the privacy measure
    - `make_pureDP_to_fixed_approxDP` to convert ε to (ε, 0)-approx DP
    - `make_pureDP_to_zCDP` to convert ε to ρ
- Additional accuracy functions for discrete noise mechanisms
    - `discrete_laplacian_scale_to_accuracy`
    - `discrete_gaussian_scale_to_accuracy`
    - `accuracy_to_discrete_laplacian_scale`
    - `accuracy_to_discrete_gaussian_scale`
- `make_b_ary_tree` Lipschitz transformation. Use in conjunction with:
    - `make_consistent_b_ary_tree` to retrieve consistent leaf node counts
    - `make_quantiles_from_counts` to retrieve quantile estimates
    - `make_cdf` to estimate a discretized cumulative distribution function
- `make_subset_by`, `make_df_is_equal` and `make_df_cast_default` transformations 
    - used for simple dataframe subsetting
- `make_chain_tm` combinator for postprocessing
- Updates for proof-writing:
    - `rust/src/lib.sty` contains a collection of latex macros to aid in cross-linking and maintenance
    - See the proof-writing section of the developer documentation
    - PRs with .tex proof documents are rendered by a bot
    - Documentation will now embed links to proof documents that are adjacent to source files
    - Proof documents are automatically hosted and versioned on docs.opendp.org
- An initial proof for `make_count` (by @silviacasac, @cwagaman @gracetian6).

### Changed
- Renamed `meas` to `measurements`, `trans` to `transformations` and `comb` to `combinators`
- Added an `honest-but-curious` feature flag to `make_population_amplification`

### Fixed
- Python bindings check that C integers do not overflow
- Fixed clamping behaviour on `make_lipschitz_float_mul`
- Let the type of the sensitivity supplied to `make_base_discrete_gaussian` vary according to type `QI`
- Fix FFI dispatch in fixed approximate DP composition


## [0.5.0] - 2022-08-23
[0.5.0]: https://github.com/opendp/opendp/compare/v0.4.0...v0.5.0

### Added
- Account for finite data types in aggregators based on our paper [CSVW22](https://arxiv.org/abs/2207.10635)
    - For the [sum #467](https://github.com/opendp/opendp/pull/467), [variance #475](https://github.com/opendp/opendp/pull/475) and [mean #476](https://github.com/opendp/opendp/pull/476)
    - Formalize privacy analysis of data ordering [#465](https://github.com/opendp/opendp/pull/465) [#466](https://github.com/opendp/opendp/pull/466)
- Stability/privacy relations replaced with maps [#463](https://github.com/opendp/opendp/pull/463)
    - You can now call `.map` on transformations and measurements to directly get the tightest `d_out`
- Composition of measurements [#482](https://github.com/opendp/opendp/pull/482)
    - Permits arbitrary nestings of compositions of an arbitrary number of measurements
- Discrete noise mechanisms from [CKS20](https://arxiv.org/pdf/2004.00010.pdf)
    - `make_base_discrete_laplace` is equivalent to `make_base_geometric`, but executes in a constant-time number of operations
    - `make_base_discrete_gaussian` for the discrete gaussian mechanism
- Add zero-concentrated differential privacy to the gaussian and discrete gaussian mechanisms
    - Output measure is now always `ZeroConcentratedDivergence<Q>`, and output distance is in terms of rho
- Add combinator to cast a measurement's output measure from `ZeroConcentratedDivergence<Q>` to `SmoothedMaxDivergence<Q>`
    - `meas_smd = opendp.comb.make_zCDP_to_approxDP(meas_zcd)`
- The `SmoothedMaxDivergence<Q>` measure represents distances as an `ε(δ)` privacy curve: 
    - Can construct a curve by invoking the map: `curve = meas_smd.map(d_in)`
    - Can evaluate a curve at a given delta `epsilon = curve.epsilon(delta)`
- Add `make_fix_delta` combinator to fix the delta parameter in a `SmoothedMaxDivergence<Q>` measure
    - The resulting measure is `FixedSmoothedMaxDivergence<Q>`, where the output distance is an `(ε, δ)` pair
    - `eps, delta = make_fix_delta(meas_smd, delta=1e-8).map(d_in)`
    - The fixed measure supports composition (unlike the curve measure)
- Utility functions `set_default_float_type` and `set_default_int_type` to set the default bit depth of ints and floats
- Exponential search when bounds are not specified in binary search utilities [#453](https://github.com/opendp/opendp/pull/453)
- Support for Apple silicon (`aarch64-apple-darwin` target)

### Changed
- Switched to a single Rust crate (merged `opendp-ffi` into `opendp`)
- Updated documentation to reflect feedback from users and added more example notebooks
- Packaging for Contributor License Agreements
- Improved formatting of rust stack traces in Python
- Expanded error-indexes

### Deprecated
- `make_base_geometric` in favor of the more efficient `make_base_discrete_laplace`
    - Constant-time execution can still be accessed via `make_base_discrete_laplace_linear`

### Removed
- `make_base_analytic_gaussian` in favor of the (now generally tighter) `make_base_gaussian`
    - This would have been a deprecation, but updating to be consistent with forward maps is nontrivial

### Fixed
- Rust documentation on docs.rs is built with "untrusted" flag enabled
- Python documentation for historical versions is rebuilt on correct tag
- Avoid potential infinite loop in binary search utility

### Security
- Replace the underlying implementation of `make_base_laplace` and `make_base_gaussian` to [address precision-based attacks](https://tpdp.journalprivacyconfidentiality.org/2022/papers/HaneyDHSH22.pdf)
    - Both measurements map input floats exactly to an integer discretization, apply discrete laplace or discrete gaussian noise, and then postprocess back to floats
    - The discretization is on ℤ*2^k, where k can be configured, similar to the Google Differential Privacy Library
    - In contrast to the Google library, the approximation to real sampling continues to improve as k is chosen to be smaller than -45. We choose a k of -1074, which matches the subnormal ULP, giving a tight privacy map
- Fixed function in `make_randomized_response_bool`
    - from proofwriting by Vicki Xu and Hanwen Zhang [#481](https://github.com/opendp/opendp/pull/481)
- Multiplicative difference in probabilities in linear-time discrete laplace sampler are now exact around zero
    - eliminates an un-accounted δ < ulp(e^-(1/scale)) from differing conservative roundings
- Biased bernoulli sampler on float probabilities is now exact
    - eliminates an un-accounted δ < 2^-500 in RR and linear-time discrete laplace sampler
    - from proofwriting by Vicki Xu and Hanwen Zhang [#496](https://github.com/opendp/opendp/pull/496)
- Added conservative rounding when converting between MFPR floats and native floats
    - MFPR has a different exponent range, which could lead to unintended rounding of floats that are out of exponent range

### Migration
- `make_base_gaussian`'s output measure is now ZeroConcentratedDivergence.
    - This means the output distance is now a single scalar, rho (it used to be an (ε, δ) tuple)
    - Use `adp_meas = opendp.comb.make_zCDP_to_approxDP(zcdp_meas)` to convert to an ε(δ) curve. 
    - Use `fadp_meas = opendp.comb.make_fix_delta(adp_meas)` to change output distance from an ε(δ) curve to an (ε, δ) tuple
        - `fadp_meas.check(d_in, (ε, δ))` is equivalent to the check on `make_base_gaussian` in 0.4
- replace `make_base_analytic_gaussian` with `make_base_gaussian`
- replace `make_base_geometric` with `make_base_discrete_laplace`
- `make_basic_composition` accepts a list of measurements as its first argument (it used to have two arguments)
- slight increase in sensitivities/privacy utilization across the library as a byproduct of floating-point attack mitigations


## [0.4.0] - 2021-12-10
[0.4.0]: https://github.com/opendp/opendp/compare/v0.3.0...v0.4.0

### Added
- `make_randomized_response_bool` and `make_randomized_response` for local differential privacy.
- `make_base_analytic_gaussian` for a tighter, analytic calibration of the gaussian mechanism.  
- `make_population_amplification` combinator for privacy amplification by subsampling.
- `make_drop_null` transformation for dropping null values in nullish data.
- `make_find`, `make_find_bin` and `make_index` transformations for categorical relabeling and binning.
- `make_base_alp` for histograms via approximate laplace projections from Christian Lebeda (https://github.com/ChristianLebeda)
- `make_base_ptr` for stability histograms via propose-test-release.
- Added floating-point numbers to the admissible output types on integer queries like `make_count`, `make_count_by`, `make_count_by_categories` and `make_count_distinct`.
- Simple attack notebook from Oren Renard (https://github.com/orespo)
- Support for Numpy data types.
- Release helper script

### Fixed
- Resolved memory leaks in FFI

### Changed
- moved windows patch directory into `/rust`
- added minimum rust version of 1.56 and updated to the 2021 edition.
- dropped sized-ness domain requirements from `make_count_by`

### Security
- `make_base_stability` underestimated the sensitivity of queries. Removed in favor of `make_base_ptr`.
- Floating-point arithmetic throughout the library now has explicit rounding modes such that the budget is always slightly overestimated. There is still some potential for small floating-point leaks via rounding in floating-point aggregations.
- Fixed integer truncation issue in the sized bounded sum privacy relation.
- The resize relation is now looser to account for a worst-case situation where d_in records removed, and d_in new records are imputed.


## [0.3.0] - 2021-09-21
[0.3.0]: https://github.com/opendp/opendp/compare/v0.2.4...v0.3.0

### Changed
- All unvetted modules (which is currently all modules) are tagged with the "contrib" feature
- Programs must explicitly opt-in to access the "contrib" feature


## [0.2.4] - 2021-09-20
[0.2.4]: https://github.com/opendp/opendp/compare/v0.2.3...v0.2.4

### Fixed
- Version tag


## [0.2.3] - 2021-09-20
[0.2.3]: https://github.com/opendp/opendp/compare/v0.2.2...v0.2.3

### Fixed
- Version tag


## [0.2.2] - 2021-09-20
[0.2.2]: https://github.com/opendp/opendp/compare/v0.2.1...v0.2.2

### Added
- User guide, developer guide, and general focus on documentation
- Examples folder has complete notebooks for getting started with the library

### Fixed
- Usability issues in the FFI layer for `make_count_by_categories` and `make_count_by`
- The FFI for make_identity ensures proper domain metric pairing


## [0.2.1] - 2021-09-09
[0.2.1]: https://github.com/opendp/opendp/compare/v0.2.0...v0.2.1

### Added
- Functions to convert between accuracy and noise scale for laplace, gaussian and geometric noise
- Error messages when chaining include a plaintext description of the mismatched domains or metrics


## [0.2.0] - 2021-08-31
[0.2.0]: https://github.com/opendp/opendp/compare/v0.1.0...v0.2.0

### Added
- User guide outline
- Initial exemplar python notebooks
- Binary search utilities in Python
- `Vec<String>` and `HashMap<K, V>` data loaders
- Resize transformation for making `VectorDomain<D>` sized
- TotalOrd trait for consistency with proofs 

### Changed
- General renaming of library interfaces. [See issue #181](https://github.com/opendp/opendp/issues/181).

### Removed
- Scalar clamping

### Fixed
- Adjust output domain on `make_count_by_categories` to make it chainable with measurements


## [0.1.0] - 2021-08-05
[0.1.0]: https://github.com/opendp/opendp/releases/tag/v0.1.0

### Added
* Initial release.


## Instructions
The format of this file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). It is processed by
scripts when generating a release, so please maintain the existing format.

Whenever you're preparing a significant commit, add a bullet list entry summarizing the change under the
X.Y.Z-dev heading at the top. Entries should be grouped in sections based on the kind of change.
Please use the following sections, maintaining the same ordering. If the appropriate section isn't present yet,
just add it by copying from those below.

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
### Migration

When a new version is released, a script will turn the Unreleased heading into a new heading with appropriate values for the version, date, and link. 
Then the script will generate a new Unreleased section for future work.
Please keep the existing dummy heading and link as they are, so that things operate correctly. Thanks!