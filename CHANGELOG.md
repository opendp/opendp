# OpenDP Changelog

This file documents the version history of OpenDP. The links on each version number will take you to a comparison
showing the source changes from the previous version.

Please keep this up to date, following the [instructions](#instructions) below.


## [Unreleased](https://github.com/opendp/opendp/compare/stable...HEAD)

### Fixed
- docs.rs failed to render due to Katex dependency


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
[Unreleased](#unreleased) heading at the top. Entries should be grouped in sections based on the kind of change.
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
