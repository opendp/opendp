import configparser
import toml

# all version numbers should be:
version = open("../VERSION", 'r').read()

# check that python version is set properly
config = configparser.RawConfigParser()
config.read('../python/setup.cfg')
assert config['metadata']['version'] == version

# check that opendp crate version is set properly
opendp_toml = toml.load('../rust/opendp/Cargo.toml')
assert opendp_toml['package']['version'] == version

# check that opendp-ffi crate version is set properly
opendp_ffi_toml = toml.load('../rust/opendp-ffi/Cargo.toml')
assert opendp_ffi_toml['package']['version'] == version
assert opendp_ffi_toml['dependencies']['opendp']['version'] == version
