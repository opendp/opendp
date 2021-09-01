# OpenDP Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
[Unreleased]: https://github.com/opendp/opendp/compare/v0.2.0...HEAD

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security

## [0.2.0] - 2021-8-31
[0.2.0]: https://github.com/opendp/opendp/compare/v0.1.0...v0.2.0

### Added
- User guide outline
- Initial exemplar python notebooks
- Binary search utilities in Python
- `Vec<String>` and `HashMap<K, V>` data loaders
- Resize transformation for making `VectorDomain<D>` sized
- TotalOrd trait for consistency with proofs 

### Removed
- Remove scalar clamping

### Fixed
- Adjust output domain on `make_count_by_categories` to make it chainable with measurements

## [0.1.0] - 2021-08-05
[0.1.0]: https://github.com/opendp/opendp/releases/tag/v0.1.0

### Added
* Initial release.
