# OpenDP Changelog

This file documents the version history of OpenDP. The links on each version number will take you to a comparison
showing the source changes from the previous version.

Please keep this up to date, following the [instructions](#instructions) below.


## [Unreleased](https://github.com/opendp/opendp/compare/stable...HEAD)


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

When a new version is released, a script will turn the Unreleased heading into a new heading with appropriate values
for the version, date, and link. Then the script they will generate a new Unreleased section for future work.
Please keep the existing dummy heading and link as they are, so that things operate correctly. Thanks!
