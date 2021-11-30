import configparser
import toml

# all version numbers should be:
version = open("VERSION", 'r').read().strip()
assert version != "0.0.0-development", "Please update the repository with the current version."
print("Checking if all version numbers are synchronized at", version)

# check that python version is set properly
config = configparser.RawConfigParser()
config.read('python/setup.cfg')
assert config['metadata']['version'] == version, \
    "python/setup.cfg package version is incorrect"

# check that opendp crate version is set properly
opendp_toml = toml.load('rust/opendp/Cargo.toml')
assert opendp_toml['package']['version'] == version, \
    "rust/opendp/Cargo.toml crate version is incorrect"

# check that opendp-ffi crate version is set properly
opendp_ffi_toml = toml.load('rust/opendp-ffi/Cargo.toml')
assert opendp_ffi_toml['package']['version'] == version, \
    "rust/opendp-ffi/Cargo.toml crate version is incorrect"

assert opendp_ffi_toml['dependencies']['opendp']['version'] == version, \
    "rust/opendp-ffi/Cargo.toml opendp dependency is incorrect"

print("All version numbers are synchronized")
