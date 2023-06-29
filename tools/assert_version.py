import configparser
import semver
import toml

# all version numbers should be:
version = open("VERSION", 'r').read().strip()
version = semver.Version.parse(version)
assert version.prerelease != "dev", "Please update the repository with the current version."
print("Checking if all version numbers are synchronized at", version)

# check that Python version is set properly
# Python doesn't allow arbitrary prerelease tags, supporting only (a|b|rc) or synonyms (alpha|beta|c|pre|preview),
# so we map nightly -> alpha.
is_nightly = version.prerelease is not None and version.prerelease.startswith("nightly.")
python_version = version.replace(prerelease=f"alpha.{version.prerelease[8:]}") if is_nightly else version
config = configparser.RawConfigParser()
config.read('python/setup.cfg')
assert config['metadata']['version'] == str(python_version), \
    "python/setup.cfg package version is incorrect"

# check that opendp crate version is set properly
# cargo doesn't like build metadata in dependency references, so we strip that for those.
stripped_version = version.replace(build=None)
opendp_toml = toml.load('rust/Cargo.toml')
assert opendp_toml['workspace']['package']['version'] == version, \
    "rust/Cargo.toml workspace version is incorrect"
assert opendp_toml['dependencies']['opendp_derive']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"
assert opendp_toml['build-dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml build-dependency opendp_tooling version is incorrect"
opendp_derive_toml = toml.load('rust/opendp_derive/Cargo.toml')
assert opendp_derive_toml['dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"

print("All version numbers are synchronized")
