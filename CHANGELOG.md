# OpenDP Changelog

This file documents the version history of OpenDP.


## [0.4.0] - 2021-11-15
[0.4.0]: https://github.com/opendp/opendp/compare/v0.3.0...v0.4.0

### Added
- `make_randomized_response_bool` and `make_randomized_response` for local differential privacy.
- `make_base_gaussian` now has an optional boolean to enable tighter analytic gaussian bounds.
- `make_population_amplification` combinator for privacy amplification by subsampling.
- `make_drop_null` transformation for dropping null values in nullish data.
- `make_find`, `make_find_bin` and `make_index` transformations for categorical relabeling and binning.
- `make_count_by_ptr` for stability-based histograms.
- Added floating-point numbers to the admissible output types on integer queries like `make_count`, `make_count_by_categories` and `make_count_distinct`.
- Simple attack notebook (thanks to https://github.com/orespo)

### Changed
- moved windows patch directory into `/rust`
- added minimum rust version of 1.56 and updated to the 2021 edition.

### Security
- `make_base_stability` underestimated the sensitivity of queries. Removed in favor of `make_count_by_ptr`.
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



## [Unreleased]
[Unreleased]: https://github.com/opendp/opendp/compare/stable...HEAD
Template for additions.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security