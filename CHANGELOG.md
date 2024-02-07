# OpenDP Changelog

This file documents the version history of OpenDP. The links on each version number will take you to a comparison
showing the source changes from the previous version.


## [0.9.0-dev](https://github.com/opendp/opendp/compare/v0.8.0...HEAD) - TBD

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
